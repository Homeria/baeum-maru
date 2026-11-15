//! `/api/rooms`의 요청·응답 JSON 모양과 DB·업무 규칙 타입 사이의 변환.

use baeum_domain::RoomInput;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::db::rooms::RoomRecord;

/// `POST /api/rooms` 본문. 확인 전이라 글자 그대로다.
#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RoomRequest {
    pub name: String,
    pub building: String,
    /// 지하 1층은 `-1`. 0층은 없다. 선택.
    pub floor: Option<i32>,
}

impl RoomRequest {
    pub(super) fn as_input(&self) -> RoomInput<'_> {
        RoomInput {
            name: &self.name,
            building: &self.building,
            floor: self.floor,
        }
    }
}

/// `POST /api/rooms` 응답.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct CreateRoomResponse {
    /// 새 강의실의 DB id.
    #[ts(type = "number")]
    pub id: i64,
}

/// 목록의 강의실 하나.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RoomListItem {
    #[ts(type = "number")]
    pub id: i64,
    pub building: String,
    /// 지하 1층은 `-1`. 없으면 `null`.
    pub floor: Option<i8>,
    pub name: String,
}

impl From<RoomRecord> for RoomListItem {
    fn from(record: RoomRecord) -> Self {
        RoomListItem {
            id: record.id,
            building: record.building,
            floor: record.floor,
            name: record.name,
        }
    }
}

/// `GET /api/rooms` 응답. 강의실은 적어서 쪽을 나누지 않는다.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct RoomListResponse {
    pub items: Vec<RoomListItem>,
}
