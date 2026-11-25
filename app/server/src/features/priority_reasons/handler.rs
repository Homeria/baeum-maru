//! `/api/priority-reasons`: 우선 배정 사유(BR-S10, BR-S11).
//!
//! 목록은 모든 역할이 보고, 바꾸기는 관리자만 한다.

use axum::Json;
use axum::Router;
use axum::extract::FromRef;
use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::{get, patch};
use baeum_domain::{Action, NewPriorityReason, PriorityError};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::dto::{
    CreatePriorityReasonResponse, PriorityReasonListResponse, PriorityReasonRequest,
    PriorityReasonResponse,
};
use super::error::SavePriorityReasonError;
use super::repository;
use crate::features::auth::middleware::CurrentStaff;
use crate::http::changes::{ChangeKind, ChangeNotices, notifies};
use crate::http::error::ApiError;

/// 우선 배정 사유 요청 주소. 로그인한 요청만 받는 묶음에 들어간다 (ADR 0023).
pub(crate) fn routes<S>(notices: &ChangeNotices) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    SqlitePool: FromRef<S>,
{
    Router::new()
        .route(
            "/api/priority-reasons",
            notifies(
                get(list_priority_reasons).post(create_priority_reason),
                notices,
                ChangeKind::PriorityReason,
            ),
        )
        .route(
            "/api/priority-reasons/{id}",
            notifies(
                patch(update_priority_reason).delete(delete_priority_reason),
                notices,
                ChangeKind::PriorityReason,
            ),
        )
}

/// 사유를 모두 보여 준다. 이름 순서.
async fn list_priority_reasons(
    State(db): State<SqlitePool>,
) -> Result<Json<PriorityReasonListResponse>, ApiError> {
    let records = repository::list(&db).await.map_err(ApiError::internal)?;
    Ok(Json(PriorityReasonListResponse {
        items: records
            .into_iter()
            .map(PriorityReasonResponse::from)
            .collect(),
    }))
}

/// 새 사유를 저장한다. 성공하면 201과 새 id를 돌려준다.
async fn create_priority_reason(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    body: Result<Json<PriorityReasonRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    staff.require(Action::ManagePriorityReasons)?;
    let Json(request) = body?;
    let reason = parse(&request)?;
    let id = repository::insert(&db, &reason, OffsetDateTime::now_utc()).await?;
    // BR-S15: 사유 이름은 민감할 수 있어 id만 남긴다
    tracing::info!(staff_id = staff.id, reason_id = id, "우선 배정 사유 저장");

    let location = format!("/api/priority-reasons/{id}");
    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, location)],
        Json(CreatePriorityReasonResponse { id }),
    ))
}

/// `id`인 사유의 이름과 방식을 본문 값으로 바꾼다.
async fn update_priority_reason(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
    body: Result<Json<PriorityReasonRequest>, JsonRejection>,
) -> Result<Json<PriorityReasonResponse>, ApiError> {
    staff.require(Action::ManagePriorityReasons)?;
    let Path(id) = id?;
    let Json(request) = body?;
    let reason = parse(&request)?;
    let record = repository::update(&db, id, &reason, OffsetDateTime::now_utc())
        .await?
        .ok_or(ApiError::PriorityReasonNotFound)?;
    tracing::info!(staff_id = staff.id, reason_id = id, "우선 배정 사유 고침");
    Ok(Json(PriorityReasonResponse::from(record)))
}

/// `id`인 사유를 지운다. 성공하면 204. BR-S10: 신청에 붙었던 사유면 409.
async fn delete_priority_reason(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
) -> Result<StatusCode, ApiError> {
    staff.require(Action::ManagePriorityReasons)?;
    let Path(id) = id?;
    let deleted = repository::delete(&db, id)
        .await
        .map_err(|error| ApiError::from_delete(error, ApiError::PriorityReasonInUse))?;
    if !deleted {
        return Err(ApiError::PriorityReasonNotFound);
    }
    tracing::info!(staff_id = staff.id, reason_id = id, "우선 배정 사유 지움");
    Ok(StatusCode::NO_CONTENT)
}

fn parse(request: &PriorityReasonRequest) -> Result<NewPriorityReason, ApiError> {
    NewPriorityReason::parse(&request.name, request.method.into())
        .map_err(|error| ApiError::invalid_input(vec![error], field_name))
}

/// 오류가 난 칸의 요청 JSON 이름.
fn field_name(error: &PriorityError) -> &'static str {
    match error {
        PriorityError::EmptyName | PriorityError::NameTooLong => "name",
    }
}

impl From<SavePriorityReasonError> for ApiError {
    fn from(error: SavePriorityReasonError) -> Self {
        match error {
            SavePriorityReasonError::DuplicateName => ApiError::DuplicatePriorityReasonName,
            SavePriorityReasonError::Db(error) => ApiError::internal(error),
        }
    }
}
