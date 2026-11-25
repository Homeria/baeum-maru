//! `/api/handover`, `/api/run-mode/permit`의 요청 JSON 모양.

use serde::Deserialize;
use ts_rs::TS;

/// `POST /api/handover` 본문. PIN이 들어 있어 `Debug`를 만들지 않는다.
#[derive(Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HandoverRequest {
    /// 로그인한 관리자의 PIN. 한 번 더 확인한다.
    pub pin: String,
}

/// `POST /api/run-mode/permit` 본문. PIN이 들어 있어 `Debug`를 만들지 않는다.
#[derive(Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RunModePermitRequest {
    /// 로그인 화면의 직원 목록에서 고른 관리자.
    #[ts(type = "number")]
    pub staff_id: i64,
    pub pin: String,
}
