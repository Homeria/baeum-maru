//! `/api/login`, `/api/unlock`, `/api/logout`, `/api/recovery`의 요청·응답 JSON 모양과 직원 역할.
//!
//! PIN, 세션 증표, 복구 코드가 들어 있어 `Debug`를 직접 써서 글자를 가린다.

use baeum_domain::Role;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::repository::LoginChoice;

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

/// 로그인 화면에서 고를 직원 하나. 화면은 역할별로 묶어 보여 준다.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct LoginStaffResponse {
    #[ts(type = "number")]
    pub id: i64,
    pub name: String,
    pub role: RoleCode,
}

impl From<LoginChoice> for LoginStaffResponse {
    fn from(choice: LoginChoice) -> Self {
        LoginStaffResponse {
            id: choice.id,
            name: choice.name,
            role: choice.role.into(),
        }
    }
}

/// `GET /api/login/staff` 응답. 이름 순서.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct LoginStaffListResponse {
    pub items: Vec<LoginStaffResponse>,
}

/// `POST /api/login` 본문.
#[derive(Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LoginRequest {
    /// 고른 직원의 id.
    #[ts(type = "number")]
    pub staff_id: i64,
    pub pin: String,
}

impl std::fmt::Debug for LoginRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoginRequest")
            .field("staff_id", &self.staff_id)
            .field("pin", &"***")
            .finish()
    }
}

/// `POST /api/unlock` 본문. 세션 증표는 머리글에 싣는다.
#[derive(Deserialize, TS)]
#[ts(export)]
pub struct UnlockRequest {
    pub pin: String,
}

impl std::fmt::Debug for UnlockRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnlockRequest")
            .field("pin", &"***")
            .finish()
    }
}

/// 로그인한 직원.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct SignedInStaffResponse {
    #[ts(type = "number")]
    pub id: i64,
    pub name: String,
    pub role: RoleCode,
}

/// `POST /api/login` 응답.
#[derive(Serialize, TS)]
#[ts(export)]
pub struct LoginResponse {
    /// 세션 증표. 메모리에만 두고 요청마다 `Authorization: Bearer <증표>`로 싣는다.
    pub token: String,
    pub staff: SignedInStaffResponse,
}

impl std::fmt::Debug for LoginResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoginResponse")
            .field("token", &"***")
            .field("staff", &self.staff)
            .finish()
    }
}

/// `POST /api/recovery` 본문.
#[derive(Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecoveryRequest {
    /// PIN을 잊은 관리자의 id. 로그인 화면의 이름 목록에서 고른다.
    #[ts(type = "number")]
    pub staff_id: i64,
    /// 인쇄해 둔 복구 코드. `-`와 공백은 없어도 되고 소문자도 받는다.
    pub code: String,
    /// 새 PIN. 숫자 6~8자리.
    pub pin: String,
}

impl std::fmt::Debug for RecoveryRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecoveryRequest")
            .field("staff_id", &self.staff_id)
            .field("code", &"***")
            .field("pin", &"***")
            .finish()
    }
}

/// `POST /api/recovery` 응답. 새 복구 코드는 이때 한 번만 보여 준다 (BR-K12).
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecoveryResponse {
    /// 4자씩 끊은 16자. 예: `K7QF-M2XA-9TRE-HW4C`
    pub recovery_code: String,
}

impl std::fmt::Debug for RecoveryResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecoveryResponse")
            .field("recovery_code", &"***")
            .finish()
    }
}
