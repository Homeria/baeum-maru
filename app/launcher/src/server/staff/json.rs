//! `/api/staff`의 요청·응답 JSON 모양과 DB·업무 규칙 타입 사이의 변환.
//!
//! PIN이 들어 있는 요청은 `Debug`를 직접 써서 글자를 가린다.

use baeum_domain::{Role, StaffInput, is_account_locked};
use serde::{Deserialize, Serialize};
use time::{Date, OffsetDateTime};
use ts_rs::TS;

use crate::db::staff::StaffRecord;

/// 역할. BR-K3
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum RoleCode {
    Admin,
    Staff,
    TempStaff,
    ReadOnly,
}

impl From<Role> for RoleCode {
    fn from(role: Role) -> Self {
        match role {
            Role::Admin => RoleCode::Admin,
            Role::Staff => RoleCode::Staff,
            Role::TempStaff => RoleCode::TempStaff,
            Role::ReadOnly => RoleCode::ReadOnly,
        }
    }
}

impl From<RoleCode> for Role {
    fn from(code: RoleCode) -> Self {
        match code {
            RoleCode::Admin => Role::Admin,
            RoleCode::Staff => Role::Staff,
            RoleCode::TempStaff => Role::TempStaff,
            RoleCode::ReadOnly => Role::ReadOnly,
        }
    }
}

/// `POST /api/staff` 본문. 확인 전이라 글자 그대로다.
#[derive(Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CreateStaffRequest {
    pub name: String,
    pub role: RoleCode,
    /// 마지막 날. 임시 직원은 필수.
    #[ts(type = "string | null")]
    pub valid_until: Option<Date>,
    /// 숫자 4~8자리, 관리자는 6자리 이상.
    pub pin: String,
}

impl CreateStaffRequest {
    pub(super) fn as_input(&self) -> StaffInput<'_> {
        StaffInput {
            name: &self.name,
            role: self.role.into(),
            valid_until: self.valid_until,
        }
    }
}

impl std::fmt::Debug for CreateStaffRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CreateStaffRequest")
            .field("name", &self.name)
            .field("role", &self.role)
            .field("valid_until", &self.valid_until)
            .field("pin", &"***")
            .finish()
    }
}

/// `PATCH /api/staff/{id}` 본문. 칸을 모두 보낸다. PIN은 따로 바꾼다.
#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateStaffRequest {
    pub name: String,
    pub role: RoleCode,
    #[ts(type = "string | null")]
    pub valid_until: Option<Date>,
    /// BR-K6: `true`면 사용 중지.
    pub disabled: bool,
}

impl UpdateStaffRequest {
    pub(super) fn as_input(&self) -> StaffInput<'_> {
        StaffInput {
            name: &self.name,
            role: self.role.into(),
            valid_until: self.valid_until,
        }
    }
}

/// `POST /api/staff/{id}/pin` 본문: 새 PIN (BR-K10).
#[derive(Deserialize, TS)]
#[ts(export)]
pub struct ResetPinRequest {
    pub pin: String,
}

impl std::fmt::Debug for ResetPinRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResetPinRequest")
            .field("pin", &"***")
            .finish()
    }
}

/// `POST /api/staff` 응답.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct CreateStaffResponse {
    /// 새 직원의 DB id.
    #[ts(type = "number")]
    pub id: i64,
}

/// 직원 하나. 목록의 한 줄이자 `GET`·`PATCH /api/staff/{id}` 응답.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StaffResponse {
    #[ts(type = "number")]
    pub id: i64,
    pub name: String,
    pub role: RoleCode,
    /// 없으면 `null`(기한 없음).
    #[ts(type = "string | null")]
    pub valid_until: Option<Date>,
    pub disabled: bool,
    /// BR-K7: PIN을 여러 번 틀려 지금 잠겨 있는지. PIN을 새로 정해 주면 풀린다.
    pub locked: bool,
}

impl StaffResponse {
    /// `now`에 잠겨 있는지를 함께 담는다.
    pub(super) fn new(record: StaffRecord, now: OffsetDateTime) -> Self {
        StaffResponse {
            id: record.id,
            name: record.name,
            role: record.role.into(),
            valid_until: record.valid_until,
            disabled: record.disabled,
            locked: is_account_locked(record.locked_until, now),
        }
    }
}

/// `GET /api/staff` 응답. 직원은 적어서 쪽을 나누지 않는다.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct StaffListResponse {
    pub items: Vec<StaffResponse>,
}
