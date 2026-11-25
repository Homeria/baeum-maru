//! 기기 확인, 로그인 확인, 역할 확인 테스트.

use axum::Router;
use axum::body::Body;
use axum::http::header;
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use sqlx::SqlitePool;
use time::OffsetDateTime;
use tower::ServiceExt;

use crate::features::auth::middleware::*;
use crate::http::version::VERSION_HEADER;
use crate::server::test_support::{
    app, app_as, app_without_staff, delete, get, json_body, patch, post,
};

/// 로그인이 필요한 주소마다 하나씩. id는 없는 값이어도 된다 (로그인 확인이 먼저다).
const SIGNED_IN_ROUTES: [(&str, &str); 18] = [
    ("GET", "/api/members"),
    ("POST", "/api/members"),
    ("GET", "/api/members/1"),
    ("PATCH", "/api/members/1"),
    ("DELETE", "/api/members/1"),
    ("GET", "/api/rooms"),
    ("POST", "/api/rooms"),
    ("DELETE", "/api/rooms/1"),
    ("GET", "/api/subjects"),
    ("PATCH", "/api/subjects/1"),
    ("GET", "/api/terms"),
    ("POST", "/api/terms"),
    ("GET", "/api/terms/1/courses"),
    ("POST", "/api/terms/1/courses"),
    ("GET", "/api/courses/1"),
    ("GET", "/api/staff"),
    ("POST", "/api/staff/1/pin"),
    ("GET", "/api/access-logs"),
];

async fn send(router: Router, method: &str, uri: &str, token: Option<&str>) -> Value {
    let mut request = Request::builder()
        .method(method)
        .uri(uri)
        .header(VERSION_HEADER, crate::infra::version::CURRENT)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(token) = token {
        request = request.header(header::AUTHORIZATION, token);
    }
    let response = router
        .oneshot(request.body(Body::from("{}")).unwrap())
        .await
        .unwrap();
    let status = response.status().as_u16();
    json!({ "status": status, "code": json_body(response).await["code"] })
}

#[tokio::test]
async fn every_data_route_requires_login() {
    let (_dir, _db, router) = app_without_staff().await;
    let wrong_token = format!("Bearer {}", "0".repeat(64));

    for (method, uri) in SIGNED_IN_ROUTES {
        for token in [None, Some("Bearer 틀린모양"), Some(wrong_token.as_str())] {
            let result = send(router.clone(), method, uri, token).await;

            assert_eq!(
                result,
                json!({ "status": 401, "code": "LOGIN_REQUIRED" }),
                "{method} {uri} {token:?}"
            );
        }
    }
}

async fn set_last_seen(db: &SqlitePool, minutes_ago: i64) {
    let last_seen = OffsetDateTime::now_utc() - time::Duration::minutes(minutes_ago);
    sqlx::query("UPDATE sessions SET last_seen_at = ?")
        .bind(crate::infra::db::timestamp_text(last_seen))
        .execute(db)
        .await
        .unwrap();
}

async fn last_seen(db: &SqlitePool) -> String {
    sqlx::query_scalar("SELECT last_seen_at FROM sessions")
        .fetch_one(db)
        .await
        .unwrap()
}

#[tokio::test]
async fn request_keeps_session_open() {
    let (_dir, db, router) = app().await;
    set_last_seen(&db, 9).await;
    let before = last_seen(&db).await;

    let response = get(router, "/api/rooms").await;

    assert_eq!(response.status(), StatusCode::OK);
    assert!(last_seen(&db).await > before);
}

#[tokio::test]
async fn br_k8_quiet_request_does_not_keep_session_open() {
    let (_dir, db, router) = app().await;
    set_last_seen(&db, 9).await;
    let before = last_seen(&db).await;
    let request = Request::get("/api/rooms")
        .header(VERSION_HEADER, crate::infra::version::CURRENT)
        .header(QUIET_HEADER, "1")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(last_seen(&db).await, before);
}

#[tokio::test]
async fn session_locks_after_ten_quiet_minutes() {
    let (_dir, db, router) = app().await;
    set_last_seen(&db, 10).await;
    let before = last_seen(&db).await;

    let response = get(router, "/api/rooms").await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = json_body(response).await;
    assert_eq!(body["code"], "SESSION_LOCKED");
    assert_eq!(body["rule"], "BR-K8");
    // 잠긴 요청은 시각을 늘리지 않는다
    assert_eq!(last_seen(&db).await, before);
}

#[tokio::test]
async fn account_changes_apply_from_next_request() {
    let (_dir, db, router) = app().await;
    let room = json!({ "name": "201호", "building": null, "floor": null }).to_string();

    sqlx::query("UPDATE staff SET role = 'read_only'")
        .execute(&db)
        .await
        .unwrap();
    let response = post(router.clone(), "/api/rooms", &room).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(json_body(response).await["code"], "NOT_ALLOWED");

    sqlx::query("UPDATE staff SET disabled = 1")
        .execute(&db)
        .await
        .unwrap();
    let response = get(router, "/api/rooms").await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(json_body(response).await["code"], "ACCOUNT_UNAVAILABLE");
}

#[tokio::test]
async fn expired_account_is_blocked_from_next_request() {
    let (_dir, db, router) = app_as("temp_staff").await;
    sqlx::query("UPDATE staff SET valid_until = '2020-12-31'")
        .execute(&db)
        .await
        .unwrap();

    let response = get(router, "/api/members").await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(json_body(response).await["code"], "ACCOUNT_UNAVAILABLE");
}

/// 없는 id에 빈 본문으로 바꾸기를 보내 본다. 역할 확인을 지나면 본문이나 대상에서
/// 걸려 400·404가 되고, 역할이 허락하지 않으면 403이다.
async fn allowed(router: Router, method: &str, uri: &str) -> bool {
    let body = "{}";
    let response = match method {
        "PATCH" => patch(router, uri, body).await,
        "DELETE" => delete(router, uri).await,
        _ => unreachable!("{method}"),
    };
    match response.status() {
        StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND => true,
        StatusCode::FORBIDDEN => false,
        other => panic!("{method} {uri}: {other}"),
    }
}

#[tokio::test]
async fn roles_follow_the_role_table() {
    // (역할, 회원 고치기, 회원 지우기, 강좌 바꾸기, 기초 정보 바꾸기)
    let table = [
        ("admin", true, true, true, true),
        ("staff", true, true, true, true),
        ("temp_staff", true, false, false, false),
        ("read_only", false, false, false, false),
    ];
    for (role, edit_members, delete_members, edit_courses, edit_reference) in table {
        let (_dir, _db, router) = app_as(role).await;

        assert_eq!(
            allowed(router.clone(), "PATCH", "/api/members/999").await,
            edit_members,
            "{role}"
        );
        assert_eq!(
            allowed(router.clone(), "DELETE", "/api/members/999").await,
            delete_members,
            "{role}"
        );
        assert_eq!(
            allowed(router.clone(), "DELETE", "/api/courses/999").await,
            edit_courses,
            "{role}"
        );
        assert_eq!(
            allowed(router.clone(), "DELETE", "/api/terms/999").await,
            edit_courses,
            "{role}"
        );
        assert_eq!(
            allowed(router.clone(), "DELETE", "/api/subjects/999").await,
            edit_reference,
            "{role}"
        );
        assert_eq!(
            allowed(router.clone(), "DELETE", "/api/rooms/999").await,
            edit_reference,
            "{role}"
        );
        // 모두 보기는 모든 역할
        let response = get(router, "/api/members").await;
        assert_eq!(response.status(), StatusCode::OK, "{role}");
    }
}
