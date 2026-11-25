//! 결과 명단 API의 요청 JSON 모양과 업무 규칙 타입 사이의 변환.

use baeum_domain::{MaskChar, RosterColumn, RosterKind};
use serde::Deserialize;
use ts_rs::TS;

/// 결과 명단 종류(BR-P1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum RosterKindCode {
    Raw,
    PostedByCourse,
    PostedByMember,
}

impl From<RosterKindCode> for RosterKind {
    fn from(code: RosterKindCode) -> Self {
        match code {
            RosterKindCode::Raw => RosterKind::Raw,
            RosterKindCode::PostedByCourse => RosterKind::PostedByCourse,
            RosterKindCode::PostedByMember => RosterKind::PostedByMember,
        }
    }
}

/// 명단 칸(BR-P2~P5). 우선 배정 사유와 포기 사유는 없다(BR-P8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum RosterColumnCode {
    CourseName,
    Capacity,
    Result,
    WaitlistNumber,
    Name,
    MemberNo,
    PhoneNumber,
    BirthDate,
    Gender,
    Guaranteed,
}

impl From<RosterColumnCode> for RosterColumn {
    fn from(code: RosterColumnCode) -> Self {
        match code {
            RosterColumnCode::CourseName => RosterColumn::CourseName,
            RosterColumnCode::Capacity => RosterColumn::Capacity,
            RosterColumnCode::Result => RosterColumn::Result,
            RosterColumnCode::WaitlistNumber => RosterColumn::WaitlistNumber,
            RosterColumnCode::Name => RosterColumn::Name,
            RosterColumnCode::MemberNo => RosterColumn::MemberNo,
            RosterColumnCode::PhoneNumber => RosterColumn::PhoneNumber,
            RosterColumnCode::BirthDate => RosterColumn::BirthDate,
            RosterColumnCode::Gender => RosterColumn::Gender,
            RosterColumnCode::Guaranteed => RosterColumn::Guaranteed,
        }
    }
}

/// 가림 글자(BR-P7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum MaskCharCode {
    Circle,
    Asterisk,
    X,
}

impl From<MaskCharCode> for MaskChar {
    fn from(code: MaskCharCode) -> Self {
        match code {
            MaskCharCode::Circle => MaskChar::Circle,
            MaskCharCode::Asterisk => MaskChar::Asterisk,
            MaskCharCode::X => MaskChar::X,
        }
    }
}

/// `POST /api/terms/{termId}/roster-exports` 본문. 응답은 Excel 파일(.xlsx)이다.
#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RosterExportRequest {
    pub kind: RosterKindCode,
    /// 넣을 칸. 늘 넣는 칸은 적지 않아도 들어간다.
    pub include: Vec<RosterColumnCode>,
    /// 가릴 칸. 넣지 않은 칸은 무시한다. 게시용 연락처·생년월일은 적지 않아도 가린다.
    pub mask: Vec<RosterColumnCode>,
    pub mask_char: MaskCharCode,
}
