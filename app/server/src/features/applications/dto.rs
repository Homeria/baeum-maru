//! 신청 API의 요청·응답 JSON 모양과 DB·업무 규칙 타입 사이의 변환.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::repository::{ApplicationRecord, CourseCount};
use super::service::Submitted;
use crate::features::selection_results::dto::AdditionalOutcomeResponse;
use crate::http::error::ApplicationRuleCode;
use crate::infra::db::timestamp_text;

/// `GET /api/terms/{termId}/applications`의 `?` 뒤.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListApplicationsParams {
    pub member_id: i64,
}

/// `POST /api/terms/{termId}/applications` 본문. 확인 전이다.
#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SubmitApplicationsRequest {
    #[ts(type = "number")]
    pub member_id: i64,
    /// 고른 순서. 한 개 이상.
    #[ts(type = "number[]")]
    pub course_ids: Vec<i64>,
    /// 규칙 예외 사유(BR-A6). 규칙을 어기지 않았으면 `null`.
    pub override_reason: Option<String>,
    /// 고른 강좌마다 붙일 우선 배정 사유(BR-S12). 사유를 붙이지 않는 강좌는 빼도 된다.
    pub priorities: Vec<CoursePrioritiesRequest>,
}

/// 고른 강좌 하나에 붙일 우선 배정 사유들.
#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CoursePrioritiesRequest {
    #[ts(type = "number")]
    pub course_id: i64,
    #[ts(type = "number[]")]
    pub reason_ids: Vec<i64>,
}

/// `PUT /api/applications/{id}/priority-reasons` 본문. 이 목록으로 바꾼다. 빈 목록이면 모두 뗀다.
#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SetPriorityReasonsRequest {
    #[ts(type = "number[]")]
    pub reason_ids: Vec<i64>,
}

/// `PUT /api/applications/{id}/priority-reasons` 응답.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SetPriorityReasonsResponse {
    /// BR-S13: 추첨한 강좌라 추첨이 취소되고 추첨 전으로 돌아갔다.
    pub lottery_cancelled: bool,
}

/// `DELETE /api/applications/{id}` 본문.
#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct DeleteApplicationRequest {
    /// 지우는 사유(BR-D3). 선택.
    pub reason: Option<String>,
}

/// `POST /api/terms/{termId}/applications` 응답. 새 신청, 고른 순서.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct SubmitApplicationsResponse {
    pub items: Vec<SubmittedResponse>,
}

/// 저장한 신청 하나.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SubmittedResponse {
    #[ts(type = "number")]
    pub id: i64,
    #[ts(type = "number")]
    pub course_id: i64,
    /// 추가 신청(BR-X2)이면 그 결과. 보통 신청(접수됨)이면 `null`.
    pub additional: Option<AdditionalOutcomeResponse>,
}

impl From<Submitted> for SubmittedResponse {
    fn from(submitted: Submitted) -> Self {
        SubmittedResponse {
            id: submitted.id,
            course_id: submitted.course_id,
            additional: submitted.additional.map(AdditionalOutcomeResponse::from),
        }
    }
}

/// `GET /api/terms/{termId}/applications` 응답. 접수한 순서.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct ApplicationListResponse {
    pub items: Vec<ApplicationResponse>,
}

/// 회원의 신청 하나.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ApplicationResponse {
    #[ts(type = "number")]
    pub id: i64,
    #[ts(type = "number")]
    pub course_id: i64,
    /// UTC, ISO 8601
    pub submitted_at: String,
    /// 예외를 받은 규칙.
    pub overridden_rules: Vec<ApplicationRuleCode>,
    /// 붙은 우선 배정 사유 id. id 순서.
    #[ts(type = "number[]")]
    pub priority_reason_ids: Vec<i64>,
}

impl From<ApplicationRecord> for ApplicationResponse {
    fn from(record: ApplicationRecord) -> Self {
        ApplicationResponse {
            id: record.id,
            course_id: record.course_id,
            submitted_at: timestamp_text(record.submitted_at),
            overridden_rules: record
                .overridden_rules
                .into_iter()
                .map(ApplicationRuleCode::from)
                .collect(),
            priority_reason_ids: record.priority_reason_ids,
        }
    }
}

/// `GET /api/terms/{termId}/application-counts` 응답. 신청이 없는 강좌는 빠진다.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct ApplicationCountListResponse {
    pub items: Vec<ApplicationCountResponse>,
}

/// 강좌 하나의 유효한 신청 인원.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ApplicationCountResponse {
    #[ts(type = "number")]
    pub course_id: i64,
    #[ts(type = "number")]
    pub count: i64,
}

impl From<CourseCount> for ApplicationCountResponse {
    fn from(count: CourseCount) -> Self {
        ApplicationCountResponse {
            course_id: count.course_id,
            count: count.count,
        }
    }
}
