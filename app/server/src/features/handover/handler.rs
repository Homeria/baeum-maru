//! 넘기기와 실행 모드 요청 처리.
//!
//! - `POST /api/handover`: 호스트 넘기기를 시작한다(BR-H1). 서버는 관리자 PIN을 확인하고, 접속
//!   기록에 남기고, 실행 모드를 "넘기는 중"으로 바꿔 둘 뿐이다. 화면이 앱을 다시 켜면 서버 없이
//!   DB만 열어 넘기기 파일을 만든다([`super::service::hand_over`]). 그래서 넘기기 파일을 만드는 동안 다른
//!   PC가 저장하는 일이 없다.
//! - `POST /api/run-mode/permit`: 실행 모드 바꾸기를 관리자 PIN으로 허락받는다(BR-K17, ADR 0021).
//!   바꾸는 일 자체는 그 PC의 앱이 한다. 접속 모드 PC는 직원 계정이 없어서 호스트에 묻는다.

use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRef, State};
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use baeum_domain::{Action, Role};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::dto::{HandoverRequest, RunModePermitRequest};
use super::service::run_mode::{self, RunMode};
use crate::features::access_logs::repository::{self as access_logs, AccessEvent};
use crate::features::auth::middleware::{Client, CurrentDevice, CurrentStaff};
use crate::features::auth::repository as auth;
use crate::features::auth::service::check_pin;
use crate::features::backups::service::Backups;
use crate::http::error::ApiError;

/// 넘기기 시작 주소. 로그인한 요청만 받는 묶음에 들어간다.
pub(crate) fn routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    SqlitePool: FromRef<S>,
    Backups: FromRef<S>,
{
    Router::new().route("/api/handover", post(start_handover))
}

/// BR-K17: 로그인 전 화면에서 실행 모드를 바꿀 때 관리자 PIN을 확인하는 주소.
/// 기기 확인 안, 로그인 확인 바깥에 들어간다.
pub(crate) fn permit_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    SqlitePool: FromRef<S>,
{
    Router::new().route("/api/run-mode/permit", post(permit))
}

/// 관리자 PIN이 맞으면 넘기기를 시작하고 204(본문 없음)를 돌려준다. 호스트 PC에서만.
///
/// 틀린 PIN은 로그인과 같이 BR-K7의 횟수에 들어가고 로그인 실패로 남는다.
async fn start_handover(
    State(db): State<SqlitePool>,
    State(backups): State<Backups>,
    staff: CurrentStaff,
    client: Client,
    body: Result<Json<HandoverRequest>, JsonRejection>,
) -> Result<StatusCode, ApiError> {
    staff.require(Action::HandOverHost)?;
    client.require_host()?;
    let Json(HandoverRequest { pin }) = body?;
    let now = OffsetDateTime::now_utc();

    let account = auth::get_login_account(&db, staff.id)
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::StaffNotFound)?;
    // 호스트 PC에서만 하므로 기기는 늘 호스트 자신이다
    let account = check_pin(&db, account, pin, None, AccessEvent::LoginFailed, now).await?;

    // 남기지 못하면 넘기지 않는다
    access_logs::record(&db, Some(account.id), AccessEvent::Handover, now)
        .await
        .map_err(ApiError::internal)?;
    run_mode::save(backups.data_dir(), RunMode::HandingOver).map_err(ApiError::internal)?;
    tracing::info!(staff_id = account.id, "호스트 넘기기 시작");
    Ok(StatusCode::NO_CONTENT)
}

/// 고른 관리자의 PIN이 맞으면 204(본문 없음)를 돌려주고 접속 기록에 남긴다.
///
/// 틀린 PIN은 로그인과 같이 BR-K7의 횟수에 들어가고 로그인 실패로 남는다.
async fn permit(
    State(db): State<SqlitePool>,
    device: CurrentDevice,
    body: Result<Json<RunModePermitRequest>, JsonRejection>,
) -> Result<StatusCode, ApiError> {
    let Json(RunModePermitRequest { staff_id, pin }) = body?;
    let now = OffsetDateTime::now_utc();

    let account = auth::get_login_account(&db, staff_id)
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::StaffNotFound)?;
    if account.role != Role::Admin {
        return Err(ApiError::NotAllowed);
    }
    let account = check_pin(&db, account, pin, device.id, AccessEvent::LoginFailed, now).await?;

    access_logs::record_on_device(
        &db,
        Some(account.id),
        device.id,
        AccessEvent::RunModeChanged,
        now,
    )
    .await
    .map_err(ApiError::internal)?;
    tracing::info!(staff_id = account.id, "실행 모드 바꾸기 허락");
    Ok(StatusCode::NO_CONTENT)
}
