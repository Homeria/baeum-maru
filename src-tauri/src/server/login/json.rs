//! `/api/login`, `/api/unlock`, `/api/logout`의 요청·응답 JSON 모양.
//!
//! PIN과 세션 증표가 들어 있어 `Debug`를 직접 써서 글자를 가린다.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::db::staff::LoginChoice;
use crate::server::staff::RoleCode;

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
