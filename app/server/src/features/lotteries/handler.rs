//! 추첨과 선발 마감: 학기 강좌의 선발 단계는 `/api/terms/{termId}/selections`,
//! 추첨 화면·추첨은 `/api/courses/{id}/lottery`, 마감은 `/api/courses/{id}/selection-closure`.
//!
//! 추첨·재추첨·마감은 관리자·직원이 호스트 PC에서만 한다(BR-L6). 보기는 모든 역할이 한다.

use axum::Json;
use axum::Router;
use axum::extract::FromRef;
use axum::extract::rejection::PathRejection;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use baeum_domain::Action;
use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::dto::{CourseLotteryResponse, CourseStageResponse, SelectionListResponse};
use super::error::{CloseSelectionError, RunLotteryError};
use super::repository;
use super::service;
use crate::features::auth::middleware::{Client, CurrentStaff};
use crate::features::backups::service::Backups;
use crate::features::terms::handler::require_term;
use crate::http::changes::{ChangeKind, ChangeNotices, notifies};
use crate::http::error::ApiError;
use crate::infra::backup_file::name::BackupKind;
use crate::infra::secret;

/// 추첨 요청 주소. 로그인한 요청만 받는 묶음에 들어간다 (ADR 0023).
pub(crate) fn routes<S>(notices: &ChangeNotices) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    SqlitePool: FromRef<S>,
    Backups: FromRef<S>,
{
    Router::new()
        .route("/api/terms/{term_id}/selections", get(list_selections))
        .route(
            "/api/courses/{id}/lottery",
            notifies(
                get(get_lottery).post(run_lottery),
                notices,
                ChangeKind::Selection,
            ),
        )
        .route(
            "/api/courses/{id}/selection-closure",
            notifies(post(close_selection), notices, ChangeKind::Selection),
        )
}

/// 학기 강좌마다 선발 단계. 학기가 없으면 404.
async fn list_selections(
    State(db): State<SqlitePool>,
    _staff: CurrentStaff,
    term_id: Result<Path<i64>, PathRejection>,
) -> Result<Json<SelectionListResponse>, ApiError> {
    let Path(term_id) = term_id?;
    require_term(&db, term_id).await?;
    let stages = repository::stages_in_term(&db, term_id)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(SelectionListResponse {
        items: stages.into_iter().map(CourseStageResponse::from).collect(),
    }))
}

/// 강좌 하나의 추첨 화면. 강좌가 없으면 404.
async fn get_lottery(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    client: Client,
    id: Result<Path<i64>, PathRejection>,
) -> Result<Json<CourseLotteryResponse>, ApiError> {
    let Path(id) = id?;
    lottery_response(&db, id, staff, client).await.map(Json)
}

/// 강좌를 추첨한다. 이미 추첨했으면 다시 추첨한다(BR-L4). 성공하면 추첨 화면을 돌려준다.
///
/// 시드는 지금(멈춤을 누른 순간) 운영체제 난수로 만든다(ADR 0016).
async fn run_lottery(
    State(db): State<SqlitePool>,
    State(backups): State<Backups>,
    staff: CurrentStaff,
    client: Client,
    id: Result<Path<i64>, PathRejection>,
) -> Result<Json<CourseLotteryResponse>, ApiError> {
    staff.require(Action::RunLottery)?;
    client.require_host()?;
    let Path(id) = id?;
    // ADR 0011: 되돌리기 어려운 작업 직전 백업. 실패해도 추첨은 한다
    backups
        .make(
            BackupKind::BeforeLottery { course_id: id },
            OffsetDateTime::now_utc(),
        )
        .await;
    let seed = secret::new_lottery_seed().map_err(ApiError::internal)?;
    let run_id = service::run(&db, id, staff.id, seed, OffsetDateTime::now_utc())
        .await
        .map_err(|error| match error {
            RunLotteryError::CourseNotFound => ApiError::CourseNotFound,
            RunLotteryError::Closed => ApiError::SelectionClosed,
            RunLotteryError::NoApplicants => ApiError::NoApplicants,
            RunLotteryError::OverCapacity { .. } => ApiError::PriorityOverCapacity,
            RunLotteryError::Db(error) => ApiError::internal(error),
        })?;
    tracing::info!(staff_id = staff.id, course_id = id, run_id, "추첨");
    lottery_response(&db, id, staff, client).await.map(Json)
}

/// 강좌의 선발을 마감한다(BR-L5). 성공하면 204(본문 없음).
async fn close_selection(
    State(db): State<SqlitePool>,
    State(backups): State<Backups>,
    staff: CurrentStaff,
    client: Client,
    id: Result<Path<i64>, PathRejection>,
) -> Result<StatusCode, ApiError> {
    staff.require(Action::RunLottery)?;
    client.require_host()?;
    let Path(id) = id?;
    backups
        .make(
            BackupKind::BeforeClose { course_id: id },
            OffsetDateTime::now_utc(),
        )
        .await;
    service::close(&db, id, staff.id, OffsetDateTime::now_utc())
        .await
        .map_err(|error| match error {
            CloseSelectionError::CourseNotFound => ApiError::CourseNotFound,
            CloseSelectionError::NotDrawn => ApiError::NotDrawn,
            CloseSelectionError::AlreadyClosed => ApiError::SelectionClosed,
            CloseSelectionError::Db(error) => ApiError::internal(error),
        })?;
    tracing::info!(staff_id = staff.id, course_id = id, "선발 마감");
    Ok(StatusCode::NO_CONTENT)
}

async fn lottery_response(
    db: &SqlitePool,
    course_id: i64,
    staff: CurrentStaff,
    client: Client,
) -> Result<CourseLotteryResponse, ApiError> {
    let lottery = repository::course_lottery(db, course_id)
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::CourseNotFound)?;
    Ok(CourseLotteryResponse {
        capacity: lottery.capacity.value(),
        stage: lottery.stage.into(),
        entrants: lottery.entrants.into_iter().map(Into::into).collect(),
        selected_count: lottery.selected_count,
        can_draw: staff.role.can(Action::RunLottery) && client.on_host,
    })
}
