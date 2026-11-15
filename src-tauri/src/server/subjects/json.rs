//! `/api/subjects`의 요청·응답 JSON 모양과 DB·업무 규칙 타입 사이의 변환.

use baeum_domain::SubjectInput;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::db::subjects::SubjectRecord;

/// `POST /api/subjects`, `PATCH /api/subjects/{id}` 본문. 확인 전이라 글자 그대로다.
#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SubjectRequest {
    pub name: String,
    /// 선택. 비었거나 공백뿐이면 분류 없음.
    pub category: Option<String>,
}

impl SubjectRequest {
    pub(super) fn as_input(&self) -> SubjectInput<'_> {
        SubjectInput {
            name: &self.name,
            category: self.category.as_deref(),
        }
    }
}

/// `POST /api/subjects` 응답.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct CreateSubjectResponse {
    /// 새 과목의 DB id.
    #[ts(type = "number")]
    pub id: i64,
}

/// 과목 하나. 목록의 한 줄이자 `GET`·`PATCH /api/subjects/{id}` 응답.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SubjectResponse {
    #[ts(type = "number")]
    pub id: i64,
    pub name: String,
    /// 없으면 `null`.
    pub category: Option<String>,
}

impl From<SubjectRecord> for SubjectResponse {
    fn from(record: SubjectRecord) -> Self {
        SubjectResponse {
            id: record.id,
            name: record.name,
            category: record.category,
        }
    }
}

/// `GET /api/subjects` 응답. 과목은 적어서 쪽을 나누지 않는다.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct SubjectListResponse {
    pub items: Vec<SubjectResponse>,
}
