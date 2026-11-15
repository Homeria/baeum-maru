//! `/api/login`, `/api/logout`: 로그인과 로그아웃 (BR-K1, BR-K5~K7, BR-K13).

mod json;

use axum::Json;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::{HeaderMap, StatusCode, header};
use baeum_domain::is_valid_on;
use chrono::{Local, Utc};
use sqlx::SqlitePool;

use super::error::ApiError;
use crate::db::access_logs::{self, AccessEvent};
use crate::db::{sessions, staff};
use crate::secret::{self, SessionToken};

use json::{
    LoginRequest, LoginResponse, LoginStaffListResponse, LoginStaffResponse, SignedInStaffResponse,
};

/// 로그인 화면에서 고를 직원을 이름 순서로 보여 준다.
/// 사용 중지했거나 유효기간이 지난 계정은 빠진다.
pub async fn list_login_staff(
    State(db): State<SqlitePool>,
) -> Result<Json<LoginStaffListResponse>, ApiError> {
    let today = Local::now().date_naive();
    let choices = staff::list_for_login(&db, today)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(LoginStaffListResponse {
        items: choices.into_iter().map(LoginStaffResponse::from).collect(),
    }))
}

/// 고른 직원의 PIN을 확인하고 세션 증표를 준다.
///
/// 실패는 모두 접속 기록에 남긴다 (BR-K13). 잠겼거나 쓸 수 없는 계정은 PIN을 확인하지 않는다.
pub async fn login(
    State(db): State<SqlitePool>,
    body: Result<Json<LoginRequest>, JsonRejection>,
) -> Result<Json<LoginResponse>, ApiError> {
    let Json(LoginRequest { staff_id, pin }) = body?;
    let now = Utc::now();

    let Some(account) = staff::get_login_account(&db, staff_id)
        .await
        .map_err(ApiError::internal)?
    else {
        access_logs::record(&db, None, AccessEvent::LoginFailed, now)
            .await
            .map_err(ApiError::internal)?;
        return Err(ApiError::StaffNotFound);
    };

    // BR-K5, BR-K6: 유효기간은 기관의 달력 날짜라 이 PC의 날짜로 본다
    let refusal =
        if account.disabled || !is_valid_on(account.valid_until, Local::now().date_naive()) {
            Some(ApiError::AccountUnavailable)
        } else if account.locked_until.is_some_and(|until| now < until) {
            Some(ApiError::AccountLocked)
        } else {
            None
        };
    if let Some(refusal) = refusal {
        access_logs::record(&db, Some(account.id), AccessEvent::LoginFailed, now)
            .await
            .map_err(ApiError::internal)?;
        return Err(refusal);
    }

    // 해시 확인은 일부러 느려서 따로 돌린다. 클로저가 pin과 pin_hash를 가져간다
    let pin_hash = account.pin_hash;
    let pin_matches = tokio::task::spawn_blocking(move || secret::verify(&pin, &pin_hash))
        .await
        .map_err(ApiError::internal)?;
    if !pin_matches {
        let failed = staff::add_failed_pin(&db, account.id, now)
            .await
            .map_err(ApiError::internal)?;
        return Err(if failed.locked_until.is_some() {
            ApiError::AccountLocked
        } else {
            ApiError::LoginFailed
        });
    }

    let token = SessionToken::new().map_err(ApiError::internal)?;
    sessions::start(&db, account.id, &token.hash(), now)
        .await
        .map_err(ApiError::internal)?;
    tracing::info!(staff_id = account.id, "로그인");

    Ok(Json(LoginResponse {
        token: token.to_hex(),
        staff: SignedInStaffResponse {
            id: account.id,
            name: account.name,
            role: account.role.into(),
        },
    }))
}

/// 요청에 실린 세션을 끝낸다. 늘 204(본문 없음)를 돌려준다.
///
/// 세션이 이미 없어도(지워졌거나 증표가 없어도) 성공으로 본다. 앱은 어차피 증표를 버린다.
pub async fn logout(
    State(db): State<SqlitePool>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    if let Some(token) = bearer_token(&headers) {
        let ended = sessions::end(&db, &token.hash(), Utc::now())
            .await
            .map_err(ApiError::internal)?;
        if let Some(staff_id) = ended {
            tracing::info!(staff_id, "로그아웃");
        }
    }
    Ok(StatusCode::NO_CONTENT)
}

/// `Authorization: Bearer <증표>` 머리글의 증표. 없거나 모양이 틀리면 `None`.
fn bearer_token(headers: &HeaderMap) -> Option<SessionToken> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    SessionToken::parse(value.strip_prefix("Bearer ")?)
}

#[cfg(test)]
mod tests {
    use axum::Router;
    use axum::body::Body;
    use axum::http::{Request, Response};
    use serde_json::json;
    use tower::ServiceExt;

    use super::*;
    use crate::db::access_logs::list_for_test;
    use crate::db::test_support::insert_staff;
    use crate::server::test_support::{app, get, json_body, post};
    use crate::server::version::VERSION_HEADER;

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

    async fn logout(router: Router, token: Option<&str>) -> Response<Body> {
        let mut request =
            Request::post("/api/logout").header(VERSION_HEADER, crate::version::CURRENT);
        if let Some(token) = token {
            request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
        }
        router
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap()
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
        let (_dir, db, router) = app().await;
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
            json!({ "items": [{ "id": admin_id, "name": "관리자" }] })
        );
    }

    #[tokio::test]
    async fn login_gives_token_and_staff() {
        let (_dir, db, router) = app().await;
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
        let (_dir, db, router) = app().await;
        let admin_id = setup_admin(router.clone()).await;

        for _ in 0..4 {
            let response = login(router.clone(), admin_id, "000000").await;
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
            let body = json_body(response).await;
            assert_eq!(body["code"], "LOGIN_FAILED");
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
        let (_dir, db, router) = app().await;
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
        let (_dir, db, router) = app().await;
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
        let (_dir, db, router) = app().await;

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
        let (_dir, db, router) = app().await;
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
}
