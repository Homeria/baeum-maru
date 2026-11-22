//! `/api/priority-reasons`의 요청·응답 JSON 모양과 DB·업무 규칙 타입 사이의 변환.

use baeum_domain::PriorityMethod;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::db::priority_reasons::PriorityReasonRecord;

/// 배정 방식(BR-S11). 지금은 자리 보장뿐이다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PriorityMethodCode {
    Guaranteed,
}

impl From<PriorityMethodCode> for PriorityMethod {
    fn from(code: PriorityMethodCode) -> Self {
        match code {
            PriorityMethodCode::Guaranteed => PriorityMethod::Guaranteed,
        }
    }
}

impl From<PriorityMethod> for PriorityMethodCode {
    fn from(method: PriorityMethod) -> Self {
        match method {
            PriorityMethod::Guaranteed => PriorityMethodCode::Guaranteed,
        }
    }
}

/// `POST /api/priority-reasons`, `PATCH /api/priority-reasons/{id}` 본문.
#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PriorityReasonRequest {
    pub name: String,
    pub method: PriorityMethodCode,
}

/// `POST /api/priority-reasons` 응답.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct CreatePriorityReasonResponse {
    /// 새 사유의 DB id.
    #[ts(type = "number")]
    pub id: i64,
}

/// 사유 하나. 목록의 한 줄이자 `PATCH` 응답.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PriorityReasonResponse {
    #[ts(type = "number")]
    pub id: i64,
    pub name: String,
    pub method: PriorityMethodCode,
}

impl From<PriorityReasonRecord> for PriorityReasonResponse {
    fn from(record: PriorityReasonRecord) -> Self {
        PriorityReasonResponse {
            id: record.id,
            name: record.name,
            method: record.method.into(),
        }
    }
}

/// `GET /api/priority-reasons` 응답. 사유는 적어서 쪽을 나누지 않는다.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct PriorityReasonListResponse {
    pub items: Vec<PriorityReasonResponse>,
}
