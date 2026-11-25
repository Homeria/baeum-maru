//! 결과 명단(BR-P1~P11): 학기에서 선발을 마감한 강좌들의 결과를 회원 정보와 함께 읽고,
//! 내보내기 기록을 남긴다.
//!
//! 무엇을 넣고 가릴지는 [`RosterSpec`]이 정하고, Excel 모양은 다른 모듈이 만든다.
//! 여기서는 회원 정보를 모두 읽는다. 명단에 넣지 않는 칸은 Excel을 만들 때 버린다.

use std::collections::HashMap;

use baeum_domain::{Capacity, Gender, Personal, RosterColumn, RosterSpec};
use sqlx::SqlitePool;
use time::{Date, OffsetDateTime};

use crate::features::courses::repository::course_name;
use crate::features::members::repository::gender_from_text;
use crate::features::selection_results::repository::{
    CourseResultError, ResultMember, course_result_in,
};
use crate::infra::db::{decode_error, timestamp_text};

/// 선발 결과(BR-P2). 게시용은 포기를 넣지 않는다(BR-P3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RosterResult {
    Selected,
    Waitlisted,
    Withdrawn,
}

/// 명단에 넣는 회원 정보. 모두 개인정보다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RosterMember {
    /// 회원별 명단에서 한 사람의 신청을 모은다.
    pub member_id: i64,
    pub name: Personal<String>,
    pub member_no: Personal<String>,
    /// 숫자만(01012345678).
    pub phone_number: Personal<String>,
    pub birth_date: Option<Personal<Date>>,
    pub gender: Option<Personal<Gender>>,
}

/// 명단의 신청 한 건.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RosterEntry {
    pub result: RosterResult,
    /// 대기자만 있다. 추첨 때 받은 번호(BR-W6).
    pub waitlist_number: Option<usize>,
    /// 자리 보장으로 당첨(BR-S4). 당첨자만 참일 수 있다.
    pub guaranteed: bool,
    pub member: RosterMember,
}

/// 마감한 강좌 하나와 그 신청들. 당첨(선발 결과 탭과 같은 순서), 대기(순번 순), 포기 순.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RosterCourse {
    pub course_id: i64,
    /// 화면과 같은 강좌 이름. 예: `요가 초급 1반`
    pub name: String,
    pub capacity: Capacity,
    pub entries: Vec<RosterEntry>,
}

/// 학기에서 선발을 마감한 강좌들의 결과(BR-P1). 과목 이름, 강좌 id 순(강좌 목록과 같음).
///
/// 한 트랜잭션에서 읽어, 읽는 동안 다른 직원이 포기를 남겨도 명단 안에서 어긋나지 않는다.
pub async fn term_roster(
    pool: &SqlitePool,
    term_id: i64,
) -> Result<Vec<RosterCourse>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let courses: Vec<(i64, String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT c.id, s.name, c.level, c.section
         FROM courses c
         JOIN subjects s ON s.id = c.subject_id
         JOIN selection_closures sc ON sc.course_id = c.id
         WHERE c.term_id = ?
         ORDER BY s.name, c.id",
    )
    .bind(term_id)
    .fetch_all(&mut *tx)
    .await?;
    let details: Vec<DetailRow> = sqlx::query_as(
        "SELECT a.id AS application_id, a.member_id, m.phone_number, m.birth_date, m.gender
         FROM applications a
         JOIN members m ON m.id = a.member_id
         JOIN courses c ON c.id = a.course_id
         JOIN selection_closures sc ON sc.course_id = c.id
         WHERE c.term_id = ?",
    )
    .bind(term_id)
    .fetch_all(&mut *tx)
    .await?;
    let mut details: HashMap<i64, DetailRow> = details
        .into_iter()
        .map(|row| (row.application_id, row))
        .collect();

    let mut roster = Vec::with_capacity(courses.len());
    for (course_id, subject, level, section) in courses {
        let result = course_result_in(&mut tx, course_id)
            .await
            .map_err(|error| match error {
                CourseResultError::Db(error) => error,
                // 위에서 마감한 강좌만 골랐다
                CourseResultError::CourseNotFound | CourseResultError::NotClosed => {
                    decode_error("마감한 강좌의 결과를 읽지 못했습니다")
                }
            })?;
        let mut member = |member: ResultMember| -> Result<RosterMember, sqlx::Error> {
            let row = details
                .remove(&member.application_id)
                .ok_or_else(|| decode_error("신청의 회원 정보가 없습니다"))?;
            Ok(RosterMember {
                member_id: row.member_id,
                name: member.member_name,
                member_no: member.member_no,
                phone_number: Personal::new(row.phone_number),
                birth_date: row.birth_date.map(Personal::new),
                gender: row
                    .gender
                    .as_deref()
                    .map(gender_from_text)
                    .transpose()?
                    .map(Personal::new),
            })
        };
        let mut entries = Vec::new();
        for entry in result.selected {
            entries.push(RosterEntry {
                result: RosterResult::Selected,
                waitlist_number: None,
                guaranteed: entry.guaranteed,
                member: member(entry.member)?,
            });
        }
        for entry in result.waitlisted {
            entries.push(RosterEntry {
                result: RosterResult::Waitlisted,
                waitlist_number: Some(entry.waitlist_number),
                guaranteed: false,
                member: member(entry.member)?,
            });
        }
        for entry in result.withdrawn {
            entries.push(RosterEntry {
                result: RosterResult::Withdrawn,
                waitlist_number: None,
                guaranteed: false,
                member: member(entry.member)?,
            });
        }
        roster.push(RosterCourse {
            course_id,
            name: course_name(&subject, level.as_deref(), section.as_deref()),
            capacity: result.capacity,
            entries,
        });
    }
    tx.commit().await?;
    Ok(roster)
}

#[derive(sqlx::FromRow)]
struct DetailRow {
    application_id: i64,
    member_id: i64,
    phone_number: String,
    birth_date: Option<Date>,
    gender: Option<String>,
}

/// 내보내기 기록을 남긴다(BR-P11). 새 기록의 id를 돌려준다.
pub async fn record_export(
    pool: &SqlitePool,
    term_id: i64,
    spec: &RosterSpec,
    course_count: usize,
    staff_id: i64,
    now: OffsetDateTime,
) -> Result<i64, sqlx::Error> {
    let codes = |columns: &[RosterColumn]| -> String {
        columns
            .iter()
            .map(|column| column.code())
            .collect::<Vec<_>>()
            .join(",")
    };
    let course_count =
        i64::try_from(course_count).map_err(|_| decode_error("강좌 수가 범위 밖입니다"))?;
    let result = sqlx::query(
        "INSERT INTO roster_exports
             (term_id, kind, included_columns, masked_columns, mask_char, course_count,
              exported_by, exported_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(term_id)
    .bind(spec.kind().code())
    .bind(codes(spec.columns()))
    .bind(codes(spec.masked()))
    .bind(spec.mask_char().code())
    .bind(course_count)
    .bind(staff_id)
    .bind(timestamp_text(now))
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}
