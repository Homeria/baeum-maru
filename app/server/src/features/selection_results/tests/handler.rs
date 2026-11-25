//! 선발 결과 요청 처리 테스트.

use axum::Router;
use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::SqlitePool;

use crate::db::test_support::{DrawnCourse, insert_drawn_course};
use crate::server::test_support::{app, app_as, get, json_body, post};

/// 정원 2명 강좌에 신청 5건을 추첨했다. `close`면 마감까지. 당첨 2명, 대기 3명.
async fn drawn(db: &SqlitePool, close: bool) -> DrawnCourse {
    insert_drawn_course(db, 2, 5, close).await
}

async fn result(router: Router, course: &DrawnCourse) -> Value {
    let uri = format!("/api/courses/{}/selection-result", course.course_id);
    json_body(get(router, &uri).await).await
}

fn id(entry: &Value) -> i64 {
    entry["member"]["applicationId"].as_i64().unwrap()
}

async fn withdraw(router: Router, application_id: i64) -> StatusCode {
    let uri = format!("/api/applications/{application_id}/withdrawal");
    post(router, &uri, &json!({ "reason": null }).to_string())
        .await
        .status()
}

async fn contact(router: Router, application_id: i64, result: &str) -> (StatusCode, Value) {
    let uri = format!("/api/applications/{application_id}/contacts");
    let response = post(router, &uri, &json!({ "result": result }).to_string()).await;
    let status = response.status();
    let body = if status == StatusCode::NO_CONTENT {
        Value::Null
    } else {
        json_body(response).await
    };
    (status, body)
}

async fn correct(router: Router, application_id: i64, reason: &str) -> (StatusCode, Value) {
    let uri = format!("/api/applications/{application_id}/withdrawal-correction");
    let response = post(router, &uri, &json!({ "reason": reason }).to_string()).await;
    let status = response.status();
    let body = if status == StatusCode::NO_CONTENT {
        Value::Null
    } else {
        json_body(response).await
    };
    (status, body)
}

#[tokio::test]
async fn lists_closed_courses_and_shows_result() {
    let (_dir, db, router) = app().await;
    let course = drawn(&db, true).await;

    let list_uri = format!("/api/terms/{}/selection-results", course.term_id);
    let list = json_body(get(router.clone(), &list_uri).await).await;
    let shown = result(router, &course).await;

    assert_eq!(
        list,
        json!({ "items": [{
            "courseId": course.course_id, "capacity": 2, "selected": 2,
            "waitlisted": 3, "openSeats": 0,
            // 대기 1~3번 다음
            "nextAdditional": { "kind": "waitlisted", "waitlistNumber": 4 }
        }] })
    );
    assert_eq!(shown["selected"].as_array().unwrap().len(), 2);
    assert_eq!(shown["waitlisted"][0]["waitlistNumber"], 1);
    assert!(shown["waitlisted"][0]["phoneNumber"].is_string());
    assert_eq!(shown["canRecord"], true);
    assert_eq!(shown["canCorrect"], true);
}

#[tokio::test]
async fn br_w2_read_only_sees_result_without_phone_numbers() {
    let (_dir, db, router) = app_as("read_only").await;
    let course = drawn(&db, true).await;

    let shown = result(router.clone(), &course).await;
    let selected = id(&shown["selected"][0]);

    assert_eq!(shown["waitlisted"][0]["phoneNumber"], Value::Null);
    assert_eq!(shown["canRecord"], false);
    assert_eq!(withdraw(router, selected).await, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn br_c3_not_closed_course_is_conflict() {
    let (_dir, db, router) = app().await;
    let course = drawn(&db, false).await;
    let application_id: i64 = sqlx::query_scalar("SELECT MIN(id) FROM applications")
        .fetch_one(&db)
        .await
        .unwrap();

    let shown = result(router.clone(), &course).await;
    let status = withdraw(router, application_id).await;

    assert_eq!(shown["code"], "SELECTION_NOT_CLOSED");
    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn br_w1_w3_withdrawal_opens_seat_and_turn_accepts() {
    let (_dir, db, router) = app_as("staff").await;
    let course = drawn(&db, true).await;
    let before = result(router.clone(), &course).await;
    let first = id(&before["waitlisted"][0]);
    let second = id(&before["waitlisted"][1]);

    let (full, body) = contact(router.clone(), first, "accepted").await;
    assert_eq!(full, StatusCode::CONFLICT);
    assert_eq!(body["code"], "NO_OPEN_SEAT");

    let selected = id(&before["selected"][0]);
    assert_eq!(
        withdraw(router.clone(), selected).await,
        StatusCode::NO_CONTENT
    );
    let opened = result(router.clone(), &course).await;
    assert_eq!(opened["openSeats"], 1);
    assert_eq!(opened["waitlisted"][0]["contactTurn"], true);

    let (unreachable, _) = contact(router.clone(), first, "unreachable").await;
    let (accepted, _) = contact(router.clone(), second, "accepted").await;
    let (wrong, body) = contact(router.clone(), second, "declined").await;

    assert_eq!(unreachable, StatusCode::NO_CONTENT);
    assert_eq!(accepted, StatusCode::NO_CONTENT);
    assert_eq!(wrong, StatusCode::CONFLICT);
    assert_eq!(body["code"], "SELECTION_CHANGE_NOT_ALLOWED");
    let after = result(router, &course).await;
    assert_eq!(after["openSeats"], 0);
    assert_eq!(after["waitlisted"][0]["lastContact"], "unreachable");
}

#[tokio::test]
async fn br_w8_only_admin_corrects_with_reason() {
    let (_dir, db, admin) = app().await;
    let course = drawn(&db, true).await;
    let waiting = id(&result(admin.clone(), &course).await["waitlisted"][0]);
    assert_eq!(
        withdraw(admin.clone(), waiting).await,
        StatusCode::NO_CONTENT
    );

    let (blank, body) = correct(admin.clone(), waiting, "  ").await;
    let (fixed, _) = correct(admin.clone(), waiting, "다른 회원을 잘못 누름").await;

    assert_eq!(blank, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["fields"][0]["field"], "reason");
    assert_eq!(fixed, StatusCode::NO_CONTENT);
    assert_eq!(id(&result(admin, &course).await["waitlisted"][0]), waiting);
}

#[tokio::test]
async fn br_w8_staff_cannot_correct() {
    let (_dir, db, router) = app_as("staff").await;
    let course = drawn(&db, true).await;
    let selected = id(&result(router.clone(), &course).await["selected"][0]);
    withdraw(router.clone(), selected).await;

    let (status, _) = correct(router.clone(), selected, "잘못 누름").await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(result(router, &course).await["canCorrect"], false);
}
