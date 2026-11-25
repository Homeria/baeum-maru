//! 선발 결과: 마감한 강좌 목록은 `/api/terms/{termId}/selection-results`,
//! 강좌 하나는 `/api/courses/{id}/selection-result`, 포기·연락 결과·바로잡기는
//! `/api/applications/{id}/withdrawal`, `/contacts`, `/withdrawal-correction`.
//!
//! 보기는 모든 역할이 한다. 대기자 연락처는 포기·연락을 남길 수 있는 역할(관리자·직원)에게만
//! 싣는다(BR-W2). 바로잡기는 관리자만 한다(BR-W8). 호스트 PC가 아니어도 된다.

mod json;

use axum::Json;
use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use baeum_domain::{Action, ContactResult, Reason};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use crate::db::selection_changes::{self, ChangeSelectionError};
use crate::db::selection_results::{self, CourseResultError};
use crate::features::auth::middleware::CurrentStaff;
use crate::features::terms::handler::require_term;
use crate::http::error::ApiError;
use crate::http::error::field_error;

use json::{
    ClosedCourseListResponse, ClosedCourseResponse, ContactRequest, CorrectWithdrawalRequest,
    CourseResultResponse, SelectedResponse, WaitingResponse, WithdrawRequest, WithdrawnResponse,
};

/// 학기에서 선발을 마감한 강좌들의 인원. 학기가 없으면 404.
pub async fn list_closed_courses(
    State(db): State<SqlitePool>,
    term_id: Result<Path<i64>, PathRejection>,
) -> Result<Json<ClosedCourseListResponse>, ApiError> {
    let Path(term_id) = term_id?;
    require_term(&db, term_id).await?;
    let courses = selection_results::closed_courses(&db, term_id)
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
pub async fn get_course_result(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
) -> Result<Json<CourseResultResponse>, ApiError> {
    let Path(id) = id?;
    let result = selection_results::course_result(&db, id)
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
pub async fn withdraw(
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
pub async fn record_contact(
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
pub async fn correct_withdrawal(
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

#[cfg(test)]
mod tests {
    use axum::Router;
    use serde_json::{Value, json};

    use super::*;
    use crate::db::test_support::{DrawnCourse, insert_drawn_course};
    use crate::server::test_support::{app, app_as, get, json_body, post};

    /// 정원 2명 강좌에 신청 5건을 추첨했다. `close`면 마감까지. 당첨 2명, 대기 3명.
    async fn drawn(db: &SqlitePool, close: bool) -> DrawnCourse {
        insert_drawn_course(db, 2, 5, close).await
    }

    async fn result(router: Router, course: &DrawnCourse) -> Value {
        let uri = format!("/api/courses/{}/selection-result", course.course_id);
        json_body(get(router, &uri).await).await
    }

    fn id(entry: &Value) -> i64 {
        entry["member"]["applicationId"].as_i64().unwrap()
    }

    async fn withdraw(router: Router, application_id: i64) -> StatusCode {
        let uri = format!("/api/applications/{application_id}/withdrawal");
        post(router, &uri, &json!({ "reason": null }).to_string())
            .await
            .status()
    }

    async fn contact(router: Router, application_id: i64, result: &str) -> (StatusCode, Value) {
        let uri = format!("/api/applications/{application_id}/contacts");
        let response = post(router, &uri, &json!({ "result": result }).to_string()).await;
        let status = response.status();
        let body = if status == StatusCode::NO_CONTENT {
            Value::Null
        } else {
            json_body(response).await
        };
        (status, body)
    }

    async fn correct(router: Router, application_id: i64, reason: &str) -> (StatusCode, Value) {
        let uri = format!("/api/applications/{application_id}/withdrawal-correction");
        let response = post(router, &uri, &json!({ "reason": reason }).to_string()).await;
        let status = response.status();
        let body = if status == StatusCode::NO_CONTENT {
            Value::Null
        } else {
            json_body(response).await
        };
        (status, body)
    }

    #[tokio::test]
    async fn lists_closed_courses_and_shows_result() {
        let (_dir, db, router) = app().await;
        let course = drawn(&db, true).await;

        let list_uri = format!("/api/terms/{}/selection-results", course.term_id);
        let list = json_body(get(router.clone(), &list_uri).await).await;
        let shown = result(router, &course).await;

        assert_eq!(
            list,
            json!({ "items": [{
                "courseId": course.course_id, "capacity": 2, "selected": 2,
                "waitlisted": 3, "openSeats": 0,
                // 대기 1~3번 다음
                "nextAdditional": { "kind": "waitlisted", "waitlistNumber": 4 }
            }] })
        );
        assert_eq!(shown["selected"].as_array().unwrap().len(), 2);
        assert_eq!(shown["waitlisted"][0]["waitlistNumber"], 1);
        assert!(shown["waitlisted"][0]["phoneNumber"].is_string());
        assert_eq!(shown["canRecord"], true);
        assert_eq!(shown["canCorrect"], true);
    }

    #[tokio::test]
    async fn br_w2_read_only_sees_result_without_phone_numbers() {
        let (_dir, db, router) = app_as("read_only").await;
        let course = drawn(&db, true).await;

        let shown = result(router.clone(), &course).await;
        let selected = id(&shown["selected"][0]);

        assert_eq!(shown["waitlisted"][0]["phoneNumber"], Value::Null);
        assert_eq!(shown["canRecord"], false);
        assert_eq!(withdraw(router, selected).await, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn br_c3_not_closed_course_is_conflict() {
        let (_dir, db, router) = app().await;
        let course = drawn(&db, false).await;
        let application_id: i64 = sqlx::query_scalar("SELECT MIN(id) FROM applications")
            .fetch_one(&db)
            .await
            .unwrap();

        let shown = result(router.clone(), &course).await;
        let status = withdraw(router, application_id).await;

        assert_eq!(shown["code"], "SELECTION_NOT_CLOSED");
        assert_eq!(status, StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn br_w1_w3_withdrawal_opens_seat_and_turn_accepts() {
        let (_dir, db, router) = app_as("staff").await;
        let course = drawn(&db, true).await;
        let before = result(router.clone(), &course).await;
        let first = id(&before["waitlisted"][0]);
        let second = id(&before["waitlisted"][1]);

        let (full, body) = contact(router.clone(), first, "accepted").await;
        assert_eq!(full, StatusCode::CONFLICT);
        assert_eq!(body["code"], "NO_OPEN_SEAT");

        let selected = id(&before["selected"][0]);
        assert_eq!(
            withdraw(router.clone(), selected).await,
            StatusCode::NO_CONTENT
        );
        let opened = result(router.clone(), &course).await;
        assert_eq!(opened["openSeats"], 1);
        assert_eq!(opened["waitlisted"][0]["contactTurn"], true);

        let (unreachable, _) = contact(router.clone(), first, "unreachable").await;
        let (accepted, _) = contact(router.clone(), second, "accepted").await;
        let (wrong, body) = contact(router.clone(), second, "declined").await;

        assert_eq!(unreachable, StatusCode::NO_CONTENT);
        assert_eq!(accepted, StatusCode::NO_CONTENT);
        assert_eq!(wrong, StatusCode::CONFLICT);
        assert_eq!(body["code"], "SELECTION_CHANGE_NOT_ALLOWED");
        let after = result(router, &course).await;
        assert_eq!(after["openSeats"], 0);
        assert_eq!(after["waitlisted"][0]["lastContact"], "unreachable");
    }

    #[tokio::test]
    async fn br_w8_only_admin_corrects_with_reason() {
        let (_dir, db, admin) = app().await;
        let course = drawn(&db, true).await;
        let waiting = id(&result(admin.clone(), &course).await["waitlisted"][0]);
        assert_eq!(
            withdraw(admin.clone(), waiting).await,
            StatusCode::NO_CONTENT
        );

        let (blank, body) = correct(admin.clone(), waiting, "  ").await;
        let (fixed, _) = correct(admin.clone(), waiting, "다른 회원을 잘못 누름").await;

        assert_eq!(blank, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body["fields"][0]["field"], "reason");
        assert_eq!(fixed, StatusCode::NO_CONTENT);
        assert_eq!(id(&result(admin, &course).await["waitlisted"][0]), waiting);
    }

    #[tokio::test]
    async fn br_w8_staff_cannot_correct() {
        let (_dir, db, router) = app_as("staff").await;
        let course = drawn(&db, true).await;
        let selected = id(&result(router.clone(), &course).await["selected"][0]);
        withdraw(router.clone(), selected).await;

        let (status, _) = correct(router.clone(), selected, "잘못 누름").await;

        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(result(router, &course).await["canCorrect"], false);
    }
}
