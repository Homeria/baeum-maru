//! `/api/terms`의 요청·응답 JSON 모양과 DB·업무 규칙 타입 사이의 변환.

use baeum_domain::TermInput;
use serde::{Deserialize, Serialize};
use time::Date;
use ts_rs::TS;

use super::repository::TermRecord;

/// `POST /api/terms`, `PATCH /api/terms/{id}` 본문. 확인 전이다.
#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TermRequest {
    pub name: String,
    /// YYYY-MM-DD. 비웠으면 `null`.
    #[ts(type = "string | null")]
    pub starts_on: Option<Date>,
    /// YYYY-MM-DD. 비웠으면 `null`.
    #[ts(type = "string | null")]
    pub ends_on: Option<Date>,
}

impl TermRequest {
    pub(super) fn as_input(&self) -> TermInput<'_> {
        TermInput {
            name: &self.name,
            starts_on: self.starts_on,
            ends_on: self.ends_on,
        }
    }
}

/// `POST /api/terms` 응답.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct CreateTermResponse {
    /// 새 학기의 DB id.
    #[ts(type = "number")]
    pub id: i64,
}

/// 학기 하나. 목록의 한 줄이자 `GET`·`PATCH /api/terms/{id}` 응답.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TermResponse {
    #[ts(type = "number")]
    pub id: i64,
    pub name: String,
    /// YYYY-MM-DD
    #[ts(type = "string")]
    pub starts_on: Date,
    /// YYYY-MM-DD
    #[ts(type = "string")]
    pub ends_on: Date,
}

impl From<TermRecord> for TermResponse {
    fn from(record: TermRecord) -> Self {
        TermResponse {
            id: record.id,
            name: record.name,
            starts_on: record.starts_on,
            ends_on: record.ends_on,
        }
    }
}

/// `GET /api/terms` 응답. 시작일이 늦은 학기부터. 학기는 적어서 쪽을 나누지 않는다.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct TermListResponse {
    pub items: Vec<TermResponse>,
}
