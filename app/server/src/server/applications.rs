//! 신청: 회원별 목록·신청은 `/api/terms/{termId}/applications`,
//! 강좌별 인원은 `/api/terms/{termId}/application-counts`, 지우기는 `/api/applications/{id}`,
//! 우선 배정 사유 바꾸기는 `/api/applications/{id}/priority-reasons`.

mod json;

use axum::Json;
use axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use baeum_domain::{Action, Reason};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use crate::db::application_priorities::{self, SetPrioritiesError};
use crate::db::applications::{
    self, CoursePriorities, DeleteApplicationError, NewApplications, SubmitError,
};
use crate::features::auth::middleware::CurrentStaff;
use crate::features::terms::handler::require_term;
use crate::http::error::{ApiError, RuleViolationResponse, field_error};

pub use json::AdditionalOutcomeResponse;
use json::{
    ApplicationCountListResponse, ApplicationCountResponse, ApplicationListResponse,
    ApplicationResponse, DeleteApplicationRequest, ListApplicationsParams,
    SetPriorityReasonsRequest, SetPriorityReasonsResponse, SubmitApplicationsRequest,
    SubmitApplicationsResponse, SubmittedResponse,
};

/// 회원이 이 학기에 낸 유효한 신청. 학기가 없으면 404.
pub async fn list_applications(
    State(db): State<SqlitePool>,
    term_id: Result<Path<i64>, PathRejection>,
    params: Result<Query<ListApplicationsParams>, QueryRejection>,
) -> Result<Json<ApplicationListResponse>, ApiError> {
    let Path(term_id) = term_id?;
    let Query(params) = params?;
    require_term(&db, term_id).await?;
    let records = applications::list_for_member(&db, term_id, params.member_id)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(ApplicationListResponse {
        items: records.into_iter().map(ApplicationResponse::from).collect(),
    }))
}

/// 이 학기 강좌별 유효한 신청 인원. 학기가 없으면 404.
pub async fn list_application_counts(
    State(db): State<SqlitePool>,
    term_id: Result<Path<i64>, PathRejection>,
) -> Result<Json<ApplicationCountListResponse>, ApiError> {
    let Path(term_id) = term_id?;
    require_term(&db, term_id).await?;
    let counts = applications::count_by_course(&db, term_id)
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
pub async fn submit_applications(
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
    let submitted = applications::submit(&db, new, OffsetDateTime::now_utc()).await?;
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
pub async fn delete_application(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
    body: Result<Json<DeleteApplicationRequest>, JsonRejection>,
) -> Result<StatusCode, ApiError> {
    staff.require(Action::TakeApplications)?;
    let Path(id) = id?;
    let Json(request) = body?;
    let reason = parse_reason(request.reason.as_deref(), "reason")?;
    applications::delete(
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
pub async fn set_priority_reasons(
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

#[cfg(test)]
mod tests {
    use axum::Router;
    use serde_json::{Value, json};

    use super::*;
    use crate::db::test_support::{
        DrawnCourse, insert_application, insert_course, insert_drawn_course, insert_lottery_run,
        insert_member, insert_priority_reason, insert_schedule, insert_staff, insert_subject,
        insert_term,
    };
    use crate::server::test_support::{app, app_as, delete_with_body, get, json_body, post, put};

    /// 학기, 회원과 강좌 셋. 요가 초급(월 10시), 요가 중급(화 10시), 한글교실(월 10시).
    ///
    /// 강좌를 만들 수 없는 역할도 시험하려고 DB에 바로 넣는다.
    struct Setup {
        term_id: i64,
        member_id: i64,
        yoga_basic: i64,
        yoga_middle: i64,
        hangul: i64,
    }

    async fn setup(db: &SqlitePool) -> Setup {
        let term_id = insert_term(db, "2027년 1학기", "2027-03-02", "2027-06-30")
            .await
            .unwrap();
        let yoga = insert_subject(db, "요가", "요가", None).await.unwrap();
        let hangul_subject = insert_subject(db, "한글교실", "한글교실", None)
            .await
            .unwrap();
        let course = async |subject_id, level, weekday| {
            let id = insert_course(db, term_id, subject_id, level, None)
                .await
                .unwrap();
            insert_schedule(db, id, weekday, 600, 660, None)
                .await
                .unwrap();
            id
        };
        Setup {
            term_id,
            member_id: insert_member(db, "00001").await.unwrap(),
            yoga_basic: course(yoga, Some("초급"), 1).await,
            yoga_middle: course(yoga, Some("중급"), 2).await,
            hangul: course(hangul_subject, None, 1).await,
        }
    }

    impl Setup {
        fn uri(&self) -> String {
            format!("/api/terms/{}/applications", self.term_id)
        }

        fn body(&self, course_ids: &[i64], reason: Option<&str>) -> String {
            json!({
                "memberId": self.member_id,
                "courseIds": course_ids,
                "overrideReason": reason,
                "priorities": [],
            })
            .to_string()
        }

        /// `course_id` 하나를 고르고 우선 배정 사유 `reason_ids`를 붙인 본문.
        fn body_with_priorities(&self, course_id: i64, reason_ids: &[i64]) -> String {
            json!({
                "memberId": self.member_id,
                "courseIds": [course_id],
                "overrideReason": null,
                "priorities": [{ "courseId": course_id, "reasonIds": reason_ids }],
            })
            .to_string()
        }

        async fn list(&self, router: Router) -> Value {
            let uri = format!("{}?memberId={}", self.uri(), self.member_id);
            json_body(get(router, &uri).await).await
        }
    }

    #[tokio::test]
    async fn submits_and_lists_applications_and_counts() {
        let (_dir, db, router) = app().await;
        let s = setup(&db).await;

        let response = post(
            router.clone(),
            &s.uri(),
            &s.body(&[s.yoga_basic, s.yoga_middle], Some("강사 허락")),
        )
        .await;

        assert_eq!(response.status(), StatusCode::CREATED);
        let items = json_body(response).await["items"].clone();
        let ids = json!(
            items
                .as_array()
                .unwrap()
                .iter()
                .map(|item| item["id"].clone())
                .collect::<Vec<_>>()
        );
        let list = s.list(router.clone()).await;
        assert_eq!(list["items"][0]["id"], ids[0]);
        assert_eq!(list["items"][0]["courseId"], s.yoga_basic);
        assert_eq!(list["items"][0]["overriddenRules"], json!([]));
        assert_eq!(
            list["items"][1]["overriddenRules"],
            json!(["no_same_subject"])
        );
        let counts_uri = format!("/api/terms/{}/application-counts", s.term_id);
        assert_eq!(
            json_body(get(router, &counts_uri).await).await,
            json!({ "items": [
                { "courseId": s.yoga_basic, "count": 1 },
                { "courseId": s.yoga_middle, "count": 1 },
            ]})
        );
    }

    #[tokio::test]
    async fn br_a5_broken_rules_are_conflict_with_violations() {
        let (_dir, db, router) = app().await;
        let s = setup(&db).await;
        let first = post(router.clone(), &s.uri(), &s.body(&[s.yoga_basic], None)).await;
        assert_eq!(first.status(), StatusCode::CREATED);

        let response = post(
            router.clone(),
            &s.uri(),
            &s.body(&[s.hangul, s.yoga_basic], None),
        )
        .await;

        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = json_body(response).await;
        assert_eq!(body["code"], "RULES_BROKEN");
        assert_eq!(body["rule"], "BR-A5");
        assert_eq!(
            body["violations"],
            json!([
                { "courseId": s.hangul, "rule": "no_time_overlap",
                  "otherCourseId": s.yoga_basic, "canOverride": true },
                { "courseId": s.yoga_basic, "rule": "no_duplicate",
                  "otherCourseId": s.yoga_basic, "canOverride": false },
                { "courseId": s.yoga_basic, "rule": "no_time_overlap",
                  "otherCourseId": s.hangul, "canOverride": true },
            ])
        );
        assert_eq!(s.list(router).await["items"].as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn br_l3_drawn_course_is_conflict() {
        let (_dir, db, router) = app().await;
        let s = setup(&db).await;
        let staff_id = insert_staff(&db, "직원2", "staff", None).await.unwrap();
        insert_lottery_run(&db, s.yoga_basic, staff_id)
            .await
            .unwrap();

        let response = post(router, &s.uri(), &s.body(&[s.yoga_basic], None)).await;

        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = json_body(response).await;
        assert_eq!(body["code"], "COURSE_DRAWN");
        assert_eq!(body["rule"], "BR-L3");
    }

    #[tokio::test]
    async fn br_a6_temp_staff_cannot_override_but_can_submit() {
        let (_dir, db, router) = app_as("temp_staff").await;
        let s = setup(&db).await;

        let overridden = post(
            router.clone(),
            &s.uri(),
            &s.body(&[s.yoga_basic, s.hangul], Some("강사 허락")),
        )
        .await;
        let plain = post(router, &s.uri(), &s.body(&[s.yoga_basic], None)).await;

        assert_eq!(overridden.status(), StatusCode::FORBIDDEN);
        assert_eq!(plain.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn read_only_cannot_submit() {
        let (_dir, db, router) = app_as("read_only").await;
        let s = setup(&db).await;

        let response = post(router, &s.uri(), &s.body(&[s.yoga_basic], None)).await;

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn empty_courses_and_long_reason_are_field_errors() {
        let (_dir, db, router) = app().await;
        let s = setup(&db).await;

        let empty = post(router.clone(), &s.uri(), &s.body(&[], None)).await;
        let long_reason = "가".repeat(101);
        let long = post(
            router,
            &s.uri(),
            &s.body(&[s.yoga_basic], Some(&long_reason)),
        )
        .await;

        assert_eq!(empty.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(json_body(empty).await["fields"][0]["field"], "courseIds");
        assert_eq!(long.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            json_body(long).await["fields"][0]["field"],
            "overrideReason"
        );
    }

    #[tokio::test]
    async fn unknown_term_member_and_course_are_not_found() {
        let (_dir, db, router) = app().await;
        let s = setup(&db).await;
        let unknown_member = json!({ "memberId": 999, "courseIds": [s.yoga_basic], "overrideReason": null, "priorities": [] });

        let term = post(
            router.clone(),
            "/api/terms/999/applications",
            &s.body(&[s.yoga_basic], None),
        )
        .await;
        let member = post(router.clone(), &s.uri(), &unknown_member.to_string()).await;
        let course = post(router, &s.uri(), &s.body(&[999], None)).await;

        assert_eq!(json_body(term).await["code"], "TERM_NOT_FOUND");
        assert_eq!(json_body(member).await["code"], "MEMBER_NOT_FOUND");
        assert_eq!(json_body(course).await["code"], "COURSE_NOT_FOUND");
    }

    async fn delete_with(router: Router, uri: &str, reason: Option<&str>) -> StatusCode {
        let body = json!({ "reason": reason }).to_string();
        delete_with_body(router, uri, &body).await.status()
    }

    #[tokio::test]
    async fn br_d1_d3_deletes_submitted_application_with_record() {
        let (_dir, db, router) = app_as("temp_staff").await;
        let s = setup(&db).await;
        let response = post(router.clone(), &s.uri(), &s.body(&[s.yoga_basic], None)).await;
        let id = json_body(response).await["items"][0]["id"]
            .as_i64()
            .unwrap();
        let uri = format!("/api/applications/{id}");

        let status = delete_with(router.clone(), &uri, Some(" 잘못 고름 ")).await;

        assert_eq!(status, StatusCode::NO_CONTENT);
        assert_eq!(s.list(router.clone()).await["items"], json!([]));
        let reason: Option<String> = sqlx::query_scalar("SELECT reason FROM application_deletions")
            .fetch_one(&db)
            .await
            .unwrap();
        assert_eq!(reason.as_deref(), Some("잘못 고름"));
        assert_eq!(delete_with(router, &uri, None).await, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn br_d1_selected_application_is_not_deletable() {
        let (_dir, db, router) = app().await;
        let s = setup(&db).await;
        let staff_id: i64 = sqlx::query_scalar("SELECT id FROM staff")
            .fetch_one(&db)
            .await
            .unwrap();
        let id = insert_application(&db, s.member_id, s.yoga_basic, "selected", staff_id)
            .await
            .unwrap();

        let status = delete_with(router, &format!("/api/applications/{id}"), None).await;

        assert_eq!(status, StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn read_only_cannot_delete() {
        let (_dir, db, router) = app_as("read_only").await;
        let s = setup(&db).await;
        let staff_id: i64 = sqlx::query_scalar("SELECT id FROM staff")
            .fetch_one(&db)
            .await
            .unwrap();
        let id = insert_application(&db, s.member_id, s.yoga_basic, "submitted", staff_id)
            .await
            .unwrap();

        let status = delete_with(router, &format!("/api/applications/{id}"), None).await;

        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    fn priority_uri(application_id: i64) -> String {
        format!("/api/applications/{application_id}/priority-reasons")
    }

    fn reason_ids(ids: &[i64]) -> String {
        json!({ "reasonIds": ids }).to_string()
    }

    #[tokio::test]
    async fn br_s12_submits_with_priorities_and_lists_them() {
        let (_dir, db, router) = app().await;
        let s = setup(&db).await;
        let leader = insert_priority_reason(&db, "반장").await.unwrap();
        let veteran = insert_priority_reason(&db, "국가유공자").await.unwrap();

        let response = post(
            router.clone(),
            &s.uri(),
            &s.body_with_priorities(s.yoga_basic, &[veteran, leader]),
        )
        .await;

        assert_eq!(response.status(), StatusCode::CREATED);
        let list = s.list(router).await;
        let mut want = [leader, veteran];
        want.sort_unstable();
        assert_eq!(list["items"][0]["priorityReasonIds"], json!(want));
    }

    #[tokio::test]
    async fn br_s12_temp_staff_cannot_attach_priority() {
        let (_dir, db, router) = app_as("temp_staff").await;
        let s = setup(&db).await;
        let leader = insert_priority_reason(&db, "반장").await.unwrap();

        let with_reason = post(
            router.clone(),
            &s.uri(),
            &s.body_with_priorities(s.yoga_basic, &[leader]),
        )
        .await;
        let without = post(
            router.clone(),
            &s.uri(),
            &s.body_with_priorities(s.yoga_basic, &[]),
        )
        .await;
        let id = json_body(without).await["items"][0]["id"].as_i64().unwrap();
        let changed = put(router, &priority_uri(id), &reason_ids(&[leader])).await;

        assert_eq!(with_reason.status(), StatusCode::FORBIDDEN);
        assert_eq!(changed.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn br_s13_changes_priorities_of_existing_application() {
        let (_dir, db, router) = app().await;
        let s = setup(&db).await;
        let leader = insert_priority_reason(&db, "반장").await.unwrap();
        let response = post(router.clone(), &s.uri(), &s.body(&[s.yoga_basic], None)).await;
        let id = json_body(response).await["items"][0]["id"]
            .as_i64()
            .unwrap();

        let attached = put(router.clone(), &priority_uri(id), &reason_ids(&[leader])).await;

        assert_eq!(attached.status(), StatusCode::OK);
        assert_eq!(
            json_body(attached).await,
            json!({ "lotteryCancelled": false })
        );
        assert_eq!(
            s.list(router.clone()).await["items"][0]["priorityReasonIds"],
            json!([leader])
        );

        let removed = put(router.clone(), &priority_uri(id), &reason_ids(&[])).await;

        assert_eq!(removed.status(), StatusCode::OK);
        assert_eq!(
            s.list(router).await["items"][0]["priorityReasonIds"],
            json!([])
        );
    }

    #[tokio::test]
    async fn br_s13_unknown_reason_and_application_are_refused() {
        let (_dir, db, router) = app().await;
        let s = setup(&db).await;
        let response = post(router.clone(), &s.uri(), &s.body(&[s.yoga_basic], None)).await;
        let id = json_body(response).await["items"][0]["id"]
            .as_i64()
            .unwrap();

        let unknown_reason = put(router.clone(), &priority_uri(id), &reason_ids(&[999])).await;
        let unknown_application = put(router, &priority_uri(999), &reason_ids(&[])).await;

        assert_eq!(unknown_reason.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            json_body(unknown_reason).await["fields"][0]["field"],
            "reasonIds"
        );
        assert_eq!(
            json_body(unknown_application).await["code"],
            "APPLICATION_NOT_FOUND"
        );
    }

    /// 정원 2명, 신청 3명으로 추첨하고 마감한 강좌. 새 회원 하나가 추가 신청할 주소와 본문.
    async fn closed_course(db: &SqlitePool) -> (DrawnCourse, String, String) {
        let course = insert_drawn_course(db, 2, 3, true).await;
        let member_id = insert_member(db, "10001").await.unwrap();
        let uri = format!("/api/terms/{}/applications", course.term_id);
        let body = json!({
            "memberId": member_id,
            "courseIds": [course.course_id],
            "overrideReason": null,
            "priorities": [],
        })
        .to_string();
        (course, uri, body)
    }

    #[tokio::test]
    async fn br_x2_additional_application_tells_its_outcome() {
        let (_dir, db, router) = app().await;
        let (course, uri, body) = closed_course(&db).await;

        let response = post(router, &uri, &body).await;

        assert_eq!(response.status(), StatusCode::CREATED);
        let item = json_body(response).await["items"][0].clone();
        assert_eq!(item["courseId"], course.course_id);
        // 대기 1번 다음
        assert_eq!(
            item["additional"],
            json!({ "kind": "waitlisted", "waitlistNumber": 2 })
        );
    }

    #[tokio::test]
    async fn br_x1_ended_term_takes_no_additional_application() {
        let (_dir, db, router) = app().await;
        let (_course, uri, body) = closed_course(&db).await;
        sqlx::query("UPDATE terms SET starts_on = '2020-03-02', ends_on = '2020-06-30'")
            .execute(&db)
            .await
            .unwrap();

        let response = post(router, &uri, &body).await;

        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = json_body(response).await;
        assert_eq!(body["code"], "TERM_ENDED");
        assert_eq!(body["rule"], "BR-X1");
    }
}
