//! 기기 승인 API의 요청·응답 JSON 모양.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::db::devices::{DeviceRecord, OpenRequest, RequestStatus};
use crate::db::timestamp_text;

/// `POST /api/approval/request` 본문.
#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ApprovalRequest {
    /// 이 PC의 이름. 호스트 화면에 보인다. 1~40자.
    pub name: String,
}

/// `POST /api/approval/request` 응답.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ApprovalRequestResponse {
    /// 기기 증표(16진수 64자). 허락받은 뒤에만 쓸 수 있다. 앱이 보관한다.
    pub token: String,
    /// 승인 번호 6자리. 이 PC 화면에 보여 준다.
    pub code: String,
}

/// 승인 요청의 상태.
#[derive(Debug, Clone, Copy, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ApprovalStatusCode {
    Pending,
    Approved,
    Rejected,
    Expired,
}

impl From<RequestStatus> for ApprovalStatusCode {
    fn from(status: RequestStatus) -> Self {
        match status {
            RequestStatus::Pending => ApprovalStatusCode::Pending,
            RequestStatus::Approved => ApprovalStatusCode::Approved,
            RequestStatus::Rejected => ApprovalStatusCode::Rejected,
            RequestStatus::Expired => ApprovalStatusCode::Expired,
        }
    }
}

/// `GET /api/approval/status` 응답.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ApprovalStatusResponse {
    pub status: ApprovalStatusCode,
}

/// 호스트 화면의 기다리는 승인 요청 한 줄.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DeviceRequestResponse {
    #[ts(type = "number")]
    pub id: i64,
    pub name: String,
    pub code: String,
    /// UTC, ISO 8601.
    pub requested_at: String,
}

impl From<OpenRequest> for DeviceRequestResponse {
    fn from(request: OpenRequest) -> Self {
        DeviceRequestResponse {
            id: request.id,
            name: request.name,
            code: request.code,
            requested_at: timestamp_text(request.requested_at),
        }
    }
}

/// `GET /api/device-requests` 응답. 먼저 온 것부터.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DeviceRequestListResponse {
    pub items: Vec<DeviceRequestResponse>,
}

/// `POST /api/device-requests/{id}/approve` 본문.
#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ApproveDeviceRequest {
    /// 허락하는 관리자의 PIN.
    pub pin: String,
}

/// 승인한 기기 한 줄.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DeviceResponse {
    #[ts(type = "number")]
    pub id: i64,
    pub name: String,
    /// UTC, ISO 8601.
    pub approved_at: String,
    /// 허락한 직원의 이름.
    pub approved_by: String,
}

impl From<DeviceRecord> for DeviceResponse {
    fn from(record: DeviceRecord) -> Self {
        DeviceResponse {
            id: record.id,
            name: record.name,
            approved_at: timestamp_text(record.approved_at),
            approved_by: record.approved_by,
        }
    }
}

/// `GET /api/devices` 응답. 허락한 순서.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DeviceListResponse {
    pub items: Vec<DeviceResponse>,
}
