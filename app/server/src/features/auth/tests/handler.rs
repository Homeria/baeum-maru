//! 로그인, 잠금 풀기, 로그아웃, 복구 요청 처리 테스트.

use axum::Router;
use axum::body::Body;
use axum::http::{Request, Response, StatusCode, header};
use baeum_domain::RecoveryCode;
use serde_json::{Value, json};
use sqlx::SqlitePool;
use time::OffsetDateTime;
use tower::ServiceExt;

use crate::db::test_support::insert_staff;
use crate::features::access_logs::repository::list_for_test;
use crate::features::auth::handler::Watch;
use crate::features::auth::service as sessions;
use crate::http::version::VERSION_HEADER;
use crate::infra::secret::SessionToken;
use crate::server::test_support::{app, app_without_staff, get, json_body, post};

/// 처음 설정으로 관리자(PIN 123456)를 만들고 id를 돌려준다.
async fn setup_admin(router: Router) -> i64 {
    let body = json!({ "name": "관리자", "pin": "123456" }).to_string();
    let response = post(router.clone(), "/api/setup", &body).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let list = json_body(get(router, "/api/login/staff").await).await;
    list["items"][0]["id"].as_i64().unwrap()
}

async fn login(router: Router, staff_id: i64, pin: &str) -> Response<Body> {
    let body = json!({ "staffId": staff_id, "pin": pin }).to_string();
    post(router, "/api/login", &body).await
}

/// `token`(16진수)을 실어 보낸다. `None`이면 증표 없이.
async fn send_as(
    router: Router,
    method: &str,
    uri: &str,
    token: Option<&str>,
    body: &str,
) -> Response<Body> {
    let mut request = Request::builder()
        .method(method)
        .uri(uri)
        .header(VERSION_HEADER, crate::infra::version::CURRENT)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(token) = token {
        request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    router
        .oneshot(request.body(Body::from(body.to_owned())).unwrap())
        .await
        .unwrap()
}

async fn logout(router: Router, token: Option<&str>) -> Response<Body> {
    send_as(router, "POST", "/api/logout", token, "").await
}

async fn unlock(router: Router, token: Option<&str>, pin: &str) -> Response<Body> {
    let body = json!({ "pin": pin }).to_string();
    send_as(router, "POST", "/api/unlock", token, &body).await
}

/// 관리자로 로그인하고 증표를 돌려준 뒤 세션을 10분 전 것으로 만들어 잠근다.
async fn locked_admin_session(router: Router, db: &SqlitePool) -> String {
    let admin_id = setup_admin(router.clone()).await;
    let body = json_body(login(router, admin_id, "123456").await).await;
    let old = OffsetDateTime::now_utc() - time::Duration::minutes(10);
    sqlx::query("UPDATE sessions SET last_seen_at = ?")
        .bind(crate::infra::db::timestamp_text(old))
        .execute(db)
        .await
        .unwrap();
    body["token"].as_str().unwrap().to_owned()
}

async fn error_code(response: Response<Body>) -> String {
    json_body(response).await["code"]
        .as_str()
        .unwrap()
        .to_owned()
}

async fn session_count(db: &SqlitePool) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM sessions")
        .fetch_one(db)
        .await
        .unwrap()
}

#[tokio::test]
async fn lists_staff_to_choose() {
    let (_dir, db, router) = app_without_staff().await;
    let admin_id = setup_admin(router.clone()).await;
    let disabled = insert_staff(&db, "그만둔직원", "staff", None)
        .await
        .unwrap();
    sqlx::query("UPDATE staff SET disabled = 1 WHERE id = ?")
        .bind(disabled)
        .execute(&db)
        .await
        .unwrap();

    let response = get(router, "/api/login/staff").await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        json_body(response).await,
        json!({ "items": [{ "id": admin_id, "name": "관리자", "role": "admin" }] })
    );
}

#[tokio::test]
async fn login_gives_token_and_staff() {
    let (_dir, db, router) = app_without_staff().await;
    let admin_id = setup_admin(router.clone()).await;

    let response = login(router, admin_id, "123456").await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(
        body["staff"],
        json!({ "id": admin_id, "name": "관리자", "role": "admin" })
    );
    // DB에는 증표가 아니라 그 해시가 있다
    let token = SessionToken::parse(body["token"].as_str().unwrap()).unwrap();
    let stored: Vec<u8> = sqlx::query_scalar("SELECT token_hash FROM sessions")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(stored, token.hash());
    assert_eq!(
        list_for_test(&db).await,
        [(Some(admin_id), "login".to_owned())]
    );
}

#[tokio::test]
async fn wrong_pin_fails_and_fifth_locks_even_the_right_pin() {
    let (_dir, db, router) = app_without_staff().await;
    let admin_id = setup_admin(router.clone()).await;

    for _ in 0..4 {
        let response = login(router.clone(), admin_id, "000000").await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let body = json_body(response).await;
        assert_eq!(body["code"], "WRONG_PIN");
        assert!(!body.to_string().contains("000000"));
    }
    let fifth = login(router.clone(), admin_id, "000000").await;
    assert_eq!(fifth.status(), StatusCode::FORBIDDEN);
    assert_eq!(error_code(fifth).await, "ACCOUNT_LOCKED");

    let right = login(router, admin_id, "123456").await;

    assert_eq!(right.status(), StatusCode::FORBIDDEN);
    assert_eq!(error_code(right).await, "ACCOUNT_LOCKED");
    assert_eq!(session_count(&db).await, 0);
    let logs = list_for_test(&db).await;
    assert_eq!(logs.len(), 6);
    assert!(
        logs.iter()
            .all(|log| *log == (Some(admin_id), "login_failed".to_owned()))
    );
}

#[tokio::test]
async fn right_pin_resets_failed_count() {
    let (_dir, db, router) = app_without_staff().await;
    let admin_id = setup_admin(router.clone()).await;
    for _ in 0..4 {
        login(router.clone(), admin_id, "000000").await;
    }

    let response = login(router, admin_id, "123456").await;

    assert_eq!(response.status(), StatusCode::OK);
    let count: i64 = sqlx::query_scalar("SELECT failed_pin_count FROM staff")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn disabled_or_expired_account_cannot_log_in() {
    let (_dir, db, router) = app_without_staff().await;
    let admin_id = setup_admin(router.clone()).await;
    let pin_hash: String = sqlx::query_scalar("SELECT pin_hash FROM staff")
        .fetch_one(&db)
        .await
        .unwrap();
    let expired = insert_staff(&db, "임시1", "temp_staff", Some("2020-12-31"))
        .await
        .unwrap();
    sqlx::query("UPDATE staff SET pin_hash = ? WHERE id = ?")
        .bind(&pin_hash)
        .bind(expired)
        .execute(&db)
        .await
        .unwrap();
    sqlx::query("UPDATE staff SET disabled = 1 WHERE id = ?")
        .bind(admin_id)
        .execute(&db)
        .await
        .unwrap();

    for staff_id in [admin_id, expired] {
        let response = login(router.clone(), staff_id, "123456").await;

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        assert_eq!(error_code(response).await, "ACCOUNT_UNAVAILABLE");
    }
    assert_eq!(session_count(&db).await, 0);
    assert_eq!(list_for_test(&db).await.len(), 2);
}

#[tokio::test]
async fn unknown_staff_is_logged_without_staff() {
    let (_dir, db, router) = app_without_staff().await;

    let response = login(router, 999, "123456").await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(error_code(response).await, "STAFF_NOT_FOUND");
    assert_eq!(
        list_for_test(&db).await,
        [(None, "login_failed".to_owned())]
    );
}

#[tokio::test]
async fn logout_ends_session_once() {
    let (_dir, db, router) = app_without_staff().await;
    let admin_id = setup_admin(router.clone()).await;
    let body = json_body(login(router.clone(), admin_id, "123456").await).await;
    let token = body["token"].as_str().unwrap();

    let response = logout(router.clone(), Some(token)).await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(session_count(&db).await, 0);
    // 이미 끝난 세션, 증표 없음, 모양이 틀린 증표도 204이고 기록은 늘지 않는다
    for token in [Some(token), None, Some("틀린증표")] {
        let response = logout(router.clone(), token).await;
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }
    assert_eq!(
        list_for_test(&db).await,
        [
            (Some(admin_id), "login".to_owned()),
            (Some(admin_id), "logout".to_owned()),
        ]
    );
}

#[tokio::test]
async fn right_pin_unlocks_locked_session() {
    let (_dir, db, router) = app_without_staff().await;
    let token = locked_admin_session(router.clone(), &db).await;
    let locked = send_as(router.clone(), "GET", "/api/rooms", Some(&token), "").await;
    assert_eq!(error_code(locked).await, "SESSION_LOCKED");

    let wrong = unlock(router.clone(), Some(&token), "000000").await;
    assert_eq!(wrong.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(error_code(wrong).await, "WRONG_PIN");
    let right = unlock(router.clone(), Some(&token), "123456").await;
    assert_eq!(right.status(), StatusCode::NO_CONTENT);

    let response = send_as(router, "GET", "/api/rooms", Some(&token), "").await;
    assert_eq!(response.status(), StatusCode::OK);
    let events: Vec<String> = list_for_test(&db)
        .await
        .into_iter()
        .map(|(_, event)| event)
        .collect();
    assert_eq!(events, ["login", "unlock_failed"]);
}

#[tokio::test]
async fn fifth_wrong_unlock_locks_account() {
    let (_dir, db, router) = app_without_staff().await;
    let token = locked_admin_session(router.clone(), &db).await;

    for _ in 0..4 {
        let response = unlock(router.clone(), Some(&token), "000000").await;
        assert_eq!(error_code(response).await, "WRONG_PIN");
    }
    let fifth = unlock(router.clone(), Some(&token), "000000").await;
    assert_eq!(fifth.status(), StatusCode::FORBIDDEN);
    assert_eq!(error_code(fifth).await, "ACCOUNT_LOCKED");

    let right = unlock(router, Some(&token), "123456").await;
    assert_eq!(error_code(right).await, "ACCOUNT_LOCKED");
}

#[tokio::test]
async fn unlock_needs_a_session() {
    let (_dir, _db, router) = app_without_staff().await;

    for token in [None, Some("틀린증표")] {
        let response = unlock(router.clone(), token, "123456").await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(error_code(response).await, "LOGIN_REQUIRED");
    }
}

/// 처음 설정으로 관리자(PIN 123456)를 만들고 (관리자 id, 복구 코드)를 돌려준다.
async fn setup_admin_with_code(router: Router) -> (i64, String) {
    let body = json!({ "name": "관리자", "pin": "123456" }).to_string();
    let response = post(router.clone(), "/api/setup", &body).await;
    let code = json_body(response).await["recoveryCode"]
        .as_str()
        .unwrap()
        .to_owned();
    let list = json_body(get(router, "/api/login/staff").await).await;
    (list["items"][0]["id"].as_i64().unwrap(), code)
}

async fn recover(router: Router, staff_id: i64, code: &str, pin: &str) -> (StatusCode, Value) {
    let body = json!({ "staffId": staff_id, "code": code, "pin": pin }).to_string();
    let response = post(router, "/api/recovery", &body).await;
    (response.status(), json_body(response).await)
}

async fn login_status(router: Router, staff_id: i64, pin: &str) -> StatusCode {
    let body = json!({ "staffId": staff_id, "pin": pin }).to_string();
    post(router, "/api/login", &body).await.status()
}

#[tokio::test]
async fn code_resets_admin_pin_once_and_gives_new_code() {
    let (_dir, db, router) = app_without_staff().await;
    let (admin_id, code) = setup_admin_with_code(router.clone()).await;

    // 소문자, 끊는 줄 없이 넣어도 된다
    let typed = code.replace('-', "").to_lowercase();
    let (status, body) = recover(router.clone(), admin_id, &typed, "654321").await;

    assert_eq!(status, StatusCode::OK);
    let new_code = body["recoveryCode"].as_str().unwrap();
    assert!(RecoveryCode::parse(new_code).is_some());
    assert_ne!(new_code, code);
    assert_eq!(
        login_status(router.clone(), admin_id, "654321").await,
        StatusCode::OK
    );
    assert_eq!(
        login_status(router.clone(), admin_id, "123456").await,
        StatusCode::UNAUTHORIZED
    );

    // 쓴 코드는 버렸다
    let (status, body) = recover(router, admin_id, &code, "111111").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["code"], "WRONG_RECOVERY_CODE");
    let events: Vec<String> = list_for_test(&db)
        .await
        .into_iter()
        .map(|(_, event)| event)
        .collect();
    assert_eq!(events, ["recovery_used", "login", "login_failed"]);
}

#[tokio::test]
async fn wrong_code_changes_nothing() {
    let (_dir, _db, router) = app_without_staff().await;
    let (admin_id, _code) = setup_admin_with_code(router.clone()).await;

    let (status, body) = recover(router.clone(), admin_id, "2222-2222-2222-2222", "654321").await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["code"], "WRONG_RECOVERY_CODE");
    assert_eq!(
        login_status(router, admin_id, "123456").await,
        StatusCode::OK
    );
}

#[tokio::test]
async fn only_admin_pin_can_be_recovered() {
    let (_dir, db, router) = app_without_staff().await;
    let (_admin_id, code) = setup_admin_with_code(router.clone()).await;
    let staff_id = insert_staff(&db, "직원1", "staff", None).await.unwrap();

    let (status, body) = recover(router, staff_id, &code, "654321").await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["code"], "RECOVERY_ADMIN_ONLY");
    assert_eq!(body["rule"], "BR-K12");
}

#[tokio::test]
async fn rejects_malformed_code_and_short_pin() {
    let (_dir, _db, router) = app_without_staff().await;

    let (status, body) = recover(router, 1, "0000-1111", "1234").await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["fields"][0]["field"], "code");
    assert_eq!(body["fields"][1]["field"], "pin");
    assert!(!body.to_string().contains("0000-1111"));
}

#[tokio::test]
async fn unknown_staff_is_not_found() {
    let (_dir, _db, router) = app_without_staff().await;

    let (status, body) = recover(router, 999, "2222-2222-2222-2222", "654321").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], "STAFF_NOT_FOUND");
}

#[tokio::test]
async fn needs_login() {
    let (_dir, _db, router) = app_without_staff().await;

    let response = get(router, "/api/changes").await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// 관리자 세션 하나와 승인한 기기 하나를 DB에 바로 넣고, 둘을 지켜보는 [`Watch`]를 만든다.
async fn watch_on_approved_pc(db: &SqlitePool) -> Watch {
    let staff_id = crate::db::test_support::insert_staff(db, "지켜볼직원", "admin", None)
        .await
        .unwrap();
    let session = crate::infra::secret::SessionToken::new().unwrap();
    sessions::start(
        db,
        staff_id,
        None,
        &session.hash(),
        OffsetDateTime::now_utc(),
    )
    .await
    .unwrap();
    let device = crate::infra::secret::DeviceToken::new().unwrap();
    sqlx::query(
        "INSERT INTO devices (name, token_hash, approved_at, approved_by)
         VALUES ('시험 PC', ?, '2026-11-24T00:00:00.000Z', ?)",
    )
    .bind(device.hash().as_slice())
    .bind(staff_id)
    .execute(db)
    .await
    .unwrap();
    Watch {
        session: session.hash(),
        device: Some(device.hash()),
    }
}

#[tokio::test]
async fn br_k16_revoked_device_ends_the_connection() {
    let (_dir, db, _router) = app_without_staff().await;
    let watch = watch_on_approved_pc(&db).await;
    let now = OffsetDateTime::now_utc();
    assert!(watch.still_open(&db, now).await);

    sqlx::query("UPDATE devices SET revoked_at = '2026-11-24T00:00:00.000Z'")
        .execute(&db)
        .await
        .unwrap();

    assert!(!watch.still_open(&db, now).await);
}

#[tokio::test]
async fn br_k8_locked_session_ends_the_connection() {
    let (_dir, db, _router) = app_without_staff().await;
    let watch = watch_on_approved_pc(&db).await;

    let later = OffsetDateTime::now_utc() + time::Duration::minutes(10);

    assert!(!watch.still_open(&db, later).await);
}

#[tokio::test]
async fn closed_session_ends_the_check() {
    let (_dir, db, _router) = app().await;
    let watch = Watch {
        session: [0; 32],
        device: None,
    };

    assert!(!watch.still_open(&db, OffsetDateTime::now_utc()).await);
}
