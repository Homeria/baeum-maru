//! `/api/backups`의 요청·응답 JSON 모양.

use serde::{Deserialize, Serialize};
use time::PrimitiveDateTime;
use time::macros::format_description;
use ts_rs::TS;

use crate::backup::name::BackupKind;

/// 백업을 만든 까닭.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum BackupKindCode {
    Daily,
    Manual,
    Exit,
    BeforeLottery,
    BeforeClose,
    BeforeMigration,
}

impl BackupKindCode {
    /// 종류와, 추첨·마감 직전이면 그 강좌 id.
    pub fn split(kind: BackupKind) -> (Self, Option<i64>) {
        match kind {
            BackupKind::Daily => (BackupKindCode::Daily, None),
            BackupKind::Manual => (BackupKindCode::Manual, None),
            BackupKind::Exit => (BackupKindCode::Exit, None),
            BackupKind::BeforeLottery { course_id } => {
                (BackupKindCode::BeforeLottery, Some(course_id))
            }
            BackupKind::BeforeClose { course_id } => (BackupKindCode::BeforeClose, Some(course_id)),
            BackupKind::BeforeMigration => (BackupKindCode::BeforeMigration, None),
        }
    }
}

/// 백업 하나.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BackupResponse {
    /// 백업 폴더 안의 파일 이름. 목록의 열쇠로 쓴다.
    pub file_name: String,
    /// 호스트 PC의 지역 시각. 예: `2026-11-23T09:00:00`
    pub taken_at: String,
    pub kind: BackupKindCode,
    /// 추첨·마감 직전 백업의 강좌.
    #[ts(type = "number | null")]
    pub course_id: Option<i64>,
    /// 강좌 이름. 예: `요가 초급`. 그 강좌가 없으면 `null`.
    pub course_name: Option<String>,
    /// 바이트.
    #[ts(type = "number")]
    pub size: u64,
}

/// 마지막으로 실패한 자동 백업.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BackupFailureResponse {
    /// 호스트 PC의 지역 시각.
    pub at: String,
    pub kind: BackupKindCode,
}

/// 두 번째 위치.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SecondLocationResponse {
    /// 고른 폴더. 꺼져 있으면 `null`.
    pub path: Option<String>,
    /// 지금 그 폴더가 있는지(외장 디스크가 꽂혀 있는지).
    pub reachable: bool,
    /// 그 폴더에 있는 가장 최근 하루 백업의 시각(호스트 PC의 지역 시각).
    pub last_copied_at: Option<String>,
    /// 요청한 PC가 호스트라 바꿀 수 있는지.
    pub changeable: bool,
}

/// `GET /api/backups`의 응답.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BackupListResponse {
    /// 새것부터.
    pub items: Vec<BackupResponse>,
    pub last_failure: Option<BackupFailureResponse>,
    pub second_location: SecondLocationResponse,
}

/// `PUT /api/backups/second-location`의 본문.
#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export)]
pub struct SecondLocationRequest {
    /// 고른 폴더. `null`이면 끈다.
    pub path: Option<String>,
}

/// 지역 시각 글자. 시간대 표시가 없다. 예: `2026-11-23T09:00:00`
pub fn local_text(at: PrimitiveDateTime) -> String {
    at.format(format_description!(
        "[year]-[month]-[day]T[hour]:[minute]:[second]"
    ))
    // 이 형식은 늘 쓸 수 있다
    .unwrap_or_default()
}
