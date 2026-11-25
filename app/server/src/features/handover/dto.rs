//! `/api/handover`, `/api/run-mode/permit`의 요청 JSON 모양과, 넘기기·넘겨받기 앱 명령이 화면에
//! 주는 모양.

use serde::{Deserialize, Serialize};
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

/// 넘기기 파일 확인 화면에 보여 줄 것. 회원 이름 같은 개인정보는 담지 않는다.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HandoverFileResponse {
    pub term_names: Vec<String>,
    #[ts(type = "number")]
    pub member_count: i64,
    #[ts(type = "number")]
    pub application_count: i64,
}

/// 넘김 화면의 관리자 목록 한 줄.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HandedOverAdmin {
    #[ts(type = "number")]
    pub id: i64,
    pub name: String,
}

/// 넘겨받기 확인 화면에 보여 줄 것. 관리자 이름 말고는 개인정보를 담지 않는다.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TakeOverPreviewResponse {
    pub term_names: Vec<String>,
    #[ts(type = "number")]
    pub member_count: i64,
    #[ts(type = "number")]
    pub application_count: i64,
    /// 확인에 쓸 수 있는 파일 안의 관리자들.
    pub admins: Vec<TakeOverAdmin>,
}

/// 파일 안의 관리자 한 명.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TakeOverAdmin {
    #[ts(type = "number")]
    pub id: i64,
    pub name: String,
}
