//! `/api/recovery`의 요청·응답 JSON 모양.
//!
//! PIN과 복구 코드가 들어 있어 `Debug`를 직접 써서 글자를 가린다.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

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
