//! 학기 요청 처리 테스트.

use axum::http::StatusCode;
use serde_json::json;

use crate::server::test_support::{
    app, create, create_course, create_subject, delete, get, json_body, patch, post,
};

fn term(name: &str, starts_on: Option<&str>, ends_on: Option<&str>) -> String {
    json!({ "name": name, "startsOn": starts_on, "endsOn": ends_on }).to_string()
}

fn spring() -> String {
    term("2027년 1학기", Some("2027-03-02"), Some("2027-06-30"))
}

#[tokio::test]
async fn creates_and_lists_term() {
    let (_dir, _db, router) = app().await;

    let response = post(router.clone(), "/api/terms", &spring()).await;

    assert_eq!(response.status(), StatusCode::CREATED);
    let id = json_body(response).await["id"].clone();
    let list = json_body(get(router, "/api/terms").await).await;
    assert_eq!(
        list,
        json!({ "items": [{
            "id": id,
            "name": "2027년 1학기",
            "startsOn": "2027-03-02",
            "endsOn": "2027-06-30"
        }]})
    );
}

#[tokio::test]
async fn empty_fields_are_field_errors() {
    let (_dir, _db, router) = app().await;

    let response = post(router, "/api/terms", &term("", None, None)).await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = json_body(response).await;
    let fields: Vec<&str> = body["fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|field| field["field"].as_str().unwrap())
        .collect();
    assert_eq!(fields, ["name", "startsOn", "endsOn"]);
}

#[tokio::test]
async fn br_t2_end_before_start_is_field_error() {
    let (_dir, _db, router) = app().await;

    let body = term("거꾸로", Some("2027-06-30"), Some("2027-03-02"));
    let response = post(router, "/api/terms", &body).await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(json_body(response).await["fields"][0]["field"], "endsOn");
}

#[tokio::test]
async fn wrong_date_shape_is_malformed() {
    let (_dir, _db, router) = app().await;

    let body = term("2027년 1학기", Some("2027/03/02"), Some("2027-06-30"));
    let response = post(router, "/api/terms", &body).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn br_t1_duplicate_name_is_conflict() {
    let (_dir, _db, router) = app().await;
    post(router.clone(), "/api/terms", &spring()).await;

    let response = post(router, "/api/terms", &spring()).await;

    assert_eq!(response.status(), StatusCode::CONFLICT);
    let body = json_body(response).await;
    assert_eq!(body["code"], "DUPLICATE_TERM_NAME");
    assert_eq!(body["rule"], "BR-T1");
}

#[tokio::test]
async fn gets_and_updates_term() {
    let (_dir, _db, router) = app().await;
    let id = create(router.clone(), "/api/terms", &spring()).await;
    let uri = format!("/api/terms/{id}");

    let changed = term("2027년 봄학기", Some("2027-03-04"), Some("2027-06-27"));
    let response = patch(router.clone(), &uri, &changed).await;

    assert_eq!(response.status(), StatusCode::OK);
    let read = json_body(get(router, &uri).await).await;
    assert_eq!(read["name"], "2027년 봄학기");
    assert_eq!(read["startsOn"], "2027-03-04");
}

#[tokio::test]
async fn unknown_term_is_not_found() {
    let (_dir, _db, router) = app().await;

    let response = get(router.clone(), "/api/terms/999").await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(json_body(response).await["code"], "TERM_NOT_FOUND");
    let response = patch(router, "/api/terms/999", &spring()).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn deletes_term_without_course() {
    let (_dir, _db, router) = app().await;
    let id = create(router.clone(), "/api/terms", &spring()).await;
    let uri = format!("/api/terms/{id}");

    assert_eq!(
        delete(router.clone(), &uri).await.status(),
        StatusCode::NO_CONTENT
    );
    assert_eq!(delete(router, &uri).await.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn br_t5_term_with_course_is_conflict() {
    let (_dir, _db, router) = app().await;
    let id = create(router.clone(), "/api/terms", &spring()).await;
    let subject_id = create_subject(router.clone(), "요가").await;
    create_course(router.clone(), id, subject_id, None).await;

    let response = delete(router, &format!("/api/terms/{id}")).await;

    assert_eq!(response.status(), StatusCode::CONFLICT);
    let body = json_body(response).await;
    assert_eq!(body["code"], "TERM_IN_USE");
    assert_eq!(body["rule"], "BR-T5");
}
