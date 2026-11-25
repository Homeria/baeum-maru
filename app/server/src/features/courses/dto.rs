//! 강좌 API의 요청·응답 JSON 모양과 DB·업무 규칙 타입 사이의 변환.

use baeum_domain::{CourseInput, ScheduleInput, weekday_from_number};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::repository::{CourseRecord, ScheduleRecord, ScheduleRoom};

/// `POST /api/terms/{termId}/courses`, `PATCH /api/courses/{id}` 본문. 확인 전이다.
#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CourseRequest {
    /// 고르지 않았으면 `null`.
    #[ts(type = "number | null")]
    pub subject_id: Option<i64>,
    /// 선택. 비었거나 공백뿐이면 없음.
    pub level: Option<String>,
    /// 선택. 비었거나 공백뿐이면 없음.
    pub section: Option<String>,
    /// 넣지 않았으면 `null`.
    #[ts(type = "number | null")]
    pub capacity: Option<i64>,
    /// 화면 순서대로. 오류의 줄 번호가 이 순서를 따른다.
    pub schedules: Vec<ScheduleRequest>,
}

/// 수업 시간 한 줄.
#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ScheduleRequest {
    /// 1=월 … 7=일
    pub weekday: u8,
    /// HH:MM
    pub start: String,
    /// HH:MM
    pub end: String,
    /// 선택.
    #[ts(type = "number | null")]
    pub room_id: Option<i64>,
}

impl CourseRequest {
    /// 업무 규칙 확인에 넘길 모양. 요일 번호가 1~7 밖이면 `None`(화면 코드의 잘못).
    pub(super) fn as_input(&self) -> Option<CourseInput<'_>> {
        let schedules = self
            .schedules
            .iter()
            .map(|schedule| {
                Some(ScheduleInput {
                    weekday: weekday_from_number(schedule.weekday)?,
                    start: &schedule.start,
                    end: &schedule.end,
                    room_id: schedule.room_id,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        Some(CourseInput {
            subject_id: self.subject_id,
            level: self.level.as_deref(),
            section: self.section.as_deref(),
            capacity: self.capacity,
            schedules,
        })
    }
}

/// `POST /api/terms/{termId}/courses` 응답.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct CreateCourseResponse {
    /// 새 강좌의 DB id.
    #[ts(type = "number")]
    pub id: i64,
}

/// 강좌 하나. 목록의 한 줄이자 `GET`·`PATCH /api/courses/{id}` 응답.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CourseResponse {
    #[ts(type = "number")]
    pub id: i64,
    #[ts(type = "number")]
    pub term_id: i64,
    #[ts(type = "number")]
    pub subject_id: i64,
    pub subject_name: String,
    pub level: Option<String>,
    pub section: Option<String>,
    pub capacity: u16,
    /// 요일, 시작 시각 순서.
    pub schedules: Vec<ScheduleResponse>,
}

/// 저장된 수업 시간 한 줄.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct ScheduleResponse {
    /// 1=월 … 7=일
    pub weekday: u8,
    /// HH:MM
    pub start: String,
    /// HH:MM
    pub end: String,
    /// 없으면 `null`.
    pub room: Option<ScheduleRoomResponse>,
}

/// 수업 시간에 붙은 강의실.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct ScheduleRoomResponse {
    #[ts(type = "number")]
    pub id: i64,
    pub building: String,
    pub name: String,
}

impl From<CourseRecord> for CourseResponse {
    fn from(record: CourseRecord) -> Self {
        CourseResponse {
            id: record.id,
            term_id: record.term_id,
            subject_id: record.subject_id,
            subject_name: record.subject_name,
            level: record.level,
            section: record.section,
            capacity: record.capacity,
            schedules: record
                .schedules
                .into_iter()
                .map(ScheduleResponse::from)
                .collect(),
        }
    }
}

impl From<ScheduleRecord> for ScheduleResponse {
    fn from(record: ScheduleRecord) -> Self {
        ScheduleResponse {
            weekday: record.weekday.number_from_monday(),
            start: record.start.to_string(),
            end: record.end.to_string(),
            room: record.room.map(ScheduleRoomResponse::from),
        }
    }
}

impl From<ScheduleRoom> for ScheduleRoomResponse {
    fn from(room: ScheduleRoom) -> Self {
        ScheduleRoomResponse {
            id: room.id,
            building: room.building,
            name: room.name,
        }
    }
}

/// `GET /api/terms/{termId}/courses` 응답. 과목 이름, 등록 순서.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct CourseListResponse {
    pub items: Vec<CourseResponse>,
}
