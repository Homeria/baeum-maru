//! 신청의 SQL: 목록, 인원, 저장과 지우기에 쓰는 읽기·쓰기. 순서와 규칙 확인은 `service`가 한다.

pub(crate) mod priorities;

use std::collections::HashMap;

use baeum_domain::{
    AdditionalOutcome, ApplicationRule, ClassTime, CourseForRules, Schedule, weekday_from_number,
};
use sqlx::{SqliteConnection, SqlitePool};
use time::{Date, OffsetDateTime};

use crate::features::selection_results::error::CourseResultError;
use crate::features::selection_results::repository::{course_result_in, next_additional};
use crate::infra::db::decode_error;
use crate::infra::db::timestamp_text;

/// 유효한 신청의 상태(BR-A 머리말). 탈락·포기는 규칙 계산과 인원에서 빠진다.
macro_rules! valid_statuses {
    () => {
        "('submitted', 'selected', 'waitlisted', 'confirmed')"
    };
}

/// 회원의 신청 하나.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationRecord {
    pub id: i64,
    pub course_id: i64,
    pub submitted_at: OffsetDateTime,
    /// 예외를 받은 규칙. 규칙 순서.
    pub overridden_rules: Vec<ApplicationRule>,
    /// 붙은 우선 배정 사유(BR-S12). id 순서.
    pub priority_reason_ids: Vec<i64>,
}

/// 강좌 하나의 유효한 신청 인원.
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::FromRow)]
pub struct CourseCount {
    pub course_id: i64,
    pub count: i64,
}

/// 회원이 `term_id` 학기에 낸 유효한 신청. 접수한 순서.
pub async fn list_for_member(
    pool: &SqlitePool,
    term_id: i64,
    member_id: i64,
) -> Result<Vec<ApplicationRecord>, sqlx::Error> {
    let rows: Vec<ApplicationRow> = sqlx::query_as(concat!(
        "SELECT a.id, a.course_id, a.submitted_at
         FROM applications a JOIN courses c ON c.id = a.course_id
         WHERE c.term_id = ? AND a.member_id = ? AND a.status IN ",
        valid_statuses!(),
        " ORDER BY a.submitted_at, a.id"
    ))
    .bind(term_id)
    .bind(member_id)
    .fetch_all(pool)
    .await?;
    let overrides: Vec<OverrideRow> = sqlx::query_as(concat!(
        "SELECT o.application_id, o.rule
         FROM rule_overrides o
         JOIN applications a ON a.id = o.application_id
         JOIN courses c ON c.id = a.course_id
         WHERE c.term_id = ? AND a.member_id = ? AND a.status IN ",
        valid_statuses!()
    ))
    .bind(term_id)
    .bind(member_id)
    .fetch_all(pool)
    .await?;

    let priorities: Vec<(i64, i64)> = sqlx::query_as(concat!(
        "SELECT p.application_id, p.reason_id
         FROM application_priorities p
         JOIN applications a ON a.id = p.application_id
         JOIN courses c ON c.id = a.course_id
         WHERE c.term_id = ? AND a.member_id = ? AND a.status IN ",
        valid_statuses!(),
        " ORDER BY p.reason_id"
    ))
    .bind(term_id)
    .bind(member_id)
    .fetch_all(pool)
    .await?;
    let mut reasons_by_application: HashMap<i64, Vec<i64>> = HashMap::new();
    for (application_id, reason_id) in priorities {
        reasons_by_application
            .entry(application_id)
            .or_default()
            .push(reason_id);
    }

    let mut by_application: HashMap<i64, Vec<ApplicationRule>> = HashMap::new();
    for row in overrides {
        let rule = ApplicationRule::from_code(&row.rule)
            .ok_or_else(|| sqlx::Error::Decode(format!("모르는 규칙: {}", row.rule).into()))?;
        by_application
            .entry(row.application_id)
            .or_default()
            .push(rule);
    }
    Ok(rows
        .into_iter()
        .map(|row| {
            let mut overridden_rules = by_application.remove(&row.id).unwrap_or_default();
            overridden_rules.sort();
            ApplicationRecord {
                id: row.id,
                course_id: row.course_id,
                submitted_at: row.submitted_at,
                overridden_rules,
                priority_reason_ids: reasons_by_application.remove(&row.id).unwrap_or_default(),
            }
        })
        .collect())
}

/// `term_id` 학기 강좌들의 유효한 신청 인원. 신청이 없는 강좌는 빠진다.
pub async fn count_by_course(
    pool: &SqlitePool,
    term_id: i64,
) -> Result<Vec<CourseCount>, sqlx::Error> {
    sqlx::query_as(concat!(
        "SELECT a.course_id, COUNT(*) AS count
         FROM applications a JOIN courses c ON c.id = a.course_id
         WHERE c.term_id = ? AND a.status IN ",
        valid_statuses!(),
        " GROUP BY a.course_id ORDER BY a.course_id"
    ))
    .bind(term_id)
    .fetch_all(pool)
    .await
}

/// 선발 마감한 강좌 `course_id`에 추가 신청 하나를 받으면 어떻게 되는지(BR-X2).
pub(super) async fn additional_outcome_for(
    conn: &mut SqliteConnection,
    course_id: i64,
) -> Result<AdditionalOutcome, sqlx::Error> {
    let result = course_result_in(conn, course_id)
        .await
        .map_err(|error| match error {
            CourseResultError::Db(error) => error,
            // 위에서 같은 트랜잭션으로 마감한 강좌임을 확인했다
            CourseResultError::CourseNotFound | CourseResultError::NotClosed => {
                decode_error("마감한 강좌의 선발 결과가 없습니다")
            }
        })?;
    Ok(next_additional(&result))
}

/// 회원이 있는지.
pub(super) async fn member_exists(
    conn: &mut SqliteConnection,
    member_id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM members WHERE id = ?)")
        .bind(member_id)
        .fetch_one(conn)
        .await
}

/// 학기 종료일.
pub(super) async fn term_ends_on(
    conn: &mut SqliteConnection,
    term_id: i64,
) -> Result<Date, sqlx::Error> {
    sqlx::query_scalar("SELECT ends_on FROM terms WHERE id = ?")
        .bind(term_id)
        .fetch_one(conn)
        .await
}

/// 신청 하나를 저장하고 id를 돌려준다. 추가 신청이면 그 결과로 당첨이나 대기, 아니면 접수됨이다.
/// `now`는 [`timestamp_text`] 글자.
pub(super) async fn insert_application(
    conn: &mut SqliteConnection,
    member_id: i64,
    course_id: i64,
    additional: Option<AdditionalOutcome>,
    staff_id: i64,
    now: &str,
) -> Result<i64, sqlx::Error> {
    let status = match additional {
        None => "submitted",
        Some(AdditionalOutcome::Selected) => "selected",
        Some(AdditionalOutcome::Waitlisted(_)) => "waitlisted",
    };
    Ok(sqlx::query(
        "INSERT INTO applications (member_id, course_id, status, submitted_by, submitted_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(member_id)
    .bind(course_id)
    .bind(status)
    .bind(staff_id)
    .bind(now)
    .execute(conn)
    .await?
    .last_insert_rowid())
}

/// 추가 신청 기록과 대기 순번을 남긴다(BR-X2).
pub(super) async fn insert_additional(
    conn: &mut SqliteConnection,
    application_id: i64,
    outcome: AdditionalOutcome,
) -> Result<(), sqlx::Error> {
    let waitlist_number = match outcome {
        AdditionalOutcome::Selected => None,
        AdditionalOutcome::Waitlisted(number) => {
            Some(i64::try_from(number).map_err(|_| decode_error("대기 순번이 범위 밖입니다"))?)
        }
    };
    sqlx::query(
        "INSERT INTO additional_applications (application_id, waitlist_number)
         VALUES (?, ?)",
    )
    .bind(application_id)
    .bind(waitlist_number)
    .execute(conn)
    .await?;
    Ok(())
}

/// 규칙 예외 하나를 남긴다(BR-A6). `now`는 [`timestamp_text`] 글자.
pub(super) async fn insert_rule_override(
    conn: &mut SqliteConnection,
    application_id: i64,
    rule: ApplicationRule,
    reason: &str,
    staff_id: i64,
    now: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO rule_overrides (application_id, rule, reason, staff_id, created_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(application_id)
    .bind(rule.code())
    .bind(reason)
    .bind(staff_id)
    .bind(now)
    .execute(conn)
    .await?;
    Ok(())
}

/// 지울 신청의 회원, 강좌, 상태(DB 글자). 없으면 `None`.
pub(super) async fn deletion_target(
    conn: &mut SqliteConnection,
    id: i64,
) -> Result<Option<(i64, i64, String)>, sqlx::Error> {
    sqlx::query_as("SELECT member_id, course_id, status FROM applications WHERE id = ?")
        .bind(id)
        .fetch_optional(conn)
        .await
}

/// 삭제 기록을 남기고 신청을 지운다(BR-D2). 규칙 예외도 함께 지워진다.
pub(super) async fn delete_with_record(
    conn: &mut SqliteConnection,
    id: i64,
    member_id: i64,
    course_id: i64,
    staff_id: i64,
    reason: Option<&str>,
    now: OffsetDateTime,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO application_deletions (member_id, course_id, deleted_by, deleted_at, reason)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(member_id)
    .bind(course_id)
    .bind(staff_id)
    .bind(timestamp_text(now))
    .bind(reason)
    .execute(&mut *conn)
    .await?;
    sqlx::query("DELETE FROM applications WHERE id = ?")
        .bind(id)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

#[derive(sqlx::FromRow)]
struct ApplicationRow {
    id: i64,
    course_id: i64,
    submitted_at: OffsetDateTime,
}

#[derive(sqlx::FromRow)]
struct OverrideRow {
    application_id: i64,
    rule: String,
}

/// 강좌 한 줄 x 수업 시간 한 줄. 수업 시간이 없으면 수업 시간 칸이 비어 있다.
#[derive(sqlx::FromRow)]
struct RuleCourseRow {
    course_id: i64,
    subject_id: i64,
    weekday: Option<u8>,
    start_minute: Option<u16>,
    end_minute: Option<u16>,
}

/// 규칙 판단에 쓰는 강좌의 과목과 수업 시간.
pub(super) struct RuleCourse {
    subject_id: i64,
    schedules: Vec<Schedule>,
}

impl RuleCourse {
    pub(super) fn for_rules(&self, course_id: i64) -> CourseForRules<'_> {
        CourseForRules {
            course_id,
            subject_id: self.subject_id,
            schedules: &self.schedules,
        }
    }
}

/// `term_id` 학기의 강좌마다 과목과 수업 시간. 강좌 id로 찾는다.
pub(super) async fn term_courses(
    conn: &mut SqliteConnection,
    term_id: i64,
) -> Result<HashMap<i64, RuleCourse>, sqlx::Error> {
    // 수업 시간이 없는 강좌도 빠지지 않게 LEFT JOIN (BR-T4로 생기지 않지만 DB가 막지는 않는다)
    let rows: Vec<RuleCourseRow> = sqlx::query_as(
        "SELECT c.id AS course_id, c.subject_id, cs.weekday, cs.start_minute, cs.end_minute
         FROM courses c LEFT JOIN course_schedules cs ON cs.course_id = c.id
         WHERE c.term_id = ?",
    )
    .bind(term_id)
    .fetch_all(conn)
    .await?;

    let mut courses: HashMap<i64, RuleCourse> = HashMap::new();
    for row in rows {
        let course = courses.entry(row.course_id).or_insert_with(|| RuleCourse {
            subject_id: row.subject_id,
            schedules: Vec::new(),
        });
        let (Some(weekday), Some(start), Some(end)) =
            (row.weekday, row.start_minute, row.end_minute)
        else {
            continue;
        };
        // 테이블 CHECK가 막는 값이라 실패하면 DB가 망가진 것이다
        let schedule = weekday_from_number(weekday)
            .zip(ClassTime::from_minutes(start).zip(ClassTime::from_minutes(end)))
            .and_then(|(weekday, (start, end))| Schedule::new(weekday, start, end, None))
            .ok_or_else(|| sqlx::Error::Decode("수업 시간 값이 범위 밖입니다".into()))?;
        course.schedules.push(schedule);
    }
    Ok(courses)
}

/// `term_id` 학기에서 추첨한(취소되지 않은 추첨 기록이 있는) 강좌들. 마감한 강좌도 들어간다.
pub(super) async fn drawn_course_ids(
    conn: &mut SqliteConnection,
    term_id: i64,
) -> Result<Vec<i64>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT r.course_id
         FROM lottery_runs r JOIN courses c ON c.id = r.course_id
         WHERE c.term_id = ? AND r.cancelled_at IS NULL",
    )
    .bind(term_id)
    .fetch_all(conn)
    .await
}

/// `term_id` 학기에서 선발을 마감한 강좌들.
pub(super) async fn closed_course_ids(
    conn: &mut SqliteConnection,
    term_id: i64,
) -> Result<Vec<i64>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT s.course_id
         FROM selection_closures s JOIN courses c ON c.id = s.course_id
         WHERE c.term_id = ?",
    )
    .bind(term_id)
    .fetch_all(conn)
    .await
}

/// 회원이 `term_id` 학기에 유효한 신청을 낸 강좌들. 접수한 순서.
pub(super) async fn applied_course_ids(
    conn: &mut SqliteConnection,
    term_id: i64,
    member_id: i64,
) -> Result<Vec<i64>, sqlx::Error> {
    sqlx::query_scalar(concat!(
        "SELECT a.course_id
         FROM applications a JOIN courses c ON c.id = a.course_id
         WHERE c.term_id = ? AND a.member_id = ? AND a.status IN ",
        valid_statuses!(),
        " ORDER BY a.submitted_at, a.id"
    ))
    .bind(term_id)
    .bind(member_id)
    .fetch_all(conn)
    .await
}
