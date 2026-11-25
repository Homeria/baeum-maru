//! 선발 결과: 마감한 강좌 목록은 `/api/terms/{termId}/selection-results`,
//! 강좌 하나는 `/api/courses/{id}/selection-result`, 포기·연락 결과·바로잡기는
//! `/api/applications/{id}/withdrawal`, `/contacts`, `/withdrawal-correction`.
//!
//! 보기는 모든 역할이 한다. 대기자 연락처는 포기·연락을 남길 수 있는 역할(관리자·직원)에게만
//! 싣는다(BR-W2). 바로잡기는 관리자만 한다(BR-W8). 호스트 PC가 아니어도 된다.

use axum::Json;
use axum::Router;
use axum::extract::FromRef;
use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use baeum_domain::{Action, ContactResult, Reason};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::dto::{
    ClosedCourseListResponse, ClosedCourseResponse, ContactRequest, CorrectWithdrawalRequest,
    CourseResultResponse, SelectedResponse, WaitingResponse, WithdrawRequest, WithdrawnResponse,
};
use super::repository::{self, CourseResultError};
use super::service;
use super::service::changes::{self as selection_changes, ChangeSelectionError};
use crate::features::auth::middleware::CurrentStaff;
use crate::features::terms::handler::require_term;
use crate::http::changes::{ChangeKind, ChangeNotices, notifies};
use crate::http::error::ApiError;
use crate::http::error::field_error;

/// 선발 결과 요청 주소. 로그인한 요청만 받는 묶음에 들어간다 (ADR 0023).
pub(crate) fn routes<S>(notices: &ChangeNotices) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    SqlitePool: FromRef<S>,
{
    Router::new()
        .route(
            "/api/terms/{term_id}/selection-results",
            get(list_closed_courses),
        )
        .route("/api/courses/{id}/selection-result", get(get_course_result))
        .route(
            "/api/applications/{id}/withdrawal",
            notifies(post(withdraw), notices, ChangeKind::Selection),
        )
        .route(
            "/api/applications/{id}/contacts",
            notifies(post(record_contact), notices, ChangeKind::Selection),
        )
        .route(
            "/api/applications/{id}/withdrawal-correction",
            notifies(post(correct_withdrawal), notices, ChangeKind::Selection),
        )
}

/// 학기에서 선발을 마감한 강좌들의 인원. 학기가 없으면 404.
async fn list_closed_courses(
    State(db): State<SqlitePool>,
    term_id: Result<Path<i64>, PathRejection>,
) -> Result<Json<ClosedCourseListResponse>, ApiError> {
    let Path(term_id) = term_id?;
    require_term(&db, term_id).await?;
    let courses = service::closed_courses(&db, term_id)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(ClosedCourseListResponse {
        items: courses
            .into_iter()
            .map(ClosedCourseResponse::from)
            .collect(),
    }))
}

/// 마감한 강좌 하나의 선발 결과. 마감 전이면 409.
async fn get_course_result(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
) -> Result<Json<CourseResultResponse>, ApiError> {
    let Path(id) = id?;
    let result = repository::course_result(&db, id)
        .await
        .map_err(|error| match error {
            CourseResultError::CourseNotFound => ApiError::CourseNotFound,
            CourseResultError::NotClosed => ApiError::SelectionNotClosed,
            CourseResultError::Db(error) => ApiError::internal(error),
        })?;
    let can_record = staff.role.can(Action::RecordWithdrawals);
    Ok(Json(CourseResultResponse {
        capacity: result.capacity.value(),
        open_seats: result.open_seats,
        selected: result
            .selected
            .into_iter()
            .map(|entry| SelectedResponse {
                member: entry.member.into(),
                guaranteed: entry.guaranteed,
                waitlist_number: entry.waitlist_number,
            })
            .collect(),
        waitlisted: result
            .waitlisted
            .into_iter()
            .map(|entry| WaitingResponse {
                member: entry.member.into(),
                // BR-W2: 연락하는 사람에게만
                phone_number: can_record.then(|| entry.phone_number.into_inner()),
                waitlist_number: entry.waitlist_number,
                last_contact: entry.last_contact.map(Into::into),
                contact_turn: entry.contact_turn,
            })
            .collect(),
        withdrawn: result
            .withdrawn
            .into_iter()
            .map(|entry| WithdrawnResponse {
                member: entry.member.into(),
                withdrawn_from: entry.withdrawn_from.into(),
                waitlist_number: entry.waitlist_number,
            })
            .collect(),
        can_record,
        can_correct: staff.role.can(Action::CorrectWithdrawals),
    }))
}

/// 당첨이나 대기인 신청을 포기로 바꾼다(BR-C3). 성공하면 204.
async fn withdraw(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
    body: Result<Json<WithdrawRequest>, JsonRejection>,
) -> Result<StatusCode, ApiError> {
    staff.require(Action::RecordWithdrawals)?;
    let Path(id) = id?;
    let Json(request) = body?;
    let reason = parse_reason(request.reason.as_deref())?;
    selection_changes::withdraw(
        &db,
        id,
        reason.as_ref().map(Reason::as_str),
        staff.id,
        OffsetDateTime::now_utc(),
    )
    .await?;
    tracing::info!(staff_id = staff.id, application_id = id, "포기");
    Ok(StatusCode::NO_CONTENT)
}

/// 대기자에게 연락한 결과를 남긴다(BR-W2~W5). 성공하면 204. 빈자리 없이 수락하면 409.
async fn record_contact(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
    body: Result<Json<ContactRequest>, JsonRejection>,
) -> Result<StatusCode, ApiError> {
    staff.require(Action::RecordWithdrawals)?;
    let Path(id) = id?;
    let Json(request) = body?;
    let result = ContactResult::from(request.result);
    selection_changes::record_contact(&db, id, result, staff.id, OffsetDateTime::now_utc()).await?;
    tracing::info!(
        staff_id = staff.id,
        application_id = id,
        result = result.code(),
        "대기자 연락 결과"
    );
    Ok(StatusCode::NO_CONTENT)
}

/// 잘못 처리한 포기를 바로잡는다(BR-W8). 관리자만, 사유 필수. 성공하면 204.
async fn correct_withdrawal(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
    body: Result<Json<CorrectWithdrawalRequest>, JsonRejection>,
) -> Result<StatusCode, ApiError> {
    staff.require(Action::CorrectWithdrawals)?;
    let Path(id) = id?;
    let Json(request) = body?;
    let reason = parse_reason(Some(&request.reason))?
        .ok_or_else(|| field_error("reason", "바로잡는 사유를 적어 주세요".to_owned()))?;
    selection_changes::correct_withdrawal(
        &db,
        id,
        reason.as_str(),
        staff.id,
        OffsetDateTime::now_utc(),
    )
    .await?;
    tracing::info!(staff_id = staff.id, application_id = id, "포기 바로잡기");
    Ok(StatusCode::NO_CONTENT)
}

/// 사유 칸을 확인한다. 비었으면 `None`, 너무 길면 `reason` 칸의 422.
fn parse_reason(input: Option<&str>) -> Result<Option<Reason>, ApiError> {
    Reason::parse(input.unwrap_or("")).map_err(|error| field_error("reason", error.to_string()))
}

impl From<ChangeSelectionError> for ApiError {
    fn from(error: ChangeSelectionError) -> Self {
        match error {
            ChangeSelectionError::ApplicationNotFound => ApiError::ApplicationNotFound,
            ChangeSelectionError::NotClosed => ApiError::SelectionNotClosed,
            ChangeSelectionError::WrongStatus => ApiError::SelectionChangeNotAllowed,
            ChangeSelectionError::NoOpenSeat => ApiError::NoOpenSeat,
            ChangeSelectionError::Db(error) => ApiError::internal(error),
        }
    }
}
