//! 강좌 요청 처리 테스트.

use axum::Router;
use axum::http::StatusCode;
use serde_json::{Value, json};

use crate::db::test_support::{insert_application, insert_member};
use crate::server::test_support::{
    app, create, create_room, create_subject, create_term, delete, get, json_body, patch, post,
};

/// 학기 하나, 과목 하나(요가), 강의실 하나를 API로 넣어 둔다. (학기 id, 과목 id, 강의실 id)
async fn setup(router: &Router) -> (i64, i64, i64) {
    let term_id = create_term(router.clone()).await;
    let subject_id = create_subject(router.clone(), "요가").await;
    let room_id = create_room(router.clone()).await;
    (term_id, subject_id, room_id)
}

fn course(subject_id: Value, schedules: Value) -> String {
    json!({
        "subjectId": subject_id,
        "level": "초급",
        "section": null,
        "capacity": 20,
        "schedules": schedules,
    })
    .to_string()
}

fn courses_uri(term_id: i64) -> String {
    format!("/api/terms/{term_id}/courses")
}

fn field_names(body: &Value) -> Vec<&str> {
    body["fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|field| field["field"].as_str().unwrap())
        .collect()
}

#[tokio::test]
async fn creates_and_lists_course() {
    let (_dir, _db, router) = app().await;
    let (term_id, subject_id, room_id) = setup(&router).await;
    let body = course(
        json!(subject_id),
        json!([
            { "weekday": 3, "start": "10:00", "end": "11:00", "roomId": null },
            { "weekday": 1, "start": "10:00", "end": "11:00", "roomId": room_id },
        ]),
    );

    let response = post(router.clone(), &courses_uri(term_id), &body).await;

    assert_eq!(response.status(), StatusCode::CREATED);
    let id = json_body(response).await["id"].clone();
    let list = json_body(get(router, &courses_uri(term_id)).await).await;
    assert_eq!(
        list,
        json!({ "items": [{
            "id": id,
            "termId": term_id,
            "subjectId": subject_id,
            "subjectName": "요가",
            "level": "초급",
            "section": null,
            "capacity": 20,
            "schedules": [
                { "weekday": 1, "start": "10:00", "end": "11:00",
                  "room": { "id": room_id, "building": "본관", "name": "201호" } },
                { "weekday": 3, "start": "10:00", "end": "11:00", "room": null },
            ]
        }]})
    );
}

#[tokio::test]
async fn invalid_fields_are_field_errors() {
    let (_dir, _db, router) = app().await;
    let (term_id, _, _) = setup(&router).await;
    let body = json!({
        "subjectId": null,
        "level": null,
        "section": null,
        "capacity": 0,
        "schedules": [{ "weekday": 1, "start": "11:00", "end": "10:00", "roomId": null }],
    })
    .to_string();

    let response = post(router, &courses_uri(term_id), &body).await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = json_body(response).await;
    assert_eq!(field_names(&body), ["subjectId", "capacity", "schedules"]);
    assert_eq!(
        body["fields"][2]["message"],
        "1번째 수업 시간: 종료 시각은 시작 시각보다 늦어야 합니다"
    );
}

#[tokio::test]
async fn weekday_out_of_range_is_malformed() {
    let (_dir, _db, router) = app().await;
    let (term_id, subject_id, _) = setup(&router).await;
    let body = course(
        json!(subject_id),
        json!([{ "weekday": 8, "start": "10:00", "end": "11:00", "roomId": null }]),
    );

    let response = post(router, &courses_uri(term_id), &body).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn missing_subject_and_room_are_field_errors() {
    let (_dir, _db, router) = app().await;
    let (term_id, _, room_id) = setup(&router).await;
    let body = course(
        json!(999),
        json!([
            { "weekday": 1, "start": "10:00", "end": "11:00", "roomId": room_id },
            { "weekday": 3, "start": "10:00", "end": "11:00", "roomId": 998 },
        ]),
    );

    let response = post(router, &courses_uri(term_id), &body).await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = json_body(response).await;
    assert_eq!(field_names(&body), ["subjectId", "schedules"]);
    assert_eq!(
        body["fields"][1]["message"],
        "2번째 수업 시간: 목록에 없는 강의실입니다. 다시 골라 주세요"
    );
}

#[tokio::test]
async fn unknown_term_is_not_found() {
    let (_dir, _db, router) = app().await;
    let (_, subject_id, _) = setup(&router).await;
    let body = course(
        json!(subject_id),
        json!([{ "weekday": 1, "start": "10:00", "end": "11:00", "roomId": null }]),
    );

    let response = get(router.clone(), &courses_uri(999)).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(json_body(response).await["code"], "TERM_NOT_FOUND");
    let response = post(router, &courses_uri(999), &body).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn br_t3_duplicate_course_is_conflict() {
    let (_dir, _db, router) = app().await;
    let (term_id, subject_id, _) = setup(&router).await;
    let body = course(
        json!(subject_id),
        json!([{ "weekday": 1, "start": "10:00", "end": "11:00", "roomId": null }]),
    );
    post(router.clone(), &courses_uri(term_id), &body).await;

    let response = post(router, &courses_uri(term_id), &body).await;

    assert_eq!(response.status(), StatusCode::CONFLICT);
    let body = json_body(response).await;
    assert_eq!(body["code"], "DUPLICATE_COURSE");
    assert_eq!(body["rule"], "BR-T3");
}

#[tokio::test]
async fn gets_updates_and_deletes_course() {
    let (_dir, _db, router) = app().await;
    let (term_id, subject_id, _) = setup(&router).await;
    let body = course(
        json!(subject_id),
        json!([{ "weekday": 1, "start": "10:00", "end": "11:00", "roomId": null }]),
    );
    let id = create(router.clone(), &courses_uri(term_id), &body).await;
    let uri = format!("/api/courses/{id}");

    let changed = course(
        json!(subject_id),
        json!([{ "weekday": 5, "start": "14:00", "end": "15:30", "roomId": null }]),
    );
    let response = patch(router.clone(), &uri, &changed).await;

    assert_eq!(response.status(), StatusCode::OK);
    let read = json_body(get(router.clone(), &uri).await).await;
    assert_eq!(
        read["schedules"],
        json!([{ "weekday": 5, "start": "14:00", "end": "15:30", "room": null }])
    );
    assert_eq!(
        delete(router.clone(), &uri).await.status(),
        StatusCode::NO_CONTENT
    );
    let response = get(router.clone(), &uri).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(json_body(response).await["code"], "COURSE_NOT_FOUND");
    assert_eq!(
        patch(router.clone(), &uri, &changed).await.status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(delete(router, &uri).await.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn br_t6_course_with_application_is_conflict() {
    let (_dir, db, router) = app().await;
    let (term_id, subject_id, _) = setup(&router).await;
    let body = course(
        json!(subject_id),
        json!([{ "weekday": 1, "start": "10:00", "end": "11:00", "roomId": null }]),
    );
    let id = create(router.clone(), &courses_uri(term_id), &body).await;
    let member_id = insert_member(&db, "00001").await.unwrap();
    let staff_id: i64 = sqlx::query_scalar("SELECT id FROM staff")
        .fetch_one(&db)
        .await
        .unwrap();
    insert_application(&db, member_id, id, "submitted", staff_id)
        .await
        .unwrap();

    let response = delete(router.clone(), &format!("/api/courses/{id}")).await;

    assert_eq!(response.status(), StatusCode::CONFLICT);
    let body = json_body(response).await;
    assert_eq!(body["code"], "COURSE_IN_USE");
    assert_eq!(body["rule"], "BR-T6");
    let still = get(router, &format!("/api/courses/{id}")).await;
    assert_eq!(still.status(), StatusCode::OK);
}
