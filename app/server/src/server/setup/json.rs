//! `/api/setup`의 요청·응답 JSON 모양.
//!
//! PIN과 복구 코드가 들어 있어 `Debug`를 직접 써서 글자를 가린다.

use baeum_domain::{Role, StaffInput};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// `GET /api/setup` 응답.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct SetupStatusResponse {
    /// 직원이 하나도 없어서 처음 설정을 해야 하면 `true`.
    pub needed: bool,
}

/// `POST /api/setup` 본문: 첫 관리자. 확인 전이라 글자 그대로다.
#[derive(Deserialize, TS)]
#[ts(export)]
pub struct SetupRequest {
    pub name: String,
    /// 숫자 6~8자리 (관리자).
    pub pin: String,
}

impl SetupRequest {
    pub(super) fn as_input(&self) -> StaffInput<'_> {
        StaffInput {
            name: &self.name,
            role: Role::Admin,
            valid_until: None,
        }
    }
}

impl std::fmt::Debug for SetupRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SetupRequest")
            .field("name", &self.name)
            .field("pin", &"***")
            .finish()
    }
}

/// `POST /api/setup` 응답. 복구 코드는 이때 한 번만 보여 준다 (BR-K11).
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SetupResponse {
    /// 4자씩 끊은 16자. 예: `K7QF-M2XA-9TRE-HW4C`
    pub recovery_code: String,
}

impl std::fmt::Debug for SetupResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SetupResponse")
            .field("recovery_code", &"***")
            .finish()
    }
}
