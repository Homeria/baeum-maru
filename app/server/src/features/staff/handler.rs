//! `/api/staff`: 직원 관리 (BR-K2~K6, BR-K9, BR-K10). 관리자만 쓴다.
//!
//! 직원은 지우지 않고 사용 중지한다 (BR-K6). 그래서 지우기 주소가 없다.

use axum::Json;
use axum::Router;
use axum::extract::FromRef;
use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use baeum_domain::{Action, NewStaff, Pin, StaffError, StaffProfile};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::dto::{
    CreateStaffRequest, CreateStaffResponse, ResetPinRequest, StaffListResponse, StaffResponse,
    UpdateStaffRequest,
};
use super::repository::{self, SaveStaffError};
use crate::features::auth::middleware::CurrentStaff;
use crate::http::changes::{ChangeKind, ChangeNotices, notifies};
use crate::http::error::ApiError;
use crate::infra::clock::local_today;
use crate::infra::secret;

/// 직원 요청 주소. 로그인한 요청만 받는 묶음에 들어간다 (ADR 0023).
pub(crate) fn routes<S>(notices: &ChangeNotices) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    SqlitePool: FromRef<S>,
{
    Router::new()
        .route(
            "/api/staff",
            notifies(
                get(list_staff).post(create_staff),
                notices,
                ChangeKind::Staff,
            ),
        )
        .route(
            "/api/staff/{id}",
            notifies(
                get(get_staff).patch(update_staff),
                notices,
                ChangeKind::Staff,
            ),
        )
        .route(
            "/api/staff/{id}/pin",
            notifies(post(reset_staff_pin), notices, ChangeKind::Staff),
        )
}

/// 직원을 모두 보여 준다. 사용 중인 직원 먼저, 이름 순서.
async fn list_staff(
    State(db): State<SqlitePool>,
    current: CurrentStaff,
) -> Result<Json<StaffListResponse>, ApiError> {
    current.require(Action::ManageStaff)?;
    let records = repository::list(&db).await.map_err(ApiError::internal)?;
    let now = OffsetDateTime::now_utc();
    Ok(Json(StaffListResponse {
        items: records
            .into_iter()
            .map(|record| StaffResponse::new(record, now))
            .collect(),
    }))
}

/// `id`인 직원 하나를 보여 준다.
async fn get_staff(
    State(db): State<SqlitePool>,
    current: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
) -> Result<Json<StaffResponse>, ApiError> {
    current.require(Action::ManageStaff)?;
    let Path(id) = id?;
    let record = repository::get(&db, id)
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::StaffNotFound)?;
    Ok(Json(StaffResponse::new(record, OffsetDateTime::now_utc())))
}

/// 새 직원을 저장한다. 성공하면 201과 새 id를 돌려준다.
async fn create_staff(
    State(db): State<SqlitePool>,
    current: CurrentStaff,
    body: Result<Json<CreateStaffRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    current.require(Action::ManageStaff)?;
    let Json(request) = body?;
    let new_staff = NewStaff::parse(request.as_input(), &request.pin)
        .map_err(|errors| ApiError::invalid_input(errors, field_name))?;

    // 해시는 일부러 느려서 따로 돌린다. 새 직원을 넘겼다가 돌려받는다
    let (new_staff, pin_hash) = tokio::task::spawn_blocking(move || {
        let pin_hash = secret::hash(new_staff.pin().as_str());
        (new_staff, pin_hash)
    })
    .await
    .map_err(ApiError::internal)?;
    let pin_hash = pin_hash.map_err(ApiError::internal)?;

    let id = repository::insert(&db, &new_staff, &pin_hash, OffsetDateTime::now_utc()).await?;
    tracing::info!(staff_id = current.id, target_staff_id = id, "직원 저장");

    let location = format!("/api/staff/{id}");
    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, location)],
        Json(CreateStaffResponse { id }),
    ))
}

/// `id`인 직원의 이름·역할·유효기간·사용 중지를 본문 값으로 바꾼다.
///
/// BR-K9: 사용 중인 관리자가 하나도 남지 않게 되면 409.
async fn update_staff(
    State(db): State<SqlitePool>,
    current: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
    body: Result<Json<UpdateStaffRequest>, JsonRejection>,
) -> Result<Json<StaffResponse>, ApiError> {
    current.require(Action::ManageStaff)?;
    let Path(id) = id?;
    let Json(request) = body?;
    let profile = StaffProfile::parse(request.as_input())
        .map_err(|errors| ApiError::invalid_input(errors, field_name))?;

    let today = local_today();
    let now = OffsetDateTime::now_utc();
    let record = repository::update(&db, id, &profile, request.disabled, today, now)
        .await?
        .ok_or(ApiError::StaffNotFound)?;
    tracing::info!(staff_id = current.id, target_staff_id = id, "직원 고침");
    Ok(Json(StaffResponse::new(record, now)))
}

/// BR-K10: `id`인 직원의 PIN을 새로 정한다. 계정 잠금도 풀린다. 성공하면 204(본문 없음).
///
/// PIN 길이는 그 직원의 역할로 확인한다 (BR-K4: 관리자는 6자리 이상).
async fn reset_staff_pin(
    State(db): State<SqlitePool>,
    current: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
    body: Result<Json<ResetPinRequest>, JsonRejection>,
) -> Result<StatusCode, ApiError> {
    current.require(Action::ManageStaff)?;
    let Path(id) = id?;
    let Json(request) = body?;
    let target = repository::get(&db, id)
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::StaffNotFound)?;
    let pin = Pin::parse(&request.pin, target.role)
        .map_err(|error| ApiError::invalid_input(vec![error], field_name))?;

    let pin_hash = tokio::task::spawn_blocking(move || secret::hash(pin.as_str()))
        .await
        .map_err(ApiError::internal)?
        .map_err(ApiError::internal)?;
    let changed = repository::reset_pin(&db, id, &pin_hash, OffsetDateTime::now_utc())
        .await
        .map_err(ApiError::internal)?;
    if !changed {
        return Err(ApiError::StaffNotFound);
    }
    tracing::info!(
        staff_id = current.id,
        target_staff_id = id,
        "직원 PIN 새로 정함"
    );
    Ok(StatusCode::NO_CONTENT)
}

/// 오류가 난 칸의 요청 JSON 이름. 처음 설정도 함께 쓴다.
pub(crate) fn field_name(error: &StaffError) -> &'static str {
    match error {
        StaffError::EmptyName | StaffError::NameTooLong => "name",
        StaffError::InvalidPin | StaffError::AdminPinTooShort => "pin",
        StaffError::MissingValidUntil => "validUntil",
    }
}

impl From<SaveStaffError> for ApiError {
    fn from(error: SaveStaffError) -> Self {
        match error {
            SaveStaffError::DuplicateName => ApiError::DuplicateStaffName,
            SaveStaffError::LastAdmin => ApiError::LastAdmin,
            SaveStaffError::Db(error) => ApiError::internal(error),
        }
    }
}
