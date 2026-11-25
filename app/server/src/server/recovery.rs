//! `/api/recovery`: 복구 코드로 관리자 PIN 다시 정하기 (BR-K12).
//!
//! PIN을 잊어 로그인하지 못한 관리자가 쓰므로 로그인 없이 받는다.

mod json;

use axum::Json;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use baeum_domain::{Pin, RecoveryCode, Role};

use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::staff::field_name;
use crate::db::{recovery, staff};
use crate::http::error::{ApiError, FieldError};
use crate::infra::secret::{self, AdminSecrets};

use json::{RecoveryRequest, RecoveryResponse};

/// 복구 코드가 맞으면 고른 관리자의 PIN을 새로 정하고 새 복구 코드를 돌려준다.
///
/// 쓴 코드는 버린다. 계정 잠금(BR-K7)도 풀린다. 다른 역할의 직원은 403.
pub async fn recover(
    State(db): State<SqlitePool>,
    body: Result<Json<RecoveryRequest>, JsonRejection>,
) -> Result<Json<RecoveryResponse>, ApiError> {
    let Json(RecoveryRequest {
        staff_id,
        code,
        pin,
    }) = body?;
    let (code, pin) = parse_input(&code, &pin)?;

    let account = staff::get_login_account(&db, staff_id)
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::StaffNotFound)?;
    if account.role != Role::Admin {
        return Err(ApiError::RecoveryAdminOnly);
    }
    if !account.is_usable() {
        return Err(ApiError::AccountUnavailable);
    }
    let stored = recovery::code_hash(&db)
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::WrongRecoveryCode)?;

    // 해시 확인 한 번과 새로 만들기 두 번이라 느리다. 따로 돌리고 stored는 돌려받는다
    let (stored, secrets) = tokio::task::spawn_blocking(move || {
        let secrets = if secret::verify(code.as_str(), &stored) {
            AdminSecrets::new(&pin).map(Some)
        } else {
            Ok(None)
        };
        (stored, secrets)
    })
    .await
    .map_err(ApiError::internal)?;
    let secrets = secrets
        .map_err(ApiError::internal)?
        .ok_or(ApiError::WrongRecoveryCode)?;

    let used = recovery::use_code(
        &db,
        account.id,
        &stored,
        &secrets.recovery_code_hash,
        &secrets.pin_hash,
        OffsetDateTime::now_utc(),
    )
    .await
    .map_err(ApiError::internal)?;
    if !used {
        // 그사이 다른 화면에서 같은 코드를 먼저 썼다
        return Err(ApiError::WrongRecoveryCode);
    }
    tracing::info!(staff_id = account.id, "복구 코드로 관리자 PIN 다시 정함");

    Ok(Json(RecoveryResponse {
        recovery_code: secrets.recovery_code.to_string(),
    }))
}

/// 복구 코드 모양과 새 PIN(관리자 기준)을 함께 확인한다. 틀린 칸을 모두 모은다.
fn parse_input(code: &str, pin: &str) -> Result<(RecoveryCode, Pin), ApiError> {
    match (RecoveryCode::parse(code), Pin::parse(pin, Role::Admin)) {
        (Some(code), Ok(pin)) => Ok((code, pin)),
        (code, pin) => {
            let mut fields = Vec::new();
            if code.is_none() {
                fields.push(FieldError {
                    field: "code".to_owned(),
                    message: "복구 코드는 영문·숫자 16자입니다. 예: K7QF-M2XA-9TRE-HW4C".to_owned(),
                });
            }
            if let Err(error) = pin {
                fields.push(FieldError {
                    field: field_name(&error).to_owned(),
                    message: error.to_string(),
                });
            }
            Err(ApiError::InvalidInput(fields))
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::Router;
    use axum::http::StatusCode;
    use serde_json::{Value, json};

    use super::*;
    use crate::db::access_logs::list_for_test;
    use crate::db::test_support::insert_staff;
    use crate::server::test_support::{app_without_staff, get, json_body, post};

    /// 처음 설정으로 관리자(PIN 123456)를 만들고 (관리자 id, 복구 코드)를 돌려준다.
    async fn setup_admin(router: Router) -> (i64, String) {
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
        let (admin_id, code) = setup_admin(router.clone()).await;

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
        let (admin_id, _code) = setup_admin(router.clone()).await;

        let (status, body) =
            recover(router.clone(), admin_id, "2222-2222-2222-2222", "654321").await;

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
        let (_admin_id, code) = setup_admin(router.clone()).await;
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
}
