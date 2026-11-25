//! 추첨 API의 응답 JSON 모양과 DB 타입 사이의 변환.

use serde::Serialize;
use ts_rs::TS;

use super::repository::{CourseStage, Entrant, SelectionStage};

/// 강좌의 선발 단계. DB 기록에서 계산한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum SelectionStageCode {
    /// 추첨 전. 신청을 받는다.
    NotDrawn,
    /// 추첨함. 마감 전이라 다시 추첨할 수 있다(BR-L4).
    Drawn,
    /// 선발 마감(BR-L5).
    Closed,
}

impl From<SelectionStage> for SelectionStageCode {
    fn from(stage: SelectionStage) -> Self {
        match stage {
            SelectionStage::NotDrawn => SelectionStageCode::NotDrawn,
            SelectionStage::Drawn => SelectionStageCode::Drawn,
            SelectionStage::Closed => SelectionStageCode::Closed,
        }
    }
}

/// `GET /api/terms/{termId}/selections` 응답. 학기의 모든 강좌, 강좌 id 순.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct SelectionListResponse {
    pub items: Vec<CourseStageResponse>,
}

/// 강좌 하나의 선발 단계.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CourseStageResponse {
    #[ts(type = "number")]
    pub course_id: i64,
    pub stage: SelectionStageCode,
}

impl From<CourseStage> for CourseStageResponse {
    fn from(stage: CourseStage) -> Self {
        CourseStageResponse {
            course_id: stage.course_id,
            stage: stage.stage.into(),
        }
    }
}

/// `GET·POST /api/courses/{id}/lottery` 응답. 추첨 화면에 보여 줄 것.
// 회원 이름이 담겨 Debug를 만들지 않는다 (로그에 찍히지 않게)
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CourseLotteryResponse {
    /// 추첨 뒤면 추첨한 때의 정원, 추첨 전이면 지금 정원.
    pub capacity: u16,
    pub stage: SelectionStageCode,
    /// 자리 보장 신청이 늘 맨 앞. 그 뒤는 추첨 전이면 접수한 순서, 추첨 뒤면 뽑힌 순서.
    pub entrants: Vec<EntrantResponse>,
    /// 추첨 뒤 `entrants`의 앞에서 몇 명이 당첨인지(자리 보장 포함). 나머지는 차례로 대기 순번 1, 2, … 추첨 전이면 0.
    #[ts(type = "number")]
    pub selected_count: usize,
    /// 이 직원이 이 PC에서 추첨·재추첨·마감을 할 수 있는지(역할과 BR-L6).
    pub can_draw: bool,
}

/// 추첨 화면의 신청자 한 명. 화면은 `이름(회원번호)`로 보여 준다.
// 회원 이름이 담겨 Debug를 만들지 않는다
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct EntrantResponse {
    #[ts(type = "number")]
    pub application_id: i64,
    pub member_name: String,
    pub member_no: String,
    /// 자리 보장으로 당첨되는 신청(BR-S4). 사유 이름은 싣지 않는다(BR-S15).
    pub guaranteed: bool,
}

impl From<Entrant> for EntrantResponse {
    fn from(entrant: Entrant) -> Self {
        EntrantResponse {
            application_id: entrant.application_id,
            member_name: entrant.member_name.into_inner(),
            member_no: entrant.member_no.into_inner(),
            guaranteed: entrant.guaranteed,
        }
    }
}
