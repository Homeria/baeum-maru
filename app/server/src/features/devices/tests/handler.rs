//! 기기 승인 요청 처리 테스트.

use axum::Router;
use axum::http::StatusCode;
use axum::http::{HeaderName, HeaderValue};
use serde_json::{Value, json};
use sqlx::SqlitePool;
use tower_http::set_header::SetRequestHeaderLayer;

use crate::features::access_logs::repository::list_for_test;
use crate::features::auth::middleware::DEVICE_HEADER;
use crate::features::devices::repository::{self};
use crate::server::test_support::{
    app_as, app_without_staff, delete, from_other_pc, get, json_body, post,
};

/// 요청마다 `name` 머리글을 `value`로 싣는다.
fn with_header(router: Router, name: &'static str, value: &str) -> Router {
    router.layer(SetRequestHeaderLayer::if_not_present(
        HeaderName::from_static(name),
        HeaderValue::from_str(value).unwrap(),
    ))
}

/// 처음 설정으로 관리자(PIN 123456)를 만들고, 호스트 PC에서 로그인한 라우터를 돌려준다.
async fn host_admin() -> (tempfile::TempDir, SqlitePool, Router, Router) {
    let (dir, db, router) = app_without_staff().await;
    let setup = json!({ "name": "관리자", "pin": "123456" }).to_string();
    post(router.clone(), "/api/setup", &setup).await;
    let list = json_body(get(router.clone(), "/api/login/staff").await).await;
    let login = json!({ "staffId": list["items"][0]["id"], "pin": "123456" }).to_string();
    let signed_in = json_body(post(router.clone(), "/api/login", &login).await).await;
    let token = format!("Bearer {}", signed_in["token"].as_str().unwrap());
    let admin = with_header(router.clone(), "authorization", &token);
    (dir, db, router, admin)
}

/// 다른 PC에서 승인을 요청하고 (기기 증표를 실은 그 PC의 라우터, 승인 번호)를 돌려준다.
async fn request_from_other_pc(router: Router) -> (Router, String) {
    let other = from_other_pc(router);
    let body = json!({ "name": "접수 노트북" }).to_string();
    let response = post(other.clone(), "/api/approval/request", &body).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = json_body(response).await;
    let token = body["token"].as_str().unwrap();
    let code = body["code"].as_str().unwrap().to_owned();
    (with_header(other, DEVICE_HEADER, token), code)
}

async fn status_of(other: Router) -> Value {
    json_body(get(other, "/api/approval/status").await).await["status"].clone()
}

async fn first_request_id(admin: Router) -> i64 {
    let list = json_body(get(admin, "/api/device-requests").await).await;
    list["items"][0]["id"].as_i64().unwrap()
}

#[tokio::test]
async fn br_k14_unapproved_pc_gets_only_version_and_approval() {
    let (_dir, _db, router, _admin) = host_admin().await;
    let other = from_other_pc(router);
    let wrong = with_header(other.clone(), DEVICE_HEADER, &"0".repeat(64));

    assert_eq!(
        get(other.clone(), "/api/version").await.status(),
        StatusCode::OK
    );
    for router in [other, wrong] {
        for uri in ["/api/login/staff", "/api/setup", "/api/members"] {
            let response = get(router.clone(), uri).await;
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{uri}");
            let body = json_body(response).await;
            assert_eq!(body["code"], "DEVICE_NOT_APPROVED", "{uri}");
            assert_eq!(body["rule"], "BR-K14");
        }
    }
}

#[tokio::test]
async fn br_k15_admin_approves_with_pin_on_host() {
    let (_dir, db, router, admin) = host_admin().await;
    let (other, code) = request_from_other_pc(router).await;
    assert_eq!(code.len(), 6);
    assert_eq!(status_of(other.clone()).await, "pending");

    let list = json_body(get(admin.clone(), "/api/device-requests").await).await;
    assert_eq!(list["items"][0]["code"], code.as_str());
    assert_eq!(list["items"][0]["name"], "접수 노트북");
    let id = list["items"][0]["id"].as_i64().unwrap();
    let uri = format!("/api/device-requests/{id}/approve");
    let response = post(admin.clone(), &uri, r#"{"pin":"123456"}"#).await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(status_of(other.clone()).await, "approved");
    assert_eq!(
        get(other, "/api/login/staff").await.status(),
        StatusCode::OK
    );
    let devices = json_body(get(admin, "/api/devices").await).await;
    assert_eq!(devices["items"][0]["name"], "접수 노트북");
    assert_eq!(devices["items"][0]["approvedBy"], "관리자");
    let events = list_for_test(&db).await;
    assert_eq!(events.last().unwrap().1, "device_approved");
}

#[tokio::test]
async fn wrong_pin_does_not_approve() {
    let (_dir, db, router, admin) = host_admin().await;
    let (other, _code) = request_from_other_pc(router).await;
    let id = first_request_id(admin.clone()).await;

    let uri = format!("/api/device-requests/{id}/approve");
    let response = post(admin, &uri, r#"{"pin":"000000"}"#).await;

    assert_eq!(json_body(response).await["code"], "WRONG_PIN");
    assert_eq!(status_of(other).await, "pending");
    let events = list_for_test(&db).await;
    assert_eq!(events.last().unwrap().1, "device_approval_failed");
}

#[tokio::test]
async fn rejected_pc_stays_blocked() {
    let (_dir, db, router, admin) = host_admin().await;
    let (other, _code) = request_from_other_pc(router).await;
    let id = first_request_id(admin.clone()).await;

    let uri = format!("/api/device-requests/{id}/reject");
    let response = post(admin.clone(), &uri, "{}").await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(status_of(other.clone()).await, "rejected");
    assert_eq!(
        get(other, "/api/login/staff").await.status(),
        StatusCode::UNAUTHORIZED
    );
    let again = post(
        admin,
        &format!("/api/device-requests/{id}/approve"),
        r#"{"pin":"123456"}"#,
    )
    .await;
    assert_eq!(json_body(again).await["code"], "APPROVAL_REQUEST_NOT_OPEN");
    let events = list_for_test(&db).await;
    assert_eq!(events.last().unwrap().1, "device_rejected");
}

#[tokio::test]
async fn br_k16_revoked_pc_is_blocked_from_next_request() {
    let (_dir, _db, router, admin) = host_admin().await;
    let (other, _code) = request_from_other_pc(router).await;
    let id = first_request_id(admin.clone()).await;
    let uri = format!("/api/device-requests/{id}/approve");
    post(admin.clone(), &uri, r#"{"pin":"123456"}"#).await;
    let devices = json_body(get(admin.clone(), "/api/devices").await).await;
    let device_id = devices["items"][0]["id"].as_i64().unwrap();

    let response = delete(admin.clone(), &format!("/api/devices/{device_id}")).await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    let blocked = json_body(get(other, "/api/login/staff").await).await;
    assert_eq!(blocked["code"], "DEVICE_NOT_APPROVED");
    let again = delete(admin, &format!("/api/devices/{device_id}")).await;
    assert_eq!(again.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn approving_is_host_only() {
    let (_dir, db, router, admin) = host_admin().await;
    let _requested = request_from_other_pc(router).await;
    let id = first_request_id(admin.clone()).await;
    // 승인받은 다른 PC에 같은 관리자 세션으로 앉았다
    let elsewhere = crate::server::test_support::from_approved_pc(&db, admin).await;

    for response in [
        get(elsewhere.clone(), "/api/device-requests").await,
        post(
            elsewhere.clone(),
            &format!("/api/device-requests/{id}/approve"),
            r#"{"pin":"123456"}"#,
        )
        .await,
        post(
            elsewhere.clone(),
            &format!("/api/device-requests/{id}/reject"),
            "{}",
        )
        .await,
    ] {
        assert_eq!(json_body(response).await["code"], "HOST_ONLY");
    }
    // 목록 보기와 승인 취소는 어느 PC에서나
    assert_eq!(
        get(elsewhere, "/api/devices").await.status(),
        StatusCode::OK
    );
}

#[tokio::test]
async fn only_admin_manages_devices() {
    for role in ["staff", "temp_staff", "read_only"] {
        let (_dir, _db, router) = app_as(role).await;

        for response in [
            get(router.clone(), "/api/devices").await,
            get(router.clone(), "/api/device-requests").await,
            delete(router.clone(), "/api/devices/1").await,
        ] {
            assert_eq!(json_body(response).await["code"], "NOT_ALLOWED", "{role}");
        }
    }
}

#[tokio::test]
async fn device_name_is_checked() {
    let (_dir, _db, router, _admin) = host_admin().await;
    let other = from_other_pc(router);

    let response = post(other, "/api/approval/request", r#"{"name":"  "}"#).await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = json_body(response).await;
    assert_eq!(body["fields"][0]["field"], "name");
}

#[tokio::test]
async fn too_many_open_requests_are_refused() {
    let (_dir, _db, router, _admin) = host_admin().await;
    for _ in 0..repository::MAX_OPEN_REQUESTS {
        let _requested = request_from_other_pc(router.clone()).await;
    }
    let other = from_other_pc(router);

    let response = post(other, "/api/approval/request", r#"{"name":"하나 더"}"#).await;

    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn status_needs_a_requested_token() {
    let (_dir, _db, router, _admin) = host_admin().await;
    let other = from_other_pc(router);

    for router in [
        other.clone(),
        with_header(other, DEVICE_HEADER, &"0".repeat(64)),
    ] {
        let body = json_body(get(router, "/api/approval/status").await).await;
        assert_eq!(body["code"], "APPROVAL_REQUEST_NOT_OPEN");
    }
}
