//! 과목 요청 처리 테스트.

use axum::body::Body;
use axum::http::Request;
use axum::http::{StatusCode, header};
use serde_json::json;
use tower::ServiceExt;

use crate::server::test_support::{
    app, create, create_course, create_subject, create_term, delete, get, json_body, patch, post,
};

fn subject(name: &str, category: Option<&str>) -> String {
    json!({ "name": name, "category": category }).to_string()
}

#[tokio::test]
async fn creates_and_lists_subject() {
    let (_dir, _db, router) = app().await;

    let response = post(
        router.clone(),
        "/api/subjects",
        &subject("한글 교실", Some("평생교육")),
    )
    .await;

    assert_eq!(response.status(), StatusCode::CREATED);
    let location = response.headers()[header::LOCATION]
        .to_str()
        .unwrap()
        .to_owned();
    let id = json_body(response).await["id"].clone();
    assert_eq!(location, format!("/api/subjects/{id}"));
    let list = json_body(get(router, "/api/subjects").await).await;
    assert_eq!(
        list,
        json!({ "items": [
            { "id": id, "name": "한글 교실", "category": "평생교육" }
        ]})
    );
}

#[tokio::test]
async fn invalid_input_returns_every_field() {
    let (_dir, _db, router) = app().await;
    let long = "가".repeat(21);

    let response = post(router, "/api/subjects", &subject(" ", Some(&long))).await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = json_body(response).await;
    let fields: Vec<&str> = body["fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|field| field["field"].as_str().unwrap())
        .collect();
    assert_eq!(fields, ["name", "category"]);
}

#[tokio::test]
async fn br_j1_name_differing_only_in_spaces_is_conflict() {
    let (_dir, _db, router) = app().await;
    post(router.clone(), "/api/subjects", &subject("한글교실", None)).await;

    let response = post(router, "/api/subjects", &subject("한글 교실", None)).await;

    assert_eq!(response.status(), StatusCode::CONFLICT);
    let body = json_body(response).await;
    assert_eq!(body["code"], "DUPLICATE_SUBJECT_NAME");
    assert_eq!(body["rule"], "BR-J1");
}

#[tokio::test]
async fn category_can_be_left_out() {
    let (_dir, _db, router) = app().await;

    let response = post(router.clone(), "/api/subjects", r#"{ "name": "요가" }"#).await;

    assert_eq!(response.status(), StatusCode::CREATED);
    let list = json_body(get(router, "/api/subjects").await).await;
    assert_eq!(list["items"][0]["category"], json!(null));
}

#[tokio::test]
async fn gets_one_subject() {
    let (_dir, _db, router) = app().await;
    let id = create(
        router.clone(),
        "/api/subjects",
        &subject("요가", Some("취미여가")),
    )
    .await;

    let response = get(router, &format!("/api/subjects/{id}")).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        json_body(response).await,
        json!({ "id": id, "name": "요가", "category": "취미여가" })
    );
}

#[tokio::test]
async fn unknown_subject_is_not_found() {
    let (_dir, _db, router) = app().await;

    let response = get(router, "/api/subjects/999").await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(json_body(response).await["code"], "SUBJECT_NOT_FOUND");
}

#[tokio::test]
async fn non_number_id_is_malformed() {
    let (_dir, _db, router) = app().await;

    let response = get(router, "/api/subjects/abc").await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn updates_subject() {
    let (_dir, _db, router) = app().await;
    let id = create(
        router.clone(),
        "/api/subjects",
        &subject("요가", Some("취미여가")),
    )
    .await;

    let uri = format!("/api/subjects/{id}");
    let response = patch(router, &uri, &subject("필라테스", None)).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        json_body(response).await,
        json!({ "id": id, "name": "필라테스", "category": null })
    );
}

#[tokio::test]
async fn br_j1_update_to_taken_name_is_conflict() {
    let (_dir, _db, router) = app().await;
    create(router.clone(), "/api/subjects", &subject("한글교실", None)).await;
    let id = create(router.clone(), "/api/subjects", &subject("요가", None)).await;

    let uri = format!("/api/subjects/{id}");
    let response = patch(router, &uri, &subject("한글 교실", None)).await;

    assert_eq!(response.status(), StatusCode::CONFLICT);
    assert_eq!(json_body(response).await["rule"], "BR-J1");
}

#[tokio::test]
async fn deletes_subject() {
    let (_dir, _db, router) = app().await;
    let id = create(router.clone(), "/api/subjects", &subject("요가", None)).await;
    let uri = format!("/api/subjects/{id}");

    let response = delete(router.clone(), &uri).await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(delete(router, &uri).await.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn subjects_need_matching_version() {
    let (_dir, _db, router) = app().await;
    let request = Request::get("/api/subjects").body(Body::empty()).unwrap();

    let response = router.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn br_j2_subject_used_by_course_is_conflict() {
    let (_dir, _db, router) = app().await;
    let subject_id = create_subject(router.clone(), "요가").await;
    let term_id = create_term(router.clone()).await;
    create_course(router.clone(), term_id, subject_id, None).await;

    let response = delete(router, &format!("/api/subjects/{subject_id}")).await;

    assert_eq!(response.status(), StatusCode::CONFLICT);
    let body = json_body(response).await;
    assert_eq!(body["code"], "SUBJECT_IN_USE");
    assert_eq!(body["rule"], "BR-J2");
}
