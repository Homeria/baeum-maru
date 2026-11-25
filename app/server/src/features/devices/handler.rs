//! 기기 승인 (BR-K14~K16, ADR 0003).
//!
//! - `/api/approval/…`: 승인받지 않은 기기가 요청하고 결과를 본다. 기기 확인 없이 받는다.
//! - `/api/device-requests/…`, `/api/devices/…`: 관리자가 허락·거절하고 승인한 기기를 관리한다.

use axum::Json;
use axum::Router;
use axum::extract::FromRef;
use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{delete, get, post};
use baeum_domain::{Action, DeviceName};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::dto::{
    ApprovalRequest, ApprovalRequestResponse, ApprovalStatusResponse, ApproveDeviceRequest,
    DeviceListResponse, DeviceRequestListResponse, DeviceRequestResponse, DeviceResponse,
};
use super::error::{DecideError, RequestError};
use super::repository;
use crate::features::access_logs::repository::AccessEvent;
use crate::features::auth::middleware::{Client, CurrentDevice, CurrentStaff, device_token};
use crate::features::auth::repository as auth;
use crate::features::auth::service::check_pin;
use crate::http::changes::{ChangeKind, ChangeNotices, notifies};
use crate::http::error::{ApiError, FieldError};
use crate::infra::secret::{self, DeviceToken};

/// 승인받지 않은 기기가 승인을 요청하고 결과를 보는 주소 (BR-K15). 기기 확인과 로그인 확인 바깥에 들어간다.
pub(crate) fn approval_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    SqlitePool: FromRef<S>,
{
    Router::new()
        .route("/api/approval/request", post(request_approval))
        .route("/api/approval/status", get(approval_status))
}

/// 관리자가 승인 요청과 승인한 기기를 다루는 주소. 로그인한 요청만 받는 묶음에 들어간다 (ADR 0023).
pub(crate) fn routes<S>(notices: &ChangeNotices) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    SqlitePool: FromRef<S>,
{
    Router::new()
        .route("/api/device-requests", get(list_device_requests))
        .route(
            "/api/device-requests/{id}/approve",
            notifies(post(approve_device), notices, ChangeKind::Device),
        )
        .route(
            "/api/device-requests/{id}/reject",
            notifies(post(reject_device), notices, ChangeKind::Device),
        )
        .route("/api/devices", get(list_devices))
        .route(
            "/api/devices/{id}",
            notifies(delete(revoke_device), notices, ChangeKind::Device),
        )
}

/// BR-K15: 이 PC의 승인을 요청한다. 기기 증표와 승인 번호를 돌려준다.
///
/// 증표는 관리자가 허락한 뒤에만 쓸 수 있다. 요청이 쏟아지지 않게 기다리는 요청 수를 제한한다.
async fn request_approval(
    State(db): State<SqlitePool>,
    body: Result<Json<ApprovalRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<ApprovalRequestResponse>), ApiError> {
    let Json(request) = body?;
    let name = DeviceName::parse(&request.name).map_err(|error| {
        ApiError::InvalidInput(vec![FieldError {
            field: "name".to_owned(),
            message: error.to_string(),
        }])
    })?;
    let token = DeviceToken::new().map_err(ApiError::internal)?;
    let code = secret::new_approval_code().map_err(ApiError::internal)?;
    let id = repository::request(&db, &name, &code, &token.hash(), OffsetDateTime::now_utc())
        .await
        .map_err(|error| match error {
            RequestError::TooMany => ApiError::TooManyApprovalRequests,
            RequestError::Db(error) => ApiError::internal(error),
        })?;
    // 기기 이름과 번호는 남기지 않는다
    tracing::info!(request_id = id, "기기 승인 요청");
    Ok((
        StatusCode::CREATED,
        Json(ApprovalRequestResponse {
            token: token.to_hex(),
            code: code.as_str().to_owned(),
        }),
    ))
}

/// 요청에 실린 기기 증표의 승인 요청 상태.
///
/// # Errors
///
/// 증표가 없거나 그런 요청이 없으면 [`ApiError::ApprovalRequestNotOpen`].
async fn approval_status(
    State(db): State<SqlitePool>,
    headers: HeaderMap,
) -> Result<Json<ApprovalStatusResponse>, ApiError> {
    let token = device_token(&headers).ok_or(ApiError::ApprovalRequestNotOpen)?;
    let status = repository::request_status(&db, &token.hash(), OffsetDateTime::now_utc())
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::ApprovalRequestNotOpen)?;
    Ok(Json(ApprovalStatusResponse {
        status: status.into(),
    }))
}

/// 호스트 화면에 보여 줄 기다리는 승인 요청. 관리자가 호스트 PC에서만 본다.
async fn list_device_requests(
    State(db): State<SqlitePool>,
    current: CurrentStaff,
    client: Client,
) -> Result<Json<DeviceRequestListResponse>, ApiError> {
    current.require(Action::ManageDevices)?;
    client.require_host()?;
    let requests = repository::open_requests(&db, OffsetDateTime::now_utc())
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(DeviceRequestListResponse {
        items: requests
            .into_iter()
            .map(DeviceRequestResponse::from)
            .collect(),
    }))
}

/// BR-K15: 관리자가 호스트 PC에서 PIN을 넣어 승인 요청을 허락한다. 성공하면 204(본문 없음).
async fn approve_device(
    State(db): State<SqlitePool>,
    current: CurrentStaff,
    client: Client,
    device: CurrentDevice,
    id: Result<Path<i64>, PathRejection>,
    body: Result<Json<ApproveDeviceRequest>, JsonRejection>,
) -> Result<StatusCode, ApiError> {
    current.require(Action::ManageDevices)?;
    client.require_host()?;
    let Path(id) = id?;
    let Json(ApproveDeviceRequest { pin }) = body?;
    let now = OffsetDateTime::now_utc();

    let account = auth::get_login_account(&db, current.id)
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::LoginRequired)?;
    check_pin(
        &db,
        account,
        pin,
        device.id,
        AccessEvent::DeviceApprovalFailed,
        now,
    )
    .await?;

    let device_id = repository::approve(&db, id, current.id, now)
        .await
        .map_err(decide_error)?;
    tracing::info!(staff_id = current.id, device_id, "기기 허락");
    Ok(StatusCode::NO_CONTENT)
}

/// BR-K15: 관리자가 호스트 PC에서 승인 요청을 거절한다. 성공하면 204(본문 없음).
async fn reject_device(
    State(db): State<SqlitePool>,
    current: CurrentStaff,
    client: Client,
    id: Result<Path<i64>, PathRejection>,
) -> Result<StatusCode, ApiError> {
    current.require(Action::ManageDevices)?;
    client.require_host()?;
    let Path(id) = id?;
    repository::reject(&db, id, current.id, OffsetDateTime::now_utc())
        .await
        .map_err(decide_error)?;
    tracing::info!(staff_id = current.id, request_id = id, "기기 거절");
    Ok(StatusCode::NO_CONTENT)
}

/// BR-K16: 승인한 기기 목록. 관리자는 어느 PC에서나 본다.
async fn list_devices(
    State(db): State<SqlitePool>,
    current: CurrentStaff,
) -> Result<Json<DeviceListResponse>, ApiError> {
    current.require(Action::ManageDevices)?;
    let records = repository::list(&db).await.map_err(ApiError::internal)?;
    Ok(Json(DeviceListResponse {
        items: records.into_iter().map(DeviceResponse::from).collect(),
    }))
}

/// BR-K16: 기기 승인을 취소한다. 그 기기는 다음 요청부터 막힌다. 성공하면 204(본문 없음).
async fn revoke_device(
    State(db): State<SqlitePool>,
    current: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
) -> Result<StatusCode, ApiError> {
    current.require(Action::ManageDevices)?;
    let Path(id) = id?;
    let revoked = repository::revoke(&db, id, current.id, OffsetDateTime::now_utc())
        .await
        .map_err(ApiError::internal)?;
    if !revoked {
        return Err(ApiError::DeviceNotFound);
    }
    tracing::info!(staff_id = current.id, device_id = id, "기기 승인 취소");
    Ok(StatusCode::NO_CONTENT)
}

fn decide_error(error: DecideError) -> ApiError {
    match error {
        DecideError::NotOpen => ApiError::ApprovalRequestNotOpen,
        DecideError::Db(error) => ApiError::internal(error),
    }
}
