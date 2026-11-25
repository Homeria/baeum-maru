//! 회원 요청 처리 테스트.

use axum::Router;
use axum::http::{StatusCode, header};
use serde_json::{Value, json};
use sqlx::SqlitePool;

use crate::db::test_support::{insert_course, insert_deletion, insert_subject, insert_term};
use crate::server::test_support::{app, create, delete, get, json_body, patch, post};

fn valid(member_no: &str) -> String {
    json!({
        "memberNo": member_no,
        "name": "회원1",
        "phoneNumber": "010-0000-0001",
        "birthDate": "1950-03-01",
        "gender": "female",
    })
    .to_string()
}

async fn count(db: &SqlitePool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM members")
        .fetch_one(db)
        .await
        .unwrap()
}

#[tokio::test]
async fn creates_member() {
    let (_dir, db, router) = app().await;

    let response = post(router, "/api/members", &valid("00001")).await;

    assert_eq!(response.status(), StatusCode::CREATED);
    // body를 읽으면 response가 사라지므로 헤더 글자를 먼저 따로 가져 둠
    let location = response.headers()[header::LOCATION]
        .to_str()
        .unwrap()
        .to_owned();
    let body = json_body(response).await;
    assert_eq!(location, format!("/api/members/{}", body["id"]));
    assert_eq!(count(&db).await, 1);
}

#[tokio::test]
async fn optional_fields_can_be_null() {
    let (_dir, _db, router) = app().await;
    let body = json!({
        "memberNo": "00001",
        "name": "회원1",
        "phoneNumber": "01000000001",
        "birthDate": null,
        "gender": null,
    });

    let response = post(router, "/api/members", &body.to_string()).await;

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn invalid_input_returns_every_field() {
    let (_dir, db, router) = app().await;
    let body = json!({
        "memberNo": "번호",
        "name": " ",
        "phoneNumber": "12",
        "birthDate": "9999-01-01",
        "gender": null,
    });

    let response = post(router, "/api/members", &body.to_string()).await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = json_body(response).await;
    assert_eq!(body["code"], "INVALID_INPUT");
    let fields: Vec<&str> = body["fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|field| field["field"].as_str().unwrap())
        .collect();
    assert_eq!(fields, ["memberNo", "name", "phoneNumber", "birthDate"]);
    assert_eq!(count(&db).await, 0);
}

#[tokio::test]
async fn br_m2_duplicate_member_no_is_conflict() {
    let (_dir, db, router) = app().await;
    post(router.clone(), "/api/members", &valid("00001")).await;

    let response = post(router, "/api/members", &valid("00001")).await;

    assert_eq!(response.status(), StatusCode::CONFLICT);
    let body = json_body(response).await;
    assert_eq!(body["code"], "DUPLICATE_MEMBER_NO");
    assert_eq!(body["rule"], "BR-M2");
    assert_eq!(count(&db).await, 1);
}

#[tokio::test]
async fn malformed_body_does_not_echo_input() {
    let (_dir, _db, router) = app().await;
    let body = json!({
        "memberNo": "00001",
        "name": "회원1",
        "phoneNumber": "01000000001",
        "birthDate": "1950-13-01",
        "gender": null,
    });

    let response = post(router, "/api/members", &body.to_string()).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(response).await;
    assert_eq!(body["code"], "MALFORMED_REQUEST");
    assert!(!body.to_string().contains("1950-13-01"));
}

#[tokio::test]
async fn unknown_gender_is_malformed() {
    let (_dir, _db, router) = app().await;
    let body = json!({
        "memberNo": "00001",
        "name": "회원1",
        "phoneNumber": "01000000001",
        "birthDate": null,
        "gender": "other",
    });

    let response = post(router, "/api/members", &body.to_string()).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn searches_members() {
    let (_dir, _db, router) = app().await;
    post(router.clone(), "/api/members", &valid("00001")).await;
    post(router.clone(), "/api/members", &valid("00002")).await;

    let response = get(router, "/api/members?query=0000-0001&pageSize=10").await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    let body = json_body(response).await;
    assert_eq!(body["total"], 2);
    assert_eq!(body["page"], 1);
    assert_eq!(body["pageSize"], 10);
    let first = &body["items"][0];
    assert_eq!(first["memberNo"], "00001");
    assert_eq!(first["name"], "회원1");
    assert_eq!(first["phoneNumber"], "01000000001");
    assert_eq!(first["birthDate"], "1950-03-01");
    assert_eq!(first["gender"], "female");
}

#[tokio::test]
async fn search_without_params_uses_defaults() {
    let (_dir, _db, router) = app().await;

    let response = get(router, "/api/members").await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["items"], json!([]));
    assert_eq!(body["page"], 1);
    assert_eq!(body["pageSize"], 20);
}

#[tokio::test]
async fn search_rejects_page_out_of_range() {
    let (_dir, _db, router) = app().await;

    let response = get(router, "/api/members?page=0&pageSize=101").await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = json_body(response).await;
    assert_eq!(body["fields"][0]["field"], "page");
    assert_eq!(body["fields"][1]["field"], "pageSize");
}

#[tokio::test]
async fn search_with_broken_page_is_malformed() {
    let (_dir, _db, router) = app().await;

    let response = get(router, "/api/members?page=abc").await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn korean_query_in_url() {
    let (_dir, _db, router) = app().await;
    post(router.clone(), "/api/members", &valid("00001")).await;

    // "회원"을 URL 인코딩
    let response = get(router, "/api/members?query=%ED%9A%8C%EC%9B%90").await;

    let body = json_body(response).await;
    assert_eq!(body["total"], 1);
}

async fn created_id(router: Router, member_no: &str) -> i64 {
    create(router, "/api/members", &valid(member_no)).await
}

#[tokio::test]
async fn gets_one_member() {
    let (_dir, _db, router) = app().await;
    let id = created_id(router.clone(), "00001").await;

    let response = get(router, &format!("/api/members/{id}")).await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["id"], id);
    assert_eq!(body["memberNo"], "00001");
    assert_eq!(body["gender"], "female");
    let created_at = body["createdAt"].as_str().unwrap();
    assert!(created_at.ends_with('Z'), "{created_at}");
    assert_eq!(created_at.len(), "2026-11-15T01:02:03.000Z".len());
    assert_eq!(body["updatedAt"], body["createdAt"]);
}

#[tokio::test]
async fn unknown_member_is_not_found() {
    let (_dir, _db, router) = app().await;

    let response = get(router, "/api/members/999").await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(json_body(response).await["code"], "MEMBER_NOT_FOUND");
}

#[tokio::test]
async fn non_number_id_is_malformed() {
    let (_dir, _db, router) = app().await;

    let response = get(router, "/api/members/abc").await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn updates_member() {
    let (_dir, _db, router) = app().await;
    let id = created_id(router.clone(), "00001").await;
    let changed = json!({
        "memberNo": "00009",
        "name": "회원2",
        "phoneNumber": "032-000-0000",
        "birthDate": null,
        "gender": "male",
    });

    let response = patch(router, &format!("/api/members/{id}"), &changed.to_string()).await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["memberNo"], "00009");
    assert_eq!(body["name"], "회원2");
    assert_eq!(body["phoneNumber"], "0320000000");
    assert_eq!(body["birthDate"], Value::Null);
    assert_eq!(body["gender"], "male");
}

#[tokio::test]
async fn update_checks_input_like_create() {
    let (_dir, _db, router) = app().await;
    let id = created_id(router.clone(), "00001").await;
    let changed = json!({
        "memberNo": "00001",
        "name": "",
        "phoneNumber": "01000000001",
        "birthDate": null,
        "gender": null,
    });

    let response = patch(router, &format!("/api/members/{id}"), &changed.to_string()).await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(json_body(response).await["fields"][0]["field"], "name");
}

#[tokio::test]
async fn br_m2_update_to_taken_member_no_is_conflict() {
    let (_dir, _db, router) = app().await;
    created_id(router.clone(), "00001").await;
    let id = created_id(router.clone(), "00002").await;

    let response = patch(router, &format!("/api/members/{id}"), &valid("00001")).await;

    assert_eq!(response.status(), StatusCode::CONFLICT);
    assert_eq!(json_body(response).await["rule"], "BR-M2");
}

#[tokio::test]
async fn update_unknown_member_is_not_found() {
    let (_dir, _db, router) = app().await;

    let response = patch(router, "/api/members/999", &valid("00001")).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn br_m5_deletes_member() {
    let (_dir, db, router) = app().await;
    let id = created_id(router.clone(), "00001").await;

    let response = delete(router.clone(), &format!("/api/members/{id}")).await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(count(&db).await, 0);
    let again = get(router, &format!("/api/members/{id}")).await;
    assert_eq!(again.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_unknown_member_is_not_found() {
    let (_dir, _db, router) = app().await;

    let response = delete(router, "/api/members/999").await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn br_m5_member_with_application_record_is_conflict() {
    let (_dir, db, router) = app().await;
    let id = created_id(router.clone(), "00001").await;
    let term_id = insert_term(&db, "2027년 1학기", "2027-03-02", "2027-06-30")
        .await
        .unwrap();
    let subject_id = insert_subject(&db, "요가", "요가", None).await.unwrap();
    let course_id = insert_course(&db, term_id, subject_id, None, None)
        .await
        .unwrap();
    let staff_id: i64 = sqlx::query_scalar("SELECT id FROM staff")
        .fetch_one(&db)
        .await
        .unwrap();
    insert_deletion(&db, id, course_id, staff_id).await.unwrap();

    let response = delete(router, &format!("/api/members/{id}")).await;

    assert_eq!(response.status(), StatusCode::CONFLICT);
    let body = json_body(response).await;
    assert_eq!(body["code"], "MEMBER_IN_USE");
    assert_eq!(body["rule"], "BR-M5");
    assert_eq!(count(&db).await, 1);
}
