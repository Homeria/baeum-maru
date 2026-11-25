//! 회원 더미 데이터. 이름은 흔한 성과 이름 글자를 번호로 조합한다.

use baeum_domain::{Gender, MemberInput, NewMember};
use sqlx::SqlitePool;
use time::{Date, Month, OffsetDateTime};

use super::{SeedError, fail};
use crate::db::members;

const MEMBER_COUNT: usize = 300;

/// 끝의 이만큼은 맨 앞 회원들과 이름이 같다 (동명이인).
const SAME_NAME_COUNT: usize = 3;

const SURNAMES: [&str; 20] = [
    "김", "이", "박", "최", "정", "강", "조", "윤", "장", "임", "한", "오", "서", "신", "권", "황",
    "안", "송", "류", "홍",
];
const FIRST_SYLLABLES: [&str; 16] = [
    "민", "서", "지", "현", "영", "수", "정", "은", "미", "순", "옥", "경", "재", "동", "성", "혜",
];
const SECOND_SYLLABLES: [&str; 16] = [
    "희", "자", "숙", "호", "철", "란", "식", "남", "분", "애", "길", "준", "연", "주", "화", "아",
];

/// 회원을 넣고 id를 넣은 순서대로 돌려준다.
pub async fn insert(
    pool: &SqlitePool,
    today: Date,
    now: OffsetDateTime,
) -> Result<Vec<i64>, SeedError> {
    let mut ids = Vec::with_capacity(MEMBER_COUNT);
    for i in 0..MEMBER_COUNT {
        let name = name(i);
        // 기준 사례처럼 "연도-고유번호" 모양
        let member_no = format!("{:02}-{:05}", 10 + i % 15, 10001 + i);
        let phone_number = format!("010-0000-{:04}", i + 1);
        let member = NewMember::parse(
            MemberInput {
                member_no: &member_no,
                name: &name,
                phone_number: &phone_number,
                birth_date: birth_date(i)?,
                gender: gender(i),
            },
            today,
        )
        .map_err(fail("회원"))?;
        ids.push(
            members::insert(pool, &member, now)
                .await
                .map_err(fail("회원"))?,
        );
    }
    Ok(ids)
}

/// 앞의 297명은 이름이 모두 다르다. (성, 첫 글자)가 번호마다 하나뿐이라서다.
fn name(i: usize) -> String {
    let first_unique = MEMBER_COUNT - SAME_NAME_COUNT;
    let i = if i >= first_unique {
        i - first_unique
    } else {
        i
    };
    format!(
        "{}{}{}",
        SURNAMES[i % SURNAMES.len()],
        FIRST_SYLLABLES[(i / SURNAMES.len()) % FIRST_SYLLABLES.len()],
        SECOND_SYLLABLES[(i * 7) % SECOND_SYLLABLES.len()],
    )
}

/// 1940~1969년생. 열 명에 한 명은 비워 둔다.
fn birth_date(i: usize) -> Result<Option<Date>, SeedError> {
    if i % 10 == 9 {
        return Ok(None);
    }
    let year = 1940 + (i % 30) as i32;
    let month = Month::try_from((i % 12) as u8 + 1).map_err(fail("생년월일"))?;
    let day = (i % 28) as u8 + 1;
    Date::from_calendar_date(year, month, day)
        .map(Some)
        .map_err(fail("생년월일"))
}

/// 번갈아 넣고, 열다섯 명에 한 명은 비워 둔다.
fn gender(i: usize) -> Option<Gender> {
    match (i % 15, i % 2) {
        (14, _) => None,
        (_, 0) => Some(Gender::Female),
        _ => Some(Gender::Male),
    }
}
