//! `/api/access-logs`의 요청·응답 JSON 모양.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::repository::{AccessEvent, AccessLogRecord};
use crate::infra::db::timestamp_text;

/// `GET /api/access-logs`의 `?` 뒤 값.
#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AccessLogParams {
    /// 1부터. 없으면 1.
    pub page: Option<u32>,
    /// 1~100. 없으면 50.
    pub page_size: Option<u32>,
}

/// 접속 기록에 남은 일. BR-K13
#[derive(Debug, Clone, Copy, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum AccessEventCode {
    Login,
    LoginFailed,
    Logout,
    UnlockFailed,
    RecoveryUsed,
    Restore,
    DeviceApproved,
    DeviceRejected,
    DeviceRevoked,
    DeviceApprovalFailed,
    RunModeChanged,
    Handover,
    HandoverCanceled,
    TakeOver,
}

impl From<AccessEvent> for AccessEventCode {
    fn from(event: AccessEvent) -> Self {
        match event {
            AccessEvent::Login => AccessEventCode::Login,
            AccessEvent::LoginFailed => AccessEventCode::LoginFailed,
            AccessEvent::Logout => AccessEventCode::Logout,
            AccessEvent::UnlockFailed => AccessEventCode::UnlockFailed,
            AccessEvent::RecoveryUsed => AccessEventCode::RecoveryUsed,
            AccessEvent::Restore => AccessEventCode::Restore,
            AccessEvent::DeviceApproved => AccessEventCode::DeviceApproved,
            AccessEvent::DeviceRejected => AccessEventCode::DeviceRejected,
            AccessEvent::DeviceRevoked => AccessEventCode::DeviceRevoked,
            AccessEvent::DeviceApprovalFailed => AccessEventCode::DeviceApprovalFailed,
            AccessEvent::RunModeChanged => AccessEventCode::RunModeChanged,
            AccessEvent::Handover => AccessEventCode::Handover,
            AccessEvent::HandoverCanceled => AccessEventCode::HandoverCanceled,
            AccessEvent::TakeOver => AccessEventCode::TakeOver,
        }
    }
}

/// 접속 기록 한 줄.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AccessLogResponse {
    #[ts(type = "number")]
    pub id: i64,
    /// UTC, ISO 8601. 예: `2026-11-15T01:02:03.000Z`
    pub occurred_at: String,
    /// 고른 직원이 없는 실패면 `null`.
    #[ts(type = "number | null")]
    pub staff_id: Option<i64>,
    pub staff_name: Option<String>,
    /// 어느 기기의 일인지. 호스트 PC 자신이면 `null`.
    pub device_name: Option<String>,
    pub event: AccessEventCode,
}

impl From<AccessLogRecord> for AccessLogResponse {
    fn from(record: AccessLogRecord) -> Self {
        AccessLogResponse {
            id: record.id,
            occurred_at: timestamp_text(record.occurred_at),
            staff_id: record.staff_id,
            staff_name: record.staff_name,
            device_name: record.device_name,
            event: record.event.into(),
        }
    }
}

/// `GET /api/access-logs` 응답. 최근 것부터.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AccessLogListResponse {
    pub items: Vec<AccessLogResponse>,
    /// 전체 줄 수.
    #[ts(type = "number")]
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
}
