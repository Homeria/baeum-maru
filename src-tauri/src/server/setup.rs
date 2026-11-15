//! `/api/setup`: 처음 설정. 첫 관리자와 복구 코드를 만든다 (BR-K11).

mod json;

use axum::Json;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use baeum_domain::{NewStaff, Pin, RecoveryCode};
use chrono::Utc;
use sqlx::SqlitePool;

use super::error::ApiError;
use super::staff::field_name;
use crate::db::staff::{self, SetupError};
use crate::secret::{self, SecretError};

use json::{SetupRequest, SetupResponse, SetupStatusResponse};

/// 처음 설정이 필요한지 알려 준다. 화면이 처음 설정과 로그인 중 무엇을 띄울지 정한다.
pub async fn get_setup_status(
    State(db): State<SqlitePool>,
) -> Result<Json<SetupStatusResponse>, ApiError> {
    let needed = staff::is_setup_needed(&db)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(SetupStatusResponse { needed }))
}

/// 첫 관리자와 복구 코드를 만든다. 성공하면 201과 복구 코드를 돌려준다.
///
/// 이미 직원이 있으면 409. 복구 코드는 이 응답에서만 보이고 DB에는 해시만 남는다.
pub async fn setup(
    State(db): State<SqlitePool>,
    body: Result<Json<SetupRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let Json(request) = body?;
    let admin = NewStaff::parse(request.as_input(), &request.pin)
        .map_err(|errors| ApiError::invalid_input(errors, field_name))?;

    // 해시는 일부러 느려서, 요청을 받는 스레드를 막지 않게 따로 돌린다.
    // 다른 스레드로 넘어가므로 admin을 통째로 넘겼다가 돌려받는다
    let (admin, secrets) = tokio::task::spawn_blocking(move || {
        let secrets = Secrets::make(admin.pin());
        (admin, secrets)
    })
    .await
    .map_err(ApiError::internal)?;
    let secrets = secrets.map_err(ApiError::internal)?;

    let admin_id = staff::setup(
        &db,
        admin.profile().name(),
        &secrets.pin_hash,
        &secrets.recovery_code_hash,
        Utc::now(),
    )
    .await?;
    tracing::info!(staff_id = admin_id, "처음 설정: 관리자와 복구 코드 만듦");

    Ok((
        StatusCode::CREATED,
        Json(SetupResponse {
            recovery_code: secrets.recovery_code.to_string(),
        }),
    ))
}

/// 처음 설정에서 만드는 비밀들. 복구 코드 글자는 응답에 한 번 싣고 버린다.
struct Secrets {
    pin_hash: String,
    recovery_code: RecoveryCode,
    recovery_code_hash: String,
}

impl Secrets {
    fn make(pin: &Pin) -> Result<Self, SecretError> {
        let recovery_code = secret::new_recovery_code()?;
        Ok(Secrets {
            pin_hash: secret::hash(pin.as_str())?,
            recovery_code_hash: secret::hash(recovery_code.as_str())?,
            recovery_code,
        })
    }
}

impl From<SetupError> for ApiError {
    fn from(error: SetupError) -> Self {
        match error {
            SetupError::AlreadyDone => ApiError::SetupAlreadyDone,
            SetupError::Db(error) => ApiError::internal(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::server::test_support::{app_without_staff, get, json_body, post};

    fn admin(name: &str, pin: &str) -> String {
        json!({ "name": name, "pin": pin }).to_string()
    }

    async fn setup_needed(router: axum::Router) -> bool {
        let response = get(router, "/api/setup").await;
        assert_eq!(response.status(), StatusCode::OK);
        json_body(response).await["needed"].as_bool().unwrap()
    }

    #[tokio::test]
    async fn new_db_needs_setup() {
        let (_dir, _db, router) = app_without_staff().await;

        assert!(setup_needed(router).await);
    }

    #[tokio::test]
    async fn creates_admin_and_shows_recovery_code_once() {
        let (_dir, db, router) = app_without_staff().await;

        let response = post(router.clone(), "/api/setup", &admin("관리자", "123456")).await;

        assert_eq!(response.status(), StatusCode::CREATED);
        let shown = json_body(response).await["recoveryCode"]
            .as_str()
            .unwrap()
            .to_owned();
        let code = RecoveryCode::parse(&shown).unwrap();
        assert_eq!(code.to_string(), shown);
        assert!(!setup_needed(router).await);

        // DB에는 해시만 있고, 그 해시가 보여 준 코드·넣은 PIN과 맞는다
        let (pin_hash, role): (String, String) = sqlx::query_as("SELECT pin_hash, role FROM staff")
            .fetch_one(&db)
            .await
            .unwrap();
        assert_eq!(role, "admin");
        assert!(secret::verify("123456", &pin_hash));
        let code_hash: String = sqlx::query_scalar("SELECT code_hash FROM recovery_code")
            .fetch_one(&db)
            .await
            .unwrap();
        assert!(secret::verify(code.as_str(), &code_hash));
    }

    #[tokio::test]
    async fn setup_runs_only_once() {
        let (_dir, _db, router) = app_without_staff().await;
        let first = post(router.clone(), "/api/setup", &admin("관리자", "123456")).await;
        assert_eq!(first.status(), StatusCode::CREATED);

        let response = post(router, "/api/setup", &admin("다른관리자", "654321")).await;

        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = json_body(response).await;
        assert_eq!(body["code"], "SETUP_ALREADY_DONE");
        assert_eq!(body["rule"], "BR-K11");
    }

    #[tokio::test]
    async fn rejects_empty_name_and_short_admin_pin() {
        let (_dir, _db, router) = app_without_staff().await;

        let response = post(router.clone(), "/api/setup", &admin(" ", "1234")).await;

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = json_body(response).await;
        assert_eq!(body["fields"][0]["field"], "name");
        assert_eq!(body["fields"][1]["field"], "pin");
        assert_eq!(
            body["fields"][1]["message"],
            "관리자 PIN은 6자리 이상입니다"
        );
        // 넣은 PIN은 응답에 다시 싣지 않는다
        assert!(!body.to_string().contains("1234"));
        assert!(setup_needed(router).await);
    }

    #[tokio::test]
    async fn rejects_malformed_body() {
        let (_dir, _db, router) = app_without_staff().await;

        let response = post(router, "/api/setup", r#"{"name":"관리자"}"#).await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
