//! 선발 결과 API의 요청·응답 JSON 모양과 DB·업무 규칙 타입 사이의 변환.

use baeum_domain::{ContactResult, WithdrawnFrom};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::db::selection_results::{ClosedCourse, ResultMember};
use crate::server::applications::AdditionalOutcomeResponse;

/// `GET /api/terms/{termId}/selection-results` 응답. 선발을 마감한 강좌만, 강좌 id 순.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct ClosedCourseListResponse {
    pub items: Vec<ClosedCourseResponse>,
}

/// 마감한 강좌 하나의 인원.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ClosedCourseResponse {
    #[ts(type = "number")]
    pub course_id: i64,
    pub capacity: u16,
    #[ts(type = "number")]
    pub selected: usize,
    #[ts(type = "number")]
    pub waitlisted: usize,
    /// 정원 - 당첨(BR-W1).
    #[ts(type = "number")]
    pub open_seats: usize,
    /// 지금 추가 신청을 하나 받으면 어떻게 되는지(BR-X2). 신청 화면이 미리 보여 준다.
    pub next_additional: AdditionalOutcomeResponse,
}

impl From<ClosedCourse> for ClosedCourseResponse {
    fn from(course: ClosedCourse) -> Self {
        ClosedCourseResponse {
            course_id: course.course_id,
            capacity: course.capacity.value(),
            selected: course.selected,
            waitlisted: course.waitlisted,
            open_seats: course.open_seats,
            next_additional: course.next_additional.into(),
        }
    }
}

/// 대기자에게 연락한 결과(BR-W2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ContactResultCode {
    Accepted,
    Declined,
    Unreachable,
}

impl From<ContactResultCode> for ContactResult {
    fn from(code: ContactResultCode) -> Self {
        match code {
            ContactResultCode::Accepted => ContactResult::Accepted,
            ContactResultCode::Declined => ContactResult::Declined,
            ContactResultCode::Unreachable => ContactResult::Unreachable,
        }
    }
}

impl From<ContactResult> for ContactResultCode {
    fn from(result: ContactResult) -> Self {
        match result {
            ContactResult::Accepted => ContactResultCode::Accepted,
            ContactResult::Declined => ContactResultCode::Declined,
            ContactResult::Unreachable => ContactResultCode::Unreachable,
        }
    }
}

/// 포기하기 전 상태(BR-W8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum WithdrawnFromCode {
    Selected,
    Waitlisted,
}

impl From<WithdrawnFrom> for WithdrawnFromCode {
    fn from(from: WithdrawnFrom) -> Self {
        match from {
            WithdrawnFrom::Selected => WithdrawnFromCode::Selected,
            WithdrawnFrom::Waitlisted => WithdrawnFromCode::Waitlisted,
        }
    }
}

/// `GET /api/courses/{id}/selection-result` 응답.
// 회원 정보가 담겨 Debug를 만들지 않는다 (로그에 찍히지 않게)
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CourseResultResponse {
    /// 지금 정원.
    pub capacity: u16,
    #[ts(type = "number")]
    pub open_seats: usize,
    /// 자리 보장 다음에 뽑힌 순서.
    pub selected: Vec<SelectedResponse>,
    /// 대기 순번 순서.
    pub waitlisted: Vec<WaitingResponse>,
    pub withdrawn: Vec<WithdrawnResponse>,
    /// 이 직원이 포기와 연락 결과를 남길 수 있는지(BR-C3, BR-W2).
    pub can_record: bool,
    /// 이 직원이 잘못 처리 바로잡기를 할 수 있는지(BR-W8). 관리자만.
    pub can_correct: bool,
}

/// 결과 목록의 회원 한 명. 화면은 `이름(회원번호)`로 보여 준다.
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ResultMemberResponse {
    #[ts(type = "number")]
    pub application_id: i64,
    pub member_name: String,
    pub member_no: String,
}

impl From<ResultMember> for ResultMemberResponse {
    fn from(member: ResultMember) -> Self {
        ResultMemberResponse {
            application_id: member.application_id,
            member_name: member.member_name.into_inner(),
            member_no: member.member_no.into_inner(),
        }
    }
}

/// 당첨자 한 명.
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SelectedResponse {
    pub member: ResultMemberResponse,
    /// 자리 보장으로 당첨(BR-S4).
    pub guaranteed: bool,
    /// 대기에서 승격했으면 그때의 대기 순번, 아니면 `null`.
    #[ts(type = "number | null")]
    pub waitlist_number: Option<usize>,
}

/// 대기자 한 명.
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct WaitingResponse {
    pub member: ResultMemberResponse,
    /// 연락처(BR-W2). 포기·연락을 남길 수 없는 역할이면 `null`.
    pub phone_number: Option<String>,
    #[ts(type = "number")]
    pub waitlist_number: usize,
    /// 마지막 연락 결과. 연락한 적이 없으면 `null`.
    pub last_contact: Option<ContactResultCode>,
    /// 지금 연락할 차례(BR-W1).
    pub contact_turn: bool,
}

/// 포기한 사람 한 명.
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct WithdrawnResponse {
    pub member: ResultMemberResponse,
    pub withdrawn_from: WithdrawnFromCode,
    /// 추첨 때 받은 대기 순번. 당첨에서 바로 포기했으면 `null`.
    #[ts(type = "number | null")]
    pub waitlist_number: Option<usize>,
}

/// `POST /api/applications/{id}/withdrawal` 본문.
#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct WithdrawRequest {
    /// 포기 사유(BR-C3). 선택.
    pub reason: Option<String>,
}

/// `POST /api/applications/{id}/contacts` 본문.
#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct ContactRequest {
    pub result: ContactResultCode,
}

/// `POST /api/applications/{id}/withdrawal-correction` 본문.
#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct CorrectWithdrawalRequest {
    /// 바로잡는 사유(BR-W8). 필수.
    pub reason: String,
}
