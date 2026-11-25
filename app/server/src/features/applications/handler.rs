//! 신청: 회원별 목록·신청은 `/api/terms/{termId}/applications`,
//! 강좌별 인원은 `/api/terms/{termId}/application-counts`, 지우기는 `/api/applications/{id}`,
//! 우선 배정 사유 바꾸기는 `/api/applications/{id}/priority-reasons`.

use axum::Json;
use axum::Router;
use axum::extract::FromRef;
use axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, put};
use baeum_domain::{Action, Reason};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::dto::{
    ApplicationCountListResponse, ApplicationCountResponse, ApplicationListResponse,
    ApplicationResponse, DeleteApplicationRequest, ListApplicationsParams,
    SetPriorityReasonsRequest, SetPriorityReasonsResponse, SubmitApplicationsRequest,
    SubmitApplicationsResponse, SubmittedResponse,
};
use super::repository;
use super::service::priorities::{self as application_priorities, SetPrioritiesError};
use super::service::{
    self, CoursePriorities, DeleteApplicationError, NewApplications, SubmitError,
};
use crate::features::auth::middleware::CurrentStaff;
use crate::features::terms::handler::require_term;
use crate::http::changes::{ChangeKind, ChangeNotices, notifies};
use crate::http::error::{ApiError, RuleViolationResponse, field_error};

/// 신청 요청 주소. 로그인한 요청만 받는 묶음에 들어간다 (ADR 0023).
pub(crate) fn routes<S>(notices: &ChangeNotices) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    SqlitePool: FromRef<S>,
{
    Router::new()
        .route(
            "/api/terms/{term_id}/applications",
            notifies(
                get(list_applications).post(submit_applications),
                notices,
                ChangeKind::Application,
            ),
        )
        .route(
            "/api/terms/{term_id}/application-counts",
            get(list_application_counts),
        )
        .route(
            "/api/applications/{id}",
            notifies(delete(delete_application), notices, ChangeKind::Application),
        )
        .route(
            "/api/applications/{id}/priority-reasons",
            notifies(put(set_priority_reasons), notices, ChangeKind::Application),
        )
}

/// 회원이 이 학기에 낸 유효한 신청. 학기가 없으면 404.
async fn list_applications(
    State(db): State<SqlitePool>,
    term_id: Result<Path<i64>, PathRejection>,
    params: Result<Query<ListApplicationsParams>, QueryRejection>,
) -> Result<Json<ApplicationListResponse>, ApiError> {
    let Path(term_id) = term_id?;
    let Query(params) = params?;
    require_term(&db, term_id).await?;
    let records = repository::list_for_member(&db, term_id, params.member_id)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(ApplicationListResponse {
        items: records.into_iter().map(ApplicationResponse::from).collect(),
    }))
}

/// 이 학기 강좌별 유효한 신청 인원. 학기가 없으면 404.
async fn list_application_counts(
    State(db): State<SqlitePool>,
    term_id: Result<Path<i64>, PathRejection>,
) -> Result<Json<ApplicationCountListResponse>, ApiError> {
    let Path(term_id) = term_id?;
    require_term(&db, term_id).await?;
    let counts = repository::count_by_course(&db, term_id)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(ApplicationCountListResponse {
        items: counts
            .into_iter()
            .map(ApplicationCountResponse::from)
            .collect(),
    }))
}

/// 회원 한 명의 신청 여러 건을 한 번에 저장한다(접수). 성공하면 201과 새 id들.
///
/// 규칙을 어기면 409 `RULES_BROKEN`과 어긴 규칙들(BR-A5). 사유를 적으면 받는다(BR-A6).
async fn submit_applications(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    term_id: Result<Path<i64>, PathRejection>,
    body: Result<Json<SubmitApplicationsRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    staff.require(Action::TakeApplications)?;
    let Path(term_id) = term_id?;
    let Json(request) = body?;
    if request.course_ids.is_empty() {
        return Err(field_error(
            "courseIds",
            "강좌를 하나 이상 골라 주세요".to_owned(),
        ));
    }
    let reason = parse_reason(request.override_reason.as_deref(), "overrideReason")?;
    if reason.is_some() {
        staff.require(Action::OverrideRules)?;
    }
    if request.priorities.iter().any(|p| !p.reason_ids.is_empty()) {
        staff.require(Action::AssignPriority)?;
    }
    require_term(&db, term_id).await?;

    let priorities: Vec<CoursePriorities<'_>> = request
        .priorities
        .iter()
        .map(|p| CoursePriorities {
            course_id: p.course_id,
            reason_ids: &p.reason_ids,
        })
        .collect();

    let new = NewApplications {
        term_id,
        member_id: request.member_id,
        course_ids: &request.course_ids,
        override_reason: reason.as_ref().map(Reason::as_str),
        staff_id: staff.id,
        priorities: &priorities,
    };
    let submitted = service::submit(&db, new, OffsetDateTime::now_utc()).await?;
    tracing::info!(
        staff_id = staff.id,
        term_id,
        member_id = request.member_id,
        count = submitted.len(),
        additional = submitted
            .iter()
            .filter(|item| item.additional.is_some())
            .count(),
        overridden = reason.is_some(),
        "신청 저장"
    );
    Ok((
        StatusCode::CREATED,
        Json(SubmitApplicationsResponse {
            items: submitted.into_iter().map(SubmittedResponse::from).collect(),
        }),
    ))
}

/// 접수됨 신청을 지우고 삭제 기록을 남긴다(BR-D1~D3). 성공하면 204(본문 없음).
///
/// 본문의 사유는 선택이다. 선발 뒤의 신청이면 409.
async fn delete_application(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
    body: Result<Json<DeleteApplicationRequest>, JsonRejection>,
) -> Result<StatusCode, ApiError> {
    staff.require(Action::TakeApplications)?;
    let Path(id) = id?;
    let Json(request) = body?;
    let reason = parse_reason(request.reason.as_deref(), "reason")?;
    service::delete(
        &db,
        id,
        staff.id,
        reason.as_ref().map(Reason::as_str),
        OffsetDateTime::now_utc(),
    )
    .await
    .map_err(|error| match error {
        DeleteApplicationError::NotFound => ApiError::ApplicationNotFound,
        DeleteApplicationError::NotSubmitted => ApiError::ApplicationNotDeletable,
        DeleteApplicationError::Db(error) => ApiError::internal(error),
    })?;
    tracing::info!(staff_id = staff.id, application_id = id, "신청 지움");
    Ok(StatusCode::NO_CONTENT)
}

/// 신청의 우선 배정 사유를 본문 목록으로 바꾼다(BR-S12, BR-S13). 바뀐 것은 기록한다(BR-S14).
///
/// 추첨한 강좌면 그 추첨이 취소되고 응답의 `lotteryCancelled`가 `true`다. 마감한 강좌면 409.
async fn set_priority_reasons(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
    body: Result<Json<SetPriorityReasonsRequest>, JsonRejection>,
) -> Result<Json<SetPriorityReasonsResponse>, ApiError> {
    staff.require(Action::AssignPriority)?;
    let Path(id) = id?;
    let Json(request) = body?;
    let result = application_priorities::set(
        &db,
        id,
        &request.reason_ids,
        staff.id,
        OffsetDateTime::now_utc(),
    )
    .await
    .map_err(|error| match error {
        SetPrioritiesError::NotFound => ApiError::ApplicationNotFound,
        SetPrioritiesError::NotChangeable => ApiError::ApplicationNotChangeable,
        SetPrioritiesError::Closed => ApiError::SelectionClosed,
        error @ SetPrioritiesError::ReasonNotFound(_) => {
            field_error("reasonIds", error.to_string())
        }
        SetPrioritiesError::Db(error) => ApiError::internal(error),
    })?;
    // BR-S15: 사유는 민감할 수 있어 로그에 남기지 않는다. 기록은 DB에 있다
    tracing::info!(
        staff_id = staff.id,
        application_id = id,
        changed = result.changed,
        lottery_cancelled = result.lottery_cancelled,
        "우선 배정 바꿈"
    );
    Ok(Json(SetPriorityReasonsResponse {
        lottery_cancelled: result.lottery_cancelled,
    }))
}

/// 사유 칸을 확인한다. 비었으면 `None`, 너무 길면 `field` 칸의 422.
fn parse_reason(input: Option<&str>, field: &'static str) -> Result<Option<Reason>, ApiError> {
    Reason::parse(input.unwrap_or("")).map_err(|error| field_error(field, error.to_string()))
}

impl From<SubmitError> for ApiError {
    fn from(error: SubmitError) -> Self {
        match error {
            SubmitError::MemberNotFound => ApiError::MemberNotFound,
            SubmitError::CourseNotFound(_) => ApiError::CourseNotFound,
            SubmitError::CourseDrawn(_) => ApiError::CourseDrawn,
            SubmitError::TermEnded(_) => ApiError::TermEnded,
            // 화면은 목록에서 고른 사유만, 추가 신청이 아닌 강좌에만 보낸다. 그 사이 사유가
            // 지워졌거나 강좌가 마감됐을 때만 온다
            error @ (SubmitError::PriorityForUnchosenCourse(_)
            | SubmitError::PriorityReasonNotFound(_)
            | SubmitError::PriorityForAdditional(_)) => {
                field_error("priorities", error.to_string())
            }
            SubmitError::RulesBroken(violations) => ApiError::RulesBroken(
                violations
                    .into_iter()
                    .map(RuleViolationResponse::from)
                    .collect(),
            ),
            SubmitError::Db(error) => ApiError::internal(error),
        }
    }
}
