//! 강좌와 수업 시간 저장, 목록, 고치기, 지우기.
//!
//! 강좌 한 줄과 수업 시간 여러 줄은 트랜잭션 하나로 함께 쓴다. 중간에 실패하면 모두 되돌린다.

use std::collections::HashMap;

use baeum_domain::{ClassTime, LevelName, NewCourse, Schedule, SectionName, weekday_from_number};
use sqlx::{SqliteConnection, SqlitePool};
use time::{OffsetDateTime, Weekday};

use super::error::{MissingReference, SaveCourseError};
use crate::infra::db::{DeleteError, timestamp_text};

/// DB에 저장된 강좌 하나와 그 수업 시간.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CourseRecord {
    pub id: i64,
    pub term_id: i64,
    pub subject_id: i64,
    pub subject_name: String,
    pub level: Option<String>,
    pub section: Option<String>,
    pub capacity: u16,
    /// 요일, 시작 시각 순서.
    pub schedules: Vec<ScheduleRecord>,
}

/// 저장된 수업 시간 한 줄.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleRecord {
    pub weekday: Weekday,
    pub start: ClassTime,
    pub end: ClassTime,
    pub room: Option<ScheduleRoom>,
}

/// 수업 시간에 붙은 강의실. 목록에 보여 줄 만큼만.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleRoom {
    pub id: i64,
    pub building: String,
    pub name: String,
}

/// 확인을 통과한 강좌를 `term_id` 학기에 저장하고 새 `id`를 돌려준다.
///
/// 확인하고 저장하므로 `BEGIN IMMEDIATE`로 한다 (ADR 0005). 보통 `BEGIN`이면 동시에
/// 저장할 때 기다리지 않고 "database is locked"로 실패한다.
pub async fn insert(
    pool: &SqlitePool,
    term_id: i64,
    course: &NewCourse,
    now: OffsetDateTime,
) -> Result<i64, SaveCourseError> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    if !exists(
        &mut tx,
        "SELECT EXISTS (SELECT 1 FROM terms WHERE id = ?)",
        term_id,
    )
    .await?
    {
        return Err(SaveCourseError::TermNotFound);
    }
    check_references(&mut tx, course).await?;

    let now = timestamp_text(now);
    let id = sqlx::query(
        "INSERT INTO courses (term_id, subject_id, level, section, capacity, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(term_id)
    .bind(course.subject_id())
    .bind(course.level().map(LevelName::as_str))
    .bind(course.section().map(SectionName::as_str))
    .bind(course.capacity().value())
    .bind(&now)
    .bind(&now)
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();
    insert_schedules(&mut tx, id, course.schedules()).await?;

    tx.commit().await?;
    Ok(id)
}

/// `id`인 강좌의 칸과 수업 시간을 모두 `course`로 바꾸고 고친 뒤 모습을 돌려준다.
/// 없으면 `None`. 학기는 바꾸지 않는다.
///
/// 수업 시간은 줄을 하나하나 맞추지 않고 모두 지운 뒤 다시 넣는다.
/// [`insert`]처럼 `BEGIN IMMEDIATE`로 한다.
pub async fn update(
    pool: &SqlitePool,
    id: i64,
    course: &NewCourse,
    now: OffsetDateTime,
) -> Result<Option<CourseRecord>, SaveCourseError> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    if !exists(
        &mut tx,
        "SELECT EXISTS (SELECT 1 FROM courses WHERE id = ?)",
        id,
    )
    .await?
    {
        return Ok(None);
    }
    check_references(&mut tx, course).await?;

    sqlx::query(
        "UPDATE courses SET subject_id = ?, level = ?, section = ?, capacity = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(course.subject_id())
    .bind(course.level().map(LevelName::as_str))
    .bind(course.section().map(SectionName::as_str))
    .bind(course.capacity().value())
    .bind(timestamp_text(now))
    .bind(id)
    .execute(&mut *tx)
    .await?;
    sqlx::query("DELETE FROM course_schedules WHERE course_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    insert_schedules(&mut tx, id, course.schedules()).await?;

    tx.commit().await?;
    Ok(get(pool, id).await?)
}

/// `term_id` 학기의 강좌를 모두 읽는다. 과목 이름 순서이고, 같은 과목은 등록한 순서다.
///
/// 수준(초급, 중급…)은 글자 순서로 늘어놓으면 어색해서 등록 순서를 따른다.
pub async fn list(pool: &SqlitePool, term_id: i64) -> Result<Vec<CourseRecord>, sqlx::Error> {
    read(pool, Filter::Term(term_id)).await
}

/// `id`인 강좌를 읽는다. 없으면 `None`.
pub async fn get(pool: &SqlitePool, id: i64) -> Result<Option<CourseRecord>, sqlx::Error> {
    Ok(read(pool, Filter::Course(id)).await?.into_iter().next())
}

/// `id`인 강좌와 그 수업 시간을 지운다. 지웠으면 `true`, 없으면 `false`.
///
/// BR-T6: 신청이나 삭제 기록이 있는 강좌는 [`DeleteError::InUse`]. 두 테이블의 외래 키가 막는다.
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<bool, DeleteError> {
    // 수업 시간은 외래 키의 ON DELETE CASCADE가 함께 지운다
    let result = sqlx::query("DELETE FROM courses WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// `sql`(`SELECT EXISTS (…)`)의 답. sqlx는 코드에 적어 둔 SQL(`&'static str`)만 받는다.
async fn exists(
    conn: &mut SqliteConnection,
    sql: &'static str,
    id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(sql).bind(id).fetch_one(conn).await
}

/// 고른 과목과 강의실이 모두 있는지. 없는 것을 모두 모아 알린다.
///
/// 외래 키 오류로는 과목과 강의실 중 어느 것이 없는지 알 수 없어서 먼저 찾아본다.
async fn check_references(
    conn: &mut SqliteConnection,
    course: &NewCourse,
) -> Result<(), SaveCourseError> {
    let mut missing = Vec::new();
    let subject_sql = "SELECT EXISTS (SELECT 1 FROM subjects WHERE id = ?)";
    if !exists(conn, subject_sql, course.subject_id()).await? {
        missing.push(MissingReference::Subject);
    }
    let room_sql = "SELECT EXISTS (SELECT 1 FROM rooms WHERE id = ?)";
    for (index, schedule) in course.schedules().iter().enumerate() {
        if let Some(room_id) = schedule.room_id()
            && !exists(conn, room_sql, room_id).await?
        {
            missing.push(MissingReference::Room { row: index + 1 });
        }
    }
    if missing.is_empty() {
        Ok(())
    } else {
        Err(SaveCourseError::MissingReferences(missing))
    }
}

async fn insert_schedules(
    conn: &mut SqliteConnection,
    course_id: i64,
    schedules: &[Schedule],
) -> Result<(), sqlx::Error> {
    for schedule in schedules {
        sqlx::query(
            "INSERT INTO course_schedules (course_id, weekday, start_minute, end_minute, room_id)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(course_id)
        .bind(schedule.weekday().number_from_monday())
        .bind(schedule.start().minutes())
        .bind(schedule.end().minutes())
        .bind(schedule.room_id())
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

/// 어떤 강좌들을 읽을지.
#[derive(Clone, Copy)]
enum Filter {
    /// 한 학기의 강좌 모두.
    Term(i64),
    /// 강좌 하나.
    Course(i64),
}

/// 강좌를 읽는 SQL. `$filter`는 `courses`(별칭 `c`)를 거르는 조건이다.
///
/// 컴파일할 때 글자를 이어 붙여 SQL을 코드에 적어 둔 것과 같게 만든다.
macro_rules! courses_sql {
    ($filter:literal) => {
        concat!(
            "SELECT c.id, c.term_id, c.subject_id, s.name AS subject_name,
                    c.level, c.section, c.capacity
             FROM courses c JOIN subjects s ON s.id = c.subject_id
             WHERE ",
            $filter,
            " ORDER BY s.name, c.id"
        )
    };
}

/// 수업 시간을 읽는 SQL. `$filter`는 [`courses_sql`]과 같다.
macro_rules! schedules_sql {
    ($filter:literal) => {
        concat!(
            "SELECT cs.course_id, cs.weekday, cs.start_minute, cs.end_minute,
                    cs.room_id, r.building AS room_building, r.name AS room_name
             FROM course_schedules cs
             JOIN courses c ON c.id = cs.course_id
             LEFT JOIN rooms r ON r.id = cs.room_id
             WHERE ",
            $filter,
            " ORDER BY cs.weekday, cs.start_minute"
        )
    };
}

impl Filter {
    /// (강좌 SQL, 수업 시간 SQL, 거르는 값)
    fn queries(self) -> (&'static str, &'static str, i64) {
        match self {
            Filter::Term(id) => (
                courses_sql!("c.term_id = ?"),
                schedules_sql!("c.term_id = ?"),
                id,
            ),
            Filter::Course(id) => (courses_sql!("c.id = ?"), schedules_sql!("c.id = ?"), id),
        }
    }
}

#[derive(sqlx::FromRow)]
struct CourseRow {
    id: i64,
    term_id: i64,
    subject_id: i64,
    subject_name: String,
    level: Option<String>,
    section: Option<String>,
    capacity: u16,
}

#[derive(sqlx::FromRow)]
struct ScheduleRow {
    course_id: i64,
    weekday: u8,
    start_minute: u16,
    end_minute: u16,
    room_id: Option<i64>,
    room_building: Option<String>,
    room_name: Option<String>,
}

impl TryFrom<ScheduleRow> for ScheduleRecord {
    type Error = sqlx::Error;

    /// 테이블 CHECK가 막는 값이라 실패하면 DB가 망가진 것이다.
    fn try_from(row: ScheduleRow) -> Result<Self, Self::Error> {
        let weekday = weekday_from_number(row.weekday);
        let start = ClassTime::from_minutes(row.start_minute);
        let end = ClassTime::from_minutes(row.end_minute);
        let (Some(weekday), Some(start), Some(end)) = (weekday, start, end) else {
            return Err(sqlx::Error::Decode("수업 시간 값이 범위 밖입니다".into()));
        };
        let room = match (row.room_id, row.room_building, row.room_name) {
            (Some(id), Some(building), Some(name)) => Some(ScheduleRoom { id, building, name }),
            _ => None,
        };
        Ok(ScheduleRecord {
            weekday,
            start,
            end,
            room,
        })
    }
}

/// 강좌들을 읽고, 수업 시간을 따로 읽어 강좌마다 붙인다.
async fn read(pool: &SqlitePool, filter: Filter) -> Result<Vec<CourseRecord>, sqlx::Error> {
    let (courses_sql, schedules_sql, id) = filter.queries();
    let courses: Vec<CourseRow> = sqlx::query_as(courses_sql).bind(id).fetch_all(pool).await?;
    let schedules: Vec<ScheduleRow> = sqlx::query_as(schedules_sql)
        .bind(id)
        .fetch_all(pool)
        .await?;

    let mut by_course: HashMap<i64, Vec<ScheduleRecord>> = HashMap::new();
    for row in schedules {
        let course_id = row.course_id;
        by_course
            .entry(course_id)
            .or_default()
            .push(ScheduleRecord::try_from(row)?);
    }
    Ok(courses
        .into_iter()
        .map(|row| CourseRecord {
            schedules: by_course.remove(&row.id).unwrap_or_default(),
            id: row.id,
            term_id: row.term_id,
            subject_id: row.subject_id,
            subject_name: row.subject_name,
            level: row.level,
            section: row.section,
            capacity: row.capacity,
        })
        .collect())
}

/// 모든 강좌의 화면 이름(과목, 수준, 반). 강좌 id로 찾는다. 예: 백업 목록의 "추첨 직전 · 요가 초급"
pub async fn names(pool: &SqlitePool) -> Result<HashMap<i64, String>, sqlx::Error> {
    let rows: Vec<(i64, String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT c.id, s.name, c.level, c.section
         FROM courses c JOIN subjects s ON s.id = c.subject_id",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, subject, level, section)| {
            let name = course_name(&subject, level.as_deref(), section.as_deref());
            (id, name)
        })
        .collect())
}

/// 화면의 강좌 이름과 같다: 과목, 수준, 반을 띄어 잇는다.
pub(crate) fn course_name(subject: &str, level: Option<&str>, section: Option<&str>) -> String {
    [Some(subject), level, section]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ")
}
