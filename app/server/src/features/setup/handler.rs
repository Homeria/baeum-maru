//! 처음 설정 요청 처리: `/api/setup`. 첫 관리자와 복구 코드를 만든다 (BR-K11).

use axum::extract::FromRef;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use baeum_domain::NewStaff;

use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::repository::{self, SetupError};
use crate::features::staff::handler::field_name;
use crate::http::error::ApiError;
use crate::infra::secret::AdminSecrets;

use super::dto::{SetupRequest, SetupResponse, SetupStatusResponse};

/// 처음 설정 주소. 직원이 없을 때 하므로 기기 확인 안, 로그인 확인 바깥에 들어간다.
pub(crate) fn routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    SqlitePool: FromRef<S>,
{
    Router::new().route("/api/setup", get(get_setup_status).post(setup))
}

/// 처음 설정이 필요한지 알려 준다. 화면이 처음 설정과 로그인 중 무엇을 띄울지 정한다.
async fn get_setup_status(
    State(db): State<SqlitePool>,
) -> Result<Json<SetupStatusResponse>, ApiError> {
    let needed = repository::is_setup_needed(&db)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(SetupStatusResponse { needed }))
}

/// 첫 관리자와 복구 코드를 만든다. 성공하면 201과 복구 코드를 돌려준다.
///
/// 이미 직원이 있으면 409. 복구 코드는 이 응답에서만 보이고 DB에는 해시만 남는다.
async fn setup(
    State(db): State<SqlitePool>,
    body: Result<Json<SetupRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let Json(request) = body?;
    let admin = NewStaff::parse(request.as_input(), &request.pin)
        .map_err(|errors| ApiError::invalid_input(errors, field_name))?;

    // 해시는 일부러 느려서, 요청을 받는 스레드를 막지 않게 따로 돌린다.
    // 다른 스레드로 넘어가므로 admin을 통째로 넘겼다가 돌려받는다
    let (admin, secrets) = tokio::task::spawn_blocking(move || {
        let secrets = AdminSecrets::new(admin.pin());
        (admin, secrets)
    })
    .await
    .map_err(ApiError::internal)?;
    let secrets = secrets.map_err(ApiError::internal)?;

    let admin_id = repository::setup(
        &db,
        admin.profile().name(),
        &secrets.pin_hash,
        &secrets.recovery_code_hash,
        OffsetDateTime::now_utc(),
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

impl From<SetupError> for ApiError {
    fn from(error: SetupError) -> Self {
        match error {
            SetupError::AlreadyDone => ApiError::SetupAlreadyDone,
            SetupError::Db(error) => ApiError::internal(error),
        }
    }
}
