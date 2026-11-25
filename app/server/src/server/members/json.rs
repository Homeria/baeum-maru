//! `/api/members`의 요청·응답 JSON 모양과 DB·업무 규칙 타입 사이의 변환.
//!
//! 회원 정보가 그대로 들어 있는 타입에는 `Debug`를 붙이지 않는다 (로그에 찍히지 않게).

use baeum_domain::{Gender, MemberInput, Personal};
use serde::{Deserialize, Serialize};
use time::Date;
use ts_rs::TS;

use crate::db::members::MemberRecord;
use crate::infra::db::timestamp_text;

/// 성별. JSON으로는 `"male"`, `"female"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "Gender")]
pub enum GenderJson {
    Male,
    Female,
}

impl From<GenderJson> for Gender {
    fn from(gender: GenderJson) -> Self {
        match gender {
            GenderJson::Male => Gender::Male,
            GenderJson::Female => Gender::Female,
        }
    }
}

impl From<Gender> for GenderJson {
    fn from(gender: Gender) -> Self {
        match gender {
            Gender::Male => GenderJson::Male,
            Gender::Female => GenderJson::Female,
        }
    }
}

/// `POST /api/members`, `PATCH /api/members/{id}` 본문. 확인 전이라 글자 그대로다.
///
/// 고칠 때도 다섯 칸을 모두 보낸다. 비운 선택 칸은 `null`이다.
///
/// 회원 정보가 그대로 들어 있어 `Debug`를 붙이지 않는다 (로그에 찍히지 않게).
#[derive(Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MemberRequest {
    pub member_no: String,
    pub name: String,
    pub phone_number: String,
    /// `YYYY-MM-DD`. 선택.
    #[ts(type = "string | null")]
    pub birth_date: Option<Date>,
    /// 선택.
    pub gender: Option<GenderJson>,
}

impl MemberRequest {
    pub(super) fn as_input(&self) -> MemberInput<'_> {
        MemberInput {
            member_no: &self.member_no,
            name: &self.name,
            phone_number: &self.phone_number,
            birth_date: self.birth_date,
            gender: self.gender.map(Gender::from),
        }
    }
}

/// `POST /api/members` 응답.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct CreateMemberResponse {
    /// 새 회원의 DB id.
    // [ts-rs]: i64는 bigint가 되는데 JSON 숫자는 number로 읽힘. id는 2^53을 넘지 않음
    #[ts(type = "number")]
    pub id: i64,
}

/// 회원 한 명. `GET`·`PATCH /api/members/{id}` 응답.
///
/// 회원 정보가 그대로 들어 있어 `Debug`를 붙이지 않는다.
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MemberResponse {
    #[ts(type = "number")]
    pub id: i64,
    pub member_no: String,
    pub name: String,
    /// 숫자만.
    pub phone_number: String,
    #[ts(type = "string | null")]
    pub birth_date: Option<Date>,
    pub gender: Option<GenderJson>,
    /// UTC, 밀리초까지, 끝에 Z. 화면에서 한국 시각으로 바꾼다.
    pub created_at: String,
    /// `created_at`과 같은 모양.
    pub updated_at: String,
}

impl From<MemberRecord> for MemberResponse {
    fn from(record: MemberRecord) -> Self {
        MemberResponse {
            id: record.id,
            member_no: record.member_no.into_inner(),
            name: record.name.into_inner(),
            phone_number: record.phone_number.into_inner(),
            birth_date: record.birth_date.map(Personal::into_inner),
            gender: record
                .gender
                .map(|gender| GenderJson::from(gender.into_inner())),
            created_at: timestamp_text(record.created_at),
            updated_at: timestamp_text(record.updated_at),
        }
    }
}

/// `GET /api/members`의 `?` 뒤 값.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchMembersParams {
    /// 이름, 회원번호, 연락처의 일부. 없으면 모두.
    pub query: Option<String>,
    /// 1부터. 없으면 1.
    pub page: Option<u32>,
    /// 1~100. 없으면 20.
    pub page_size: Option<u32>,
}

/// 목록의 회원 한 명.
///
/// 회원 정보가 그대로 들어 있어 `Debug`를 붙이지 않는다.
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MemberListItem {
    #[ts(type = "number")]
    pub id: i64,
    pub member_no: String,
    pub name: String,
    /// 숫자만. 하이픈은 화면에서 넣는다.
    pub phone_number: String,
    #[ts(type = "string | null")]
    pub birth_date: Option<Date>,
    pub gender: Option<GenderJson>,
}

impl From<MemberRecord> for MemberListItem {
    fn from(record: MemberRecord) -> Self {
        MemberListItem {
            id: record.id,
            member_no: record.member_no.into_inner(),
            name: record.name.into_inner(),
            phone_number: record.phone_number.into_inner(),
            birth_date: record.birth_date.map(Personal::into_inner),
            gender: record
                .gender
                .map(|gender| GenderJson::from(gender.into_inner())),
        }
    }
}

/// `GET /api/members` 응답.
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MemberListResponse {
    pub items: Vec<MemberListItem>,
    /// 조건에 맞는 전체 인원.
    #[ts(type = "number")]
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
}
