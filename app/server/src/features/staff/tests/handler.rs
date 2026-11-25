//! 직원 요청 처리 테스트.

use axum::Router;
use axum::http::StatusCode;
use serde_json::{Value, json};

use crate::server::test_support::{app, app_as, create, get, json_body, patch, post};

fn new_staff(name: &str, role: &str, valid_until: Option<&str>, pin: &str) -> String {
    json!({ "name": name, "role": role, "validUntil": valid_until, "pin": pin }).to_string()
}

fn update(name: &str, role: &str, valid_until: Option<&str>, disabled: bool) -> String {
    json!({ "name": name, "role": role, "validUntil": valid_until, "disabled": disabled })
        .to_string()
}

async fn list(router: Router) -> Value {
    json_body(get(router, "/api/staff").await).await["items"].clone()
}

/// `app()`이 로그인해 둔 관리자의 id.
async fn admin_id(router: Router) -> i64 {
    list(router).await[0]["id"].as_i64().unwrap()
}

#[tokio::test]
async fn only_admin_manages_staff() {
    for role in ["staff", "temp_staff", "read_only"] {
        let (_dir, _db, router) = app_as(role).await;

        let responses = [
            get(router.clone(), "/api/staff").await,
            get(router.clone(), "/api/staff/1").await,
            post(router.clone(), "/api/staff", "{}").await,
            patch(router.clone(), "/api/staff/1", "{}").await,
            post(router.clone(), "/api/staff/1/pin", "{}").await,
        ];

        for response in responses {
            assert_eq!(response.status(), StatusCode::FORBIDDEN, "{role}");
            assert_eq!(json_body(response).await["code"], "NOT_ALLOWED");
        }
    }
}

#[tokio::test]
async fn creates_and_lists_staff() {
    let (_dir, _db, router) = app().await;

    let id = create(
        router.clone(),
        "/api/staff",
        &new_staff("가 임시", "temp_staff", Some("2027-02-28"), "1234"),
    )
    .await;

    let items = list(router.clone()).await;
    assert_eq!(
        items[0],
        json!({
            "id": id,
            "name": "가 임시",
            "role": "temp_staff",
            "validUntil": "2027-02-28",
            "disabled": false,
            "locked": false,
        })
    );
    assert_eq!(items[1]["name"], "시험직원");
    let response = get(router, &format!("/api/staff/{id}")).await;
    assert_eq!(json_body(response).await["name"], "가 임시");
}

#[tokio::test]
async fn rejects_invalid_new_staff() {
    let (_dir, _db, router) = app().await;

    let response = post(
        router.clone(),
        "/api/staff",
        &new_staff("", "temp_staff", None, "12"),
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = json_body(response).await;
    let fields: Vec<&str> = body["fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|field| field["field"].as_str().unwrap())
        .collect();
    assert_eq!(fields, ["name", "pin", "validUntil"]);
    assert!(!body.to_string().contains("\"12\""));

    let unknown_role = new_staff("직원", "boss", None, "1234");
    let response = post(router, "/api/staff", &unknown_role).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rejects_duplicate_name() {
    let (_dir, _db, router) = app().await;

    let response = post(
        router,
        "/api/staff",
        &new_staff("시험 직원", "staff", None, "1234"),
    )
    .await;

    assert_eq!(response.status(), StatusCode::CONFLICT);
    let body = json_body(response).await;
    assert_eq!(body["code"], "DUPLICATE_STAFF_NAME");
    assert_eq!(body["rule"], "BR-K2");
}

#[tokio::test]
async fn updates_role_and_disables_staff() {
    let (_dir, _db, router) = app().await;
    let id = create(
        router.clone(),
        "/api/staff",
        &new_staff("직원1", "staff", None, "1234"),
    )
    .await;

    let response = patch(
        router,
        &format!("/api/staff/{id}"),
        &update("직원 1", "read_only", None, true),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["name"], "직원 1");
    assert_eq!(body["role"], "read_only");
    assert_eq!(body["disabled"], true);
}

#[tokio::test]
async fn last_admin_stays_admin() {
    let (_dir, _db, router) = app().await;
    let admin_id = admin_id(router.clone()).await;
    let uri = format!("/api/staff/{admin_id}");

    for body in [
        update("시험직원", "staff", None, false),
        update("시험직원", "admin", None, true),
    ] {
        let response = patch(router.clone(), &uri, &body).await;

        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = json_body(response).await;
        assert_eq!(body["code"], "LAST_ADMIN");
        assert_eq!(body["rule"], "BR-K9");
    }
}

#[tokio::test]
async fn missing_staff_is_not_found() {
    let (_dir, _db, router) = app().await;

    let responses = [
        get(router.clone(), "/api/staff/999").await,
        patch(
            router.clone(),
            "/api/staff/999",
            &update("없음", "staff", None, false),
        )
        .await,
        post(router, "/api/staff/999/pin", r#"{"pin":"1234"}"#).await,
    ];

    for response in responses {
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(json_body(response).await["code"], "STAFF_NOT_FOUND");
    }
}

#[tokio::test]
async fn new_pin_works_for_login_and_unlocks_account() {
    let (_dir, db, router) = app().await;
    let id = create(
        router.clone(),
        "/api/staff",
        &new_staff("직원1", "staff", None, "1234"),
    )
    .await;
    sqlx::query("UPDATE staff SET locked_until = '2999-01-01T00:00:00.000Z' WHERE id = ?")
        .bind(id)
        .execute(&db)
        .await
        .unwrap();
    let staff_uri = format!("/api/staff/{id}");
    assert_eq!(
        json_body(get(router.clone(), &staff_uri).await).await["locked"],
        true
    );

    let response = post(
        router.clone(),
        &format!("{staff_uri}/pin"),
        r#"{"pin":"5678"}"#,
    )
    .await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        json_body(get(router.clone(), &staff_uri).await).await["locked"],
        false
    );
    let login = json!({ "staffId": id, "pin": "5678" }).to_string();
    let response = post(router, "/api/login", &login).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn admin_pin_needs_six_digits() {
    let (_dir, _db, router) = app().await;
    let admin_id = admin_id(router.clone()).await;

    let response = post(
        router,
        &format!("/api/staff/{admin_id}/pin"),
        r#"{"pin":"1234"}"#,
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = json_body(response).await;
    assert_eq!(body["fields"][0]["field"], "pin");
    assert_eq!(
        body["fields"][0]["message"],
        "관리자 PIN은 6자리 이상입니다"
    );
}
