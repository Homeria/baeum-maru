//! 신청 API의 요청·응답 JSON 모양과 DB·업무 규칙 타입 사이의 변환.

use baeum_domain::{ApplicationRule, RuleViolation};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::db::applications::{ApplicationRecord, CourseCount};
use crate::db::timestamp_text;

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
}

/// `DELETE /api/applications/{id}` 본문.
#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct DeleteApplicationRequest {
    /// 지우는 사유(BR-D3). 선택.
    pub reason: Option<String>,
}

/// `POST /api/terms/{termId}/applications` 응답. 새 신청 id, 고른 순서.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct SubmitApplicationsResponse {
    #[ts(type = "number[]")]
    pub ids: Vec<i64>,
}

/// 신청 규칙. DB와 같은 이름.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
#[expect(
    clippy::enum_variant_names,
    reason = "규칙 이름이 모두 '하지 않기'(no_…)라서 같은 앞말이 뜻을 가진다"
)]
pub enum ApplicationRuleCode {
    /// BR-A4
    NoDuplicate,
    /// BR-A1
    NoTimeOverlap,
    /// BR-A2
    NoSameSubject,
}

impl From<ApplicationRule> for ApplicationRuleCode {
    fn from(rule: ApplicationRule) -> Self {
        match rule {
            ApplicationRule::NoDuplicate => ApplicationRuleCode::NoDuplicate,
            ApplicationRule::NoTimeOverlap => ApplicationRuleCode::NoTimeOverlap,
            ApplicationRule::NoSameSubject => ApplicationRuleCode::NoSameSubject,
        }
    }
}

/// 신청하려는 강좌가 어긴 규칙 하나. 오류 응답의 `violations`에 담긴다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RuleViolationResponse {
    #[ts(type = "number")]
    pub course_id: i64,
    pub rule: ApplicationRuleCode,
    /// 부딪힌 강좌. 이미 신청한 강좌이거나 함께 고른 강좌.
    #[ts(type = "number")]
    pub other_course_id: i64,
    /// 사유를 적으면 받을 수 있는지(BR-A6, BR-A7).
    pub can_override: bool,
}

impl From<RuleViolation> for RuleViolationResponse {
    fn from(violation: RuleViolation) -> Self {
        RuleViolationResponse {
            course_id: violation.course_id,
            rule: violation.rule.into(),
            other_course_id: violation.other_course_id,
            can_override: violation.rule.can_override(),
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
