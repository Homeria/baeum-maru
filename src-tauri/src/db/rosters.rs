//! 결과 명단(BR-P1~P11): 학기에서 선발을 마감한 강좌들의 결과를 회원 정보와 함께 읽고,
//! 내보내기 기록을 남긴다.
//!
//! 무엇을 넣고 가릴지는 [`RosterSpec`]이 정하고, Excel 모양은 다른 모듈이 만든다.
//! 여기서는 회원 정보를 모두 읽는다. 명단에 넣지 않는 칸은 Excel을 만들 때 버린다.

use std::collections::HashMap;

use baeum_domain::{Capacity, Gender, Personal, RosterColumn, RosterSpec};
use sqlx::SqlitePool;
use time::{Date, OffsetDateTime};

use super::lotteries::decode_error;
use super::members::gender_from_text;
use super::selection_results::{CourseResultError, ResultMember, course_result_in};
use super::timestamp_text;

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

/// 화면의 강좌 이름과 같다: 과목, 수준, 반을 띄어 잇는다.
pub(super) fn course_name(subject: &str, level: Option<&str>, section: Option<&str>) -> String {
    [Some(subject), level, section]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ")
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

#[cfg(test)]
mod tests {
    use baeum_domain::{MaskChar, RosterChoice, RosterKind};
    use time::macros::datetime;

    use super::*;
    use crate::db::open_temp;
    use crate::db::selection_changes;
    use crate::db::test_support::{insert_course, insert_drawn_course, insert_subject};

    #[tokio::test]
    async fn br_p1_reads_only_closed_courses_with_member_details() {
        let (_dir, pool) = open_temp().await;
        let drawn = insert_drawn_course(&pool, 2, 5, true).await;
        let subject_id = insert_subject(&pool, "서예", "서예", None).await.unwrap();
        insert_course(&pool, drawn.term_id, subject_id, Some("초급"), None)
            .await
            .unwrap();

        let roster = term_roster(&pool, drawn.term_id).await.unwrap();

        assert_eq!(roster.len(), 1);
        let course = &roster[0];
        assert_eq!(course.course_id, drawn.course_id);
        assert_eq!(course.name, "요가");
        assert_eq!(course.capacity.value(), 2);
        let results: Vec<(RosterResult, Option<usize>)> = course
            .entries
            .iter()
            .map(|entry| (entry.result, entry.waitlist_number))
            .collect();
        assert_eq!(
            results,
            [
                (RosterResult::Selected, None),
                (RosterResult::Selected, None),
                (RosterResult::Waitlisted, Some(1)),
                (RosterResult::Waitlisted, Some(2)),
                (RosterResult::Waitlisted, Some(3)),
            ]
        );
        assert!(
            course.entries[0]
                .member
                .phone_number
                .expose()
                .starts_with("010")
        );
    }

    #[tokio::test]
    async fn br_p2_withdrawn_comes_last_without_number() {
        let (_dir, pool) = open_temp().await;
        let drawn = insert_drawn_course(&pool, 2, 5, true).await;
        let now = datetime!(2026-11-23 01:00:00 UTC);
        let first_waiting = term_roster(&pool, drawn.term_id).await.unwrap()[0].entries[2]
            .member
            .member_id;
        let application_id: i64 =
            sqlx::query_scalar("SELECT id FROM applications WHERE member_id = ? AND course_id = ?")
                .bind(first_waiting)
                .bind(drawn.course_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        selection_changes::withdraw(&pool, application_id, None, drawn.staff_id, now)
            .await
            .unwrap();

        let roster = term_roster(&pool, drawn.term_id).await.unwrap();

        let last = roster[0].entries.last().unwrap();
        assert_eq!(last.result, RosterResult::Withdrawn);
        assert_eq!(last.member.member_id, first_waiting);
        assert_eq!(last.waitlist_number, None);
        let numbers: Vec<Option<usize>> = roster[0]
            .entries
            .iter()
            .filter(|entry| entry.result == RosterResult::Waitlisted)
            .map(|entry| entry.waitlist_number)
            .collect();
        assert_eq!(numbers, [Some(2), Some(3)]);
    }

    #[tokio::test]
    async fn br_p11_records_export_with_column_codes() {
        let (_dir, pool) = open_temp().await;
        let drawn = insert_drawn_course(&pool, 2, 5, true).await;
        let spec = RosterSpec::new(RosterChoice {
            kind: RosterKind::PostedByCourse,
            include: &[RosterColumn::PhoneNumber],
            mask: &[RosterColumn::Name],
            mask_char: MaskChar::X,
        })
        .unwrap();

        record_export(
            &pool,
            drawn.term_id,
            &spec,
            1,
            drawn.staff_id,
            datetime!(2026-11-23 01:00:00 UTC),
        )
        .await
        .unwrap();

        let row: (String, String, String, String, i64) = sqlx::query_as(
            "SELECT kind, included_columns, masked_columns, mask_char, course_count
             FROM roster_exports",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            row,
            (
                "posted_by_course".to_owned(),
                "course_name,capacity,result,waitlist_number,name,member_no,phone_number,guaranteed"
                    .to_owned(),
                "name,phone_number".to_owned(),
                "x".to_owned(),
                1
            )
        );
    }

    #[test]
    fn course_name_skips_missing_parts() {
        assert_eq!(
            course_name("요가", Some("초급"), Some("1반")),
            "요가 초급 1반"
        );
        assert_eq!(course_name("서예", None, Some("A")), "서예 A");
        assert_eq!(course_name("탁구", None, None), "탁구");
    }
}
