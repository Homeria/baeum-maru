//! 신청 저장(규칙 확인과 규칙 예외, 추가 신청), 목록, 지우기(삭제 기록).

use std::collections::{BTreeSet, HashMap};

use baeum_domain::{
    AdditionalOutcome, ApplicationRule, ClassTime, CourseForRules, RuleViolation, Schedule,
    check_application, weekday_from_number,
};
use sqlx::{SqliteConnection, SqlitePool};
use time::{Date, OffsetDateTime};

use super::application_priorities::{self, ChangeTarget};
use super::lotteries::decode_error;
use super::selection_results::{CourseResultError, course_result_in, next_additional};
use super::timestamp_text;
use crate::clock;

/// 유효한 신청의 상태(BR-A 머리말). 탈락·포기는 규칙 계산과 인원에서 빠진다.
macro_rules! valid_statuses {
    () => {
        "('submitted', 'selected', 'waitlisted', 'confirmed')"
    };
}

/// 회원 한 명이 한 학기 강좌 여러 개에 내는 신청.
#[derive(Debug, Clone, Copy)]
pub struct NewApplications<'a> {
    pub term_id: i64,
    pub member_id: i64,
    /// 고른 순서. 이번에 고른 강좌끼리 부딪히면 뒤 강좌에 붙인다.
    pub course_ids: &'a [i64],
    /// 규칙 예외 사유(BR-A6). 없으면 규칙을 어길 때 저장하지 않는다(BR-A5).
    pub override_reason: Option<&'a str>,
    /// 접수한 직원.
    pub staff_id: i64,
    /// 강좌마다 붙일 우선 배정 사유(BR-S12). 없는 강좌는 빼도 된다.
    pub priorities: &'a [CoursePriorities<'a>],
}

/// 고른 강좌 하나에 붙일 우선 배정 사유들.
#[derive(Debug, Clone, Copy)]
pub struct CoursePriorities<'a> {
    /// [`NewApplications::course_ids`]에 있는 강좌.
    pub course_id: i64,
    pub reason_ids: &'a [i64],
}

/// 신청을 저장하지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum SubmitError {
    #[error("회원을 찾지 못했습니다")]
    MemberNotFound,
    /// 고른 강좌가 그 학기에 없다. 그 사이 지워졌거나 다른 학기의 강좌다.
    #[error("이 학기에 없는 강좌를 골랐습니다")]
    CourseNotFound(Vec<i64>),
    /// BR-L3: 추첨한 강좌는 선발 마감 전까지 새 신청을 받지 않는다. 고른 강좌 가운데 그런 강좌들.
    #[error("추첨한 강좌는 선발 마감 전까지 새 신청을 받지 않습니다")]
    CourseDrawn(Vec<i64>),
    /// BR-X1: 학기 종료일이 지나 추가 신청을 받지 않는다. 고른 강좌 가운데 선발 마감한 강좌들.
    #[error("학기가 끝나 추가 신청을 받지 않습니다")]
    TermEnded(Vec<i64>),
    /// 우선 배정 사유를 고르지 않은 강좌에 붙였다.
    #[error("고르지 않은 강좌에 우선 배정을 붙였습니다")]
    PriorityForUnchosenCourse(Vec<i64>),
    /// 목록에 없는 우선 배정 사유. 그 사이 지워졌을 수 있다.
    #[error("목록에 없는 우선 배정 사유를 골랐습니다")]
    PriorityReasonNotFound(Vec<i64>),
    /// BR-A5: 규칙을 어겼다. 사유가 있어도 예외를 줄 수 없는 규칙(BR-A7)이면 여기로 온다.
    #[error("신청 규칙을 어겼습니다")]
    RulesBroken(Vec<RuleViolation>),
    #[error("신청을 저장하지 못했습니다: {0}")]
    Db(#[from] sqlx::Error),
}

/// 저장한 신청 하나.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Submitted {
    pub id: i64,
    pub course_id: i64,
    /// 추가 신청(BR-X2)이면 그 결과. 보통 신청(접수됨)이면 `None`.
    pub additional: Option<AdditionalOutcome>,
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

/// 신청을 지우지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum DeleteApplicationError {
    #[error("신청을 찾지 못했습니다")]
    NotFound,
    /// BR-D1: 접수됨(선발 전) 신청만 지운다.
    #[error("선발 뒤의 신청은 지울 수 없습니다")]
    NotSubmitted,
    #[error("신청을 지우지 못했습니다: {0}")]
    Db(#[from] sqlx::Error),
}

/// 신청을 저장하고 고른 순서대로 돌려준다.
///
/// 선발 마감한 강좌는 추가 신청으로 받아 바로 당첨이나 대기로 정한다(BR-X1, BR-X2). 같은
/// 트랜잭션에서 세므로 여러 PC가 동시에 받아도 늦게 저장한 쪽이 뒤에 선다.
///
/// 규칙을 어기면 사유가 있고 모두 예외를 줄 수 있는 규칙일 때만 저장하고, 어긴 규칙마다 규칙 예외를 남긴다.
/// 이미 낸 신청을 읽고 확인한 뒤 저장하므로 `BEGIN IMMEDIATE`로 한다 (ADR 0005).
pub async fn submit(
    pool: &SqlitePool,
    new: NewApplications<'_>,
    now: OffsetDateTime,
) -> Result<Vec<Submitted>, SubmitError> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let member_exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM members WHERE id = ?)")
            .bind(new.member_id)
            .fetch_one(&mut *tx)
            .await?;
    if !member_exists {
        return Err(SubmitError::MemberNotFound);
    }

    let courses = term_courses(&mut tx, new.term_id).await?;
    let missing: Vec<i64> = new
        .course_ids
        .iter()
        .copied()
        .filter(|id| !courses.contains_key(id))
        .collect();
    if !missing.is_empty() {
        return Err(SubmitError::CourseNotFound(missing));
    }
    let drawn_ids = drawn_course_ids(&mut tx, new.term_id).await?;
    let closed_ids = closed_course_ids(&mut tx, new.term_id).await?;
    let (closed, drawn): (Vec<i64>, Vec<i64>) = new
        .course_ids
        .iter()
        .copied()
        .filter(|id| drawn_ids.contains(id))
        .partition(|id| closed_ids.contains(id));
    if !drawn.is_empty() {
        return Err(SubmitError::CourseDrawn(drawn));
    }
    if !closed.is_empty() {
        let ends_on: Date = sqlx::query_scalar("SELECT ends_on FROM terms WHERE id = ?")
            .bind(new.term_id)
            .fetch_one(&mut *tx)
            .await?;
        if clock::local_date(now) > ends_on {
            return Err(SubmitError::TermEnded(closed));
        }
    }
    let unchosen: Vec<i64> = new
        .priorities
        .iter()
        .map(|priority| priority.course_id)
        .filter(|id| !new.course_ids.contains(id))
        .collect();
    if !unchosen.is_empty() {
        return Err(SubmitError::PriorityForUnchosenCourse(unchosen));
    }
    let reason_ids: BTreeSet<i64> = new
        .priorities
        .iter()
        .flat_map(|priority| priority.reason_ids.iter().copied())
        .collect();
    let missing = application_priorities::missing_reasons(&mut tx, &reason_ids).await?;
    if !missing.is_empty() {
        return Err(SubmitError::PriorityReasonNotFound(missing));
    }

    let applied_ids = applied_course_ids(&mut tx, new.term_id, new.member_id).await?;
    // 고른 강좌는 위에서, 신청한 강좌는 같은 학기 강좌라서 `courses`에 반드시 있다
    let for_rules = |id: &i64| courses[id].for_rules(*id);
    let applied: Vec<CourseForRules<'_>> = applied_ids.iter().map(for_rules).collect();
    let chosen: Vec<CourseForRules<'_>> = new.course_ids.iter().map(for_rules).collect();
    let violations = check_application(&applied, &chosen);

    let reason = match new.override_reason {
        _ if violations.is_empty() => None,
        Some(reason) if violations.iter().all(|v| v.rule.can_override()) => Some(reason),
        _ => return Err(SubmitError::RulesBroken(violations)),
    };

    let now = timestamp_text(now);
    let mut submitted = Vec::with_capacity(new.course_ids.len());
    for &course_id in new.course_ids {
        let additional = if closed_ids.contains(&course_id) {
            Some(additional_outcome_for(&mut tx, course_id).await?)
        } else {
            None
        };
        let status = match additional {
            None => "submitted",
            Some(AdditionalOutcome::Selected) => "selected",
            Some(AdditionalOutcome::Waitlisted(_)) => "waitlisted",
        };
        let id = sqlx::query(
            "INSERT INTO applications (member_id, course_id, status, submitted_by, submitted_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(new.member_id)
        .bind(course_id)
        .bind(status)
        .bind(new.staff_id)
        .bind(&now)
        .execute(&mut *tx)
        .await?
        .last_insert_rowid();
        if let Some(outcome) = additional {
            let waitlist_number = match outcome {
                AdditionalOutcome::Selected => None,
                AdditionalOutcome::Waitlisted(number) => Some(
                    i64::try_from(number).map_err(|_| decode_error("대기 순번이 범위 밖입니다"))?,
                ),
            };
            sqlx::query(
                "INSERT INTO additional_applications (application_id, waitlist_number)
                 VALUES (?, ?)",
            )
            .bind(id)
            .bind(waitlist_number)
            .execute(&mut *tx)
            .await?;
        }
        if let Some(reason) = reason {
            let mut rules: Vec<ApplicationRule> = violations
                .iter()
                .filter(|v| v.course_id == course_id)
                .map(|v| v.rule)
                .collect();
            rules.sort();
            rules.dedup();
            for rule in rules {
                sqlx::query(
                    "INSERT INTO rule_overrides (application_id, rule, reason, staff_id, created_at)
                     VALUES (?, ?, ?, ?, ?)",
                )
                .bind(id)
                .bind(rule.code())
                .bind(reason)
                .bind(new.staff_id)
                .bind(&now)
                .execute(&mut *tx)
                .await?;
            }
        }
        let target = ChangeTarget {
            application_id: id,
            member_id: new.member_id,
            course_id,
            staff_id: new.staff_id,
            now: &now,
        };
        // 같은 사유가 두 번 와도 한 번만 붙인다
        let reasons: BTreeSet<i64> = new
            .priorities
            .iter()
            .filter(|priority| priority.course_id == course_id)
            .flat_map(|priority| priority.reason_ids.iter().copied())
            .collect();
        for reason_id in reasons {
            application_priorities::attach(&mut tx, &target, reason_id).await?;
        }
        submitted.push(Submitted {
            id,
            course_id,
            additional,
        });
    }

    tx.commit().await?;
    Ok(submitted)
}

/// 선발 마감한 강좌 `course_id`에 추가 신청 하나를 받으면 어떻게 되는지(BR-X2).
async fn additional_outcome_for(
    conn: &mut SqliteConnection,
    course_id: i64,
) -> Result<AdditionalOutcome, SubmitError> {
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

/// 접수됨 신청을 지우고 삭제 기록을 남긴다(BR-D1~D3). 규칙 예외도 함께 지워진다.
///
/// 상태를 읽고 지우므로 `BEGIN IMMEDIATE`로 한다.
pub async fn delete(
    pool: &SqlitePool,
    id: i64,
    staff_id: i64,
    reason: Option<&str>,
    now: OffsetDateTime,
) -> Result<(), DeleteApplicationError> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let row: Option<(i64, i64, String)> =
        sqlx::query_as("SELECT member_id, course_id, status FROM applications WHERE id = ?")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
    let Some((member_id, course_id, status)) = row else {
        return Err(DeleteApplicationError::NotFound);
    };
    if status != "submitted" {
        return Err(DeleteApplicationError::NotSubmitted);
    }

    sqlx::query(
        "INSERT INTO application_deletions (member_id, course_id, deleted_by, deleted_at, reason)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(member_id)
    .bind(course_id)
    .bind(staff_id)
    .bind(timestamp_text(now))
    .bind(reason)
    .execute(&mut *tx)
    .await?;
    sqlx::query("DELETE FROM applications WHERE id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
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
struct RuleCourse {
    subject_id: i64,
    schedules: Vec<Schedule>,
}

impl RuleCourse {
    fn for_rules(&self, course_id: i64) -> CourseForRules<'_> {
        CourseForRules {
            course_id,
            subject_id: self.subject_id,
            schedules: &self.schedules,
        }
    }
}

/// `term_id` 학기의 강좌마다 과목과 수업 시간. 강좌 id로 찾는다.
async fn term_courses(
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
async fn drawn_course_ids(
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
async fn closed_course_ids(
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
async fn applied_course_ids(
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

#[cfg(test)]
mod tests {
    use time::macros::datetime;

    use super::*;
    use crate::db::open_temp;
    use crate::db::test_support::{
        insert_application, insert_course, insert_lottery_run, insert_member, insert_schedule,
        insert_staff, insert_subject, insert_term,
    };

    fn now() -> OffsetDateTime {
        datetime!(2026-11-22 01:02:03 UTC)
    }

    /// 학기 둘, 회원, 직원과 강좌 넷을 넣어 둔 DB.
    ///
    /// - 요가 초급: 월 10:00~11:00
    /// - 요가 중급: 화 10:00~11:00 (같은 과목)
    /// - 한글교실: 월 10:30~12:00 (요가 초급과 시간 겹침)
    /// - 국선도: 수 10:00~11:00
    /// - 다른 학기 강좌 하나
    struct Fixture {
        _dir: tempfile::TempDir,
        pool: SqlitePool,
        term_id: i64,
        member_id: i64,
        staff_id: i64,
        yoga_basic: i64,
        yoga_middle: i64,
        hangul: i64,
        kuksundo: i64,
        other_term_course: i64,
    }

    async fn fixture() -> Fixture {
        let (dir, pool) = open_temp().await;
        let term_id = insert_term(&pool, "2027년 1학기", "2027-03-02", "2027-06-30")
            .await
            .unwrap();
        let other_term = insert_term(&pool, "2027년 2학기", "2027-09-01", "2027-12-31")
            .await
            .unwrap();
        let yoga = insert_subject(&pool, "요가", "요가", None).await.unwrap();
        let hangul_subject = insert_subject(&pool, "한글교실", "한글교실", None)
            .await
            .unwrap();
        let kuksundo_subject = insert_subject(&pool, "국선도", "국선도", None)
            .await
            .unwrap();
        let course = async |term_id, subject_id, level, weekday, start, end| {
            let id = insert_course(&pool, term_id, subject_id, level, None)
                .await
                .unwrap();
            insert_schedule(&pool, id, weekday, start, end, None)
                .await
                .unwrap();
            id
        };
        let yoga_basic = course(term_id, yoga, Some("초급"), 1, 600, 660).await;
        let yoga_middle = course(term_id, yoga, Some("중급"), 2, 600, 660).await;
        let hangul = course(term_id, hangul_subject, None, 1, 630, 720).await;
        let kuksundo = course(term_id, kuksundo_subject, None, 3, 600, 660).await;
        let other_term_course = course(other_term, yoga, None, 1, 600, 660).await;
        let member_id = insert_member(&pool, "00001").await.unwrap();
        let staff_id = insert_staff(&pool, "직원1", "staff", None).await.unwrap();
        Fixture {
            _dir: dir,
            pool,
            term_id,
            member_id,
            staff_id,
            yoga_basic,
            yoga_middle,
            hangul,
            kuksundo,
            other_term_course,
        }
    }

    impl Fixture {
        fn applying<'a>(
            &self,
            course_ids: &'a [i64],
            reason: Option<&'a str>,
        ) -> NewApplications<'a> {
            NewApplications {
                term_id: self.term_id,
                member_id: self.member_id,
                course_ids,
                override_reason: reason,
                staff_id: self.staff_id,
                priorities: &[],
            }
        }

        async fn list(&self) -> Vec<ApplicationRecord> {
            list_for_member(&self.pool, self.term_id, self.member_id)
                .await
                .unwrap()
        }

        async fn count(&self, table: &'static str) -> i64 {
            let sql = match table {
                "rule_overrides" => "SELECT COUNT(*) FROM rule_overrides",
                "application_deletions" => "SELECT COUNT(*) FROM application_deletions",
                _ => "SELECT COUNT(*) FROM applications",
            };
            sqlx::query_scalar(sql).fetch_one(&self.pool).await.unwrap()
        }
    }

    #[tokio::test]
    async fn submits_several_courses_at_once() {
        let f = fixture().await;

        let ids = submit(
            &f.pool,
            f.applying(&[f.yoga_basic, f.kuksundo], None),
            now(),
        )
        .await
        .unwrap();

        let list = f.list().await;
        assert_eq!(list.len(), 2);
        assert_eq!(
            list.iter().map(|a| a.id).collect::<Vec<_>>(),
            ids.iter().map(|item| item.id).collect::<Vec<_>>()
        );
        assert!(ids.iter().all(|item| item.additional.is_none()));
        assert_eq!(list[0].course_id, f.yoga_basic);
        assert_eq!(list[0].submitted_at, now());
        assert_eq!(list[0].overridden_rules, []);
    }

    #[tokio::test]
    async fn br_a5_broken_rules_save_nothing() {
        let f = fixture().await;
        submit(&f.pool, f.applying(&[f.yoga_basic], None), now())
            .await
            .unwrap();

        let error = submit(&f.pool, f.applying(&[f.kuksundo, f.hangul], None), now())
            .await
            .unwrap_err();

        let SubmitError::RulesBroken(violations) = error else {
            panic!("{error:?}");
        };
        assert_eq!(
            violations,
            [RuleViolation {
                course_id: f.hangul,
                rule: ApplicationRule::NoTimeOverlap,
                other_course_id: f.yoga_basic,
            }]
        );
        assert_eq!(f.list().await.len(), 1, "국선도도 저장하지 않는다");
    }

    #[tokio::test]
    async fn br_a6_reason_saves_with_rule_overrides() {
        let f = fixture().await;
        submit(&f.pool, f.applying(&[f.yoga_basic], None), now())
            .await
            .unwrap();

        submit(
            &f.pool,
            f.applying(&[f.yoga_middle, f.hangul], Some("강사 허락")),
            now(),
        )
        .await
        .unwrap();

        let list = f.list().await;
        assert_eq!(list[1].overridden_rules, [ApplicationRule::NoSameSubject]);
        assert_eq!(list[2].overridden_rules, [ApplicationRule::NoTimeOverlap]);
        assert_eq!(f.count("rule_overrides").await, 2);
    }

    #[tokio::test]
    async fn reason_without_broken_rule_leaves_no_override() {
        let f = fixture().await;

        submit(&f.pool, f.applying(&[f.kuksundo], Some("사유")), now())
            .await
            .unwrap();

        assert_eq!(f.count("rule_overrides").await, 0);
    }

    #[tokio::test]
    async fn br_a7_duplicate_is_refused_even_with_reason() {
        let f = fixture().await;
        submit(&f.pool, f.applying(&[f.yoga_basic], None), now())
            .await
            .unwrap();

        let error = submit(&f.pool, f.applying(&[f.yoga_basic], Some("사유")), now())
            .await
            .unwrap_err();

        assert!(matches!(error, SubmitError::RulesBroken(_)), "{error:?}");
        assert_eq!(f.count("applications").await, 1);
    }

    #[tokio::test]
    async fn course_of_other_term_or_missing_is_not_found() {
        let f = fixture().await;

        let error = submit(
            &f.pool,
            f.applying(&[f.kuksundo, f.other_term_course, 999], None),
            now(),
        )
        .await
        .unwrap_err();

        let SubmitError::CourseNotFound(missing) = error else {
            panic!("{error:?}");
        };
        assert_eq!(missing, [f.other_term_course, 999]);
    }

    #[tokio::test]
    async fn br_l3_drawn_course_takes_no_new_application() {
        let f = fixture().await;
        let other_member = insert_member(&f.pool, "00002").await.unwrap();
        insert_application(&f.pool, other_member, f.yoga_basic, "selected", f.staff_id)
            .await
            .unwrap();
        insert_lottery_run(&f.pool, f.yoga_basic, f.staff_id)
            .await
            .unwrap();
        let course_ids = [f.kuksundo, f.yoga_basic];

        let result = submit(&f.pool, f.applying(&course_ids, None), now()).await;

        assert!(
            matches!(&result, Err(SubmitError::CourseDrawn(ids)) if *ids == [f.yoga_basic]),
            "{result:?}"
        );
        assert_eq!(f.count("applications").await, 1);
    }

    #[tokio::test]
    async fn missing_member_is_not_found() {
        let f = fixture().await;
        let course_ids = [f.kuksundo];
        let new = NewApplications {
            member_id: 999,
            ..f.applying(&course_ids, None)
        };

        let error = submit(&f.pool, new, now()).await.unwrap_err();

        assert!(matches!(error, SubmitError::MemberNotFound), "{error:?}");
    }

    #[tokio::test]
    async fn withdrawn_application_is_left_out_of_rules_and_counts() {
        let f = fixture().await;
        insert_application(&f.pool, f.member_id, f.yoga_basic, "withdrawn", f.staff_id)
            .await
            .unwrap();

        submit(&f.pool, f.applying(&[f.yoga_basic], None), now())
            .await
            .unwrap();

        assert_eq!(f.list().await.len(), 1);
        assert_eq!(
            count_by_course(&f.pool, f.term_id).await.unwrap(),
            [CourseCount {
                course_id: f.yoga_basic,
                count: 1,
            }]
        );
    }

    #[tokio::test]
    async fn counts_valid_applications_by_course_in_term() {
        let f = fixture().await;
        let other_member = insert_member(&f.pool, "00002").await.unwrap();
        submit(
            &f.pool,
            f.applying(&[f.yoga_basic, f.kuksundo], None),
            now(),
        )
        .await
        .unwrap();
        let course_ids = [f.yoga_basic];
        let other = NewApplications {
            member_id: other_member,
            ..f.applying(&course_ids, None)
        };
        submit(&f.pool, other, now()).await.unwrap();

        assert_eq!(
            count_by_course(&f.pool, f.term_id).await.unwrap(),
            [
                CourseCount {
                    course_id: f.yoga_basic,
                    count: 2,
                },
                CourseCount {
                    course_id: f.kuksundo,
                    count: 1,
                },
            ]
        );
    }

    #[tokio::test]
    async fn br_d2_d3_delete_leaves_deletion_record_and_frees_rules() {
        let f = fixture().await;
        let ids = submit(&f.pool, f.applying(&[f.yoga_basic], None), now())
            .await
            .unwrap();

        delete(&f.pool, ids[0].id, f.staff_id, Some("잘못 고름"), now())
            .await
            .unwrap();

        assert_eq!(f.list().await, []);
        assert_eq!(f.count("application_deletions").await, 1);
        let (member_id, course_id, reason): (i64, i64, Option<String>) =
            sqlx::query_as("SELECT member_id, course_id, reason FROM application_deletions")
                .fetch_one(&f.pool)
                .await
                .unwrap();
        assert_eq!((member_id, course_id), (f.member_id, f.yoga_basic));
        assert_eq!(reason.as_deref(), Some("잘못 고름"));
        submit(&f.pool, f.applying(&[f.hangul], None), now())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn br_d1_only_submitted_application_is_deleted() {
        let f = fixture().await;
        let id = insert_application(&f.pool, f.member_id, f.yoga_basic, "selected", f.staff_id)
            .await
            .unwrap();

        let error = delete(&f.pool, id, f.staff_id, None, now())
            .await
            .unwrap_err();

        assert!(
            matches!(error, DeleteApplicationError::NotSubmitted),
            "{error:?}"
        );
        assert_eq!(f.count("application_deletions").await, 0);
    }

    #[tokio::test]
    async fn deleting_missing_application_is_not_found() {
        let f = fixture().await;

        let error = delete(&f.pool, 999, f.staff_id, None, now())
            .await
            .unwrap_err();

        assert!(
            matches!(error, DeleteApplicationError::NotFound),
            "{error:?}"
        );
    }

    /// 추가 신청(BR-X1~X3). 정원과 신청자 수를 정해 추첨하고 마감한 강좌 하나에 받는다.
    mod additional {
        use super::*;
        use crate::db::selection_changes;
        use crate::db::selection_results::course_result;
        use crate::db::test_support::{DrawnCourse, insert_drawn_course};

        async fn closed_course(
            capacity: i64,
            applicants: usize,
        ) -> (tempfile::TempDir, SqlitePool, DrawnCourse) {
            let (dir, pool) = open_temp().await;
            let course = insert_drawn_course(&pool, capacity, applicants, true).await;
            (dir, pool, course)
        }

        /// 새 회원 하나가 그 강좌에 추가 신청한다.
        async fn apply_new_member(
            pool: &SqlitePool,
            course: &DrawnCourse,
            member_no: &str,
            now: OffsetDateTime,
        ) -> Result<Vec<Submitted>, SubmitError> {
            let member_id = insert_member(pool, member_no).await.unwrap();
            submit(
                pool,
                NewApplications {
                    term_id: course.term_id,
                    member_id,
                    course_ids: &[course.course_id],
                    override_reason: None,
                    staff_id: course.staff_id,
                    priorities: &[],
                },
                now,
            )
            .await
        }

        fn outcome(submitted: &[Submitted]) -> Option<AdditionalOutcome> {
            submitted[0].additional
        }

        async fn waitlist_numbers(pool: &SqlitePool, course_id: i64) -> Vec<usize> {
            course_result(pool, course_id)
                .await
                .unwrap()
                .waitlisted
                .iter()
                .map(|entry| entry.waitlist_number)
                .collect()
        }

        #[tokio::test]
        async fn br_x2_open_seat_without_waiting_is_selected_at_once() {
            let (_dir, pool, course) = closed_course(3, 2).await;

            let submitted = apply_new_member(&pool, &course, "10001", now())
                .await
                .unwrap();

            assert_eq!(outcome(&submitted), Some(AdditionalOutcome::Selected));
            let result = course_result(&pool, course.course_id).await.unwrap();
            assert_eq!(result.selected.len(), 3);
            assert_eq!(result.open_seats, 0);
            // 추첨 순서가 없어도 자리 보장이 아니다
            let last = result.selected.last().unwrap();
            assert_eq!(last.member.application_id, submitted[0].id);
            assert!(!last.guaranteed);
            assert_eq!(last.waitlist_number, None);
        }

        #[tokio::test]
        async fn br_x2_full_course_waits_after_last_number() {
            let (_dir, pool, course) = closed_course(2, 4).await;
            assert_eq!(waitlist_numbers(&pool, course.course_id).await, [1, 2]);

            let first = apply_new_member(&pool, &course, "10001", now())
                .await
                .unwrap();
            let second = apply_new_member(&pool, &course, "10002", now())
                .await
                .unwrap();

            assert_eq!(outcome(&first), Some(AdditionalOutcome::Waitlisted(3)));
            assert_eq!(outcome(&second), Some(AdditionalOutcome::Waitlisted(4)));
            assert_eq!(
                waitlist_numbers(&pool, course.course_id).await,
                [1, 2, 3, 4]
            );
        }

        #[tokio::test]
        async fn br_x2_open_seat_with_waiting_members_still_waits() {
            let (_dir, pool, course) = closed_course(2, 4).await;
            let result = course_result(&pool, course.course_id).await.unwrap();
            let winner = result.selected[0].member.application_id;
            selection_changes::withdraw(&pool, winner, None, course.staff_id, now())
                .await
                .unwrap();

            let submitted = apply_new_member(&pool, &course, "10001", now())
                .await
                .unwrap();

            // 빈자리는 연락을 기다리는 대기자 차례다 (BR-W1)
            assert_eq!(outcome(&submitted), Some(AdditionalOutcome::Waitlisted(3)));
        }

        #[tokio::test]
        async fn br_w6_number_of_withdrawn_waiting_member_is_not_reused() {
            let (_dir, pool, course) = closed_course(2, 3).await;
            let result = course_result(&pool, course.course_id).await.unwrap();
            let waiting = result.waitlisted[0].member.application_id;
            selection_changes::withdraw(&pool, waiting, None, course.staff_id, now())
                .await
                .unwrap();

            let submitted = apply_new_member(&pool, &course, "10001", now())
                .await
                .unwrap();

            assert_eq!(outcome(&submitted), Some(AdditionalOutcome::Waitlisted(2)));
        }

        #[tokio::test]
        async fn br_x2_later_of_two_at_once_waits_for_last_seat() {
            let (_dir, pool, course) = closed_course(2, 1).await;

            let (first, second) = tokio::join!(
                apply_new_member(&pool, &course, "10001", now()),
                apply_new_member(&pool, &course, "10002", now()),
            );

            let mut outcomes = [outcome(&first.unwrap()), outcome(&second.unwrap())];
            outcomes.sort_by_key(|outcome| format!("{outcome:?}"));
            assert_eq!(
                outcomes,
                [
                    Some(AdditionalOutcome::Selected),
                    Some(AdditionalOutcome::Waitlisted(1))
                ]
            );
        }

        #[tokio::test]
        async fn br_x1_term_end_date_is_last_day() {
            let (_dir, pool, course) = closed_course(3, 1).await;
            // 학기 종료일 2027-06-30. 한국 시간으로 그날 밤과 다음 날 아침
            let last_day = datetime!(2027-06-30 14:59:59 UTC);
            let next_day = datetime!(2027-06-30 15:00:00 UTC);

            apply_new_member(&pool, &course, "10001", last_day)
                .await
                .unwrap();
            let result = apply_new_member(&pool, &course, "10002", next_day).await;

            assert!(
                matches!(&result, Err(SubmitError::TermEnded(ids)) if *ids == [course.course_id]),
                "{result:?}"
            );
        }

        #[tokio::test]
        async fn br_x3_rules_apply_to_additional_application() {
            let (_dir, pool, course) = closed_course(3, 1).await;
            // 이미 이 강좌에 당첨된 회원 (BR-A4)
            let member_id: i64 =
                sqlx::query_scalar("SELECT member_id FROM applications WHERE course_id = ?")
                    .bind(course.course_id)
                    .fetch_one(&pool)
                    .await
                    .unwrap();

            let result = submit(
                &pool,
                NewApplications {
                    term_id: course.term_id,
                    member_id,
                    course_ids: &[course.course_id],
                    override_reason: Some("사유"),
                    staff_id: course.staff_id,
                    priorities: &[],
                },
                now(),
            )
            .await;

            assert!(
                matches!(&result, Err(SubmitError::RulesBroken(_))),
                "{result:?}"
            );
        }

        #[tokio::test]
        async fn br_x2_additional_record_cannot_change() {
            let (_dir, pool, course) = closed_course(3, 1).await;
            apply_new_member(&pool, &course, "10001", now())
                .await
                .unwrap();

            let update = sqlx::query("UPDATE additional_applications SET waitlist_number = 9")
                .execute(&pool)
                .await;
            let delete = sqlx::query("DELETE FROM additional_applications")
                .execute(&pool)
                .await;

            assert!(update.is_err());
            assert!(delete.is_err());
        }
    }
}
