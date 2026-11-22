//! 강좌와 수업 시간 저장, 목록, 고치기, 지우기.
//!
//! 강좌 한 줄과 수업 시간 여러 줄은 트랜잭션 하나로 함께 쓴다. 중간에 실패하면 모두 되돌린다.

use std::collections::HashMap;

use baeum_domain::{ClassTime, LevelName, NewCourse, Schedule, SectionName, weekday_from_number};
use sqlx::{SqliteConnection, SqlitePool};
use time::{OffsetDateTime, Weekday};

use super::{is_unique_violation, timestamp_text};

/// 강좌가 가리키는 것 중 DB에 없는 것. 그 사이 다른 직원이 지웠을 수 있다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum MissingReference {
    #[error("목록에 없는 과목입니다. 다시 골라 주세요")]
    Subject,
    /// `row`는 요청의 몇 번째 수업 시간인지(1부터)다.
    #[error("{row}번째 수업 시간: 목록에 없는 강의실입니다. 다시 골라 주세요")]
    Room { row: usize },
}

/// 강좌를 저장하지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum SaveCourseError {
    /// 강좌를 넣을 학기가 없다.
    #[error("학기를 찾지 못했습니다")]
    TermNotFound,
    /// 고른 과목이나 강의실이 없다. 없는 것을 모두 모았다.
    #[error("목록에 없는 과목이나 강의실을 골랐습니다")]
    MissingReferences(Vec<MissingReference>),
    /// BR-T3: 같은 학기 안에서 과목·수준·반이 모두 같은 강좌는 하나뿐이다.
    #[error("같은 학기에 같은 강좌가 있습니다")]
    Duplicate,
    #[error("강좌를 저장하지 못했습니다: {0}")]
    Db(sqlx::Error),
}

impl From<sqlx::Error> for SaveCourseError {
    fn from(error: sqlx::Error) -> Self {
        // UNIQUE는 courses_unique_in_term 하나뿐이다 (course_schedules에는 없다)
        if is_unique_violation(&error) {
            SaveCourseError::Duplicate
        } else {
            SaveCourseError::Db(error)
        }
    }
}

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
pub async fn insert(
    pool: &SqlitePool,
    term_id: i64,
    course: &NewCourse,
    now: OffsetDateTime,
) -> Result<i64, SaveCourseError> {
    let mut tx = pool.begin().await?;
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
pub async fn update(
    pool: &SqlitePool,
    id: i64,
    course: &NewCourse,
    now: OffsetDateTime,
) -> Result<Option<CourseRecord>, SaveCourseError> {
    let mut tx = pool.begin().await?;
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
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<bool, sqlx::Error> {
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

#[cfg(test)]
mod tests {
    use baeum_domain::{CourseInput, ScheduleInput};
    use time::macros::datetime;

    use super::*;
    use crate::db::open_temp;
    use crate::db::test_support::{insert_room, insert_subject, insert_term};

    fn now() -> OffsetDateTime {
        datetime!(2026-11-15 01:02:03 UTC)
    }

    fn time(text: &str) -> ClassTime {
        ClassTime::parse(text).unwrap()
    }

    /// 학기 둘, 과목 둘(요가, 국선도), 강의실 하나를 넣어 둔 DB.
    struct Fixture {
        _dir: tempfile::TempDir,
        pool: SqlitePool,
        term_id: i64,
        other_term_id: i64,
        yoga_id: i64,
        kuksundo_id: i64,
        room_id: i64,
    }

    async fn fixture() -> Fixture {
        let (dir, pool) = open_temp().await;
        let term_id = insert_term(&pool, "2027년 1학기", "2027-03-02", "2027-06-30")
            .await
            .unwrap();
        let other_term_id = insert_term(&pool, "2027년 2학기", "2027-09-01", "2027-12-31")
            .await
            .unwrap();
        let yoga_id = insert_subject(&pool, "요가", "요가", None).await.unwrap();
        let kuksundo_id = insert_subject(&pool, "국선도", "국선도", None)
            .await
            .unwrap();
        let room_id = insert_room(&pool, Some("본관"), Some(2), "201호")
            .await
            .unwrap();
        Fixture {
            _dir: dir,
            pool,
            term_id,
            other_term_id,
            yoga_id,
            kuksundo_id,
            room_id,
        }
    }

    fn new_course(
        subject_id: i64,
        level: Option<&str>,
        schedules: Vec<ScheduleInput<'_>>,
    ) -> NewCourse {
        NewCourse::parse(CourseInput {
            subject_id: Some(subject_id),
            level,
            section: None,
            capacity: Some(20),
            schedules,
        })
        .unwrap()
    }

    fn row(weekday: Weekday, room_id: Option<i64>) -> ScheduleInput<'static> {
        ScheduleInput {
            weekday,
            start: "10:00",
            end: "11:00",
            room_id,
        }
    }

    #[tokio::test]
    async fn saves_course_with_schedules_in_weekday_order() {
        let f = fixture().await;
        let course = new_course(
            f.yoga_id,
            Some("초급"),
            vec![
                row(Weekday::Wednesday, None),
                row(Weekday::Monday, Some(f.room_id)),
            ],
        );

        let id = insert(&f.pool, f.term_id, &course, now()).await.unwrap();

        assert_eq!(
            get(&f.pool, id).await.unwrap(),
            Some(CourseRecord {
                id,
                term_id: f.term_id,
                subject_id: f.yoga_id,
                subject_name: "요가".to_owned(),
                level: Some("초급".to_owned()),
                section: None,
                capacity: 20,
                schedules: vec![
                    ScheduleRecord {
                        weekday: Weekday::Monday,
                        start: time("10:00"),
                        end: time("11:00"),
                        room: Some(ScheduleRoom {
                            id: f.room_id,
                            building: "본관".to_owned(),
                            name: "201호".to_owned(),
                        }),
                    },
                    ScheduleRecord {
                        weekday: Weekday::Wednesday,
                        start: time("10:00"),
                        end: time("11:00"),
                        room: None,
                    },
                ],
            })
        );
    }

    #[tokio::test]
    async fn lists_only_term_courses_by_subject_then_registration() {
        let f = fixture().await;
        for (term_id, subject_id, level) in [
            (f.term_id, f.yoga_id, "초급"),
            (f.term_id, f.yoga_id, "중급"),
            (f.term_id, f.kuksundo_id, "초급"),
            (f.other_term_id, f.yoga_id, "초급"),
        ] {
            let course = new_course(subject_id, Some(level), vec![row(Weekday::Monday, None)]);
            insert(&f.pool, term_id, &course, now()).await.unwrap();
        }

        let names: Vec<String> = list(&f.pool, f.term_id)
            .await
            .unwrap()
            .into_iter()
            .map(|course| format!("{} {}", course.subject_name, course.level.unwrap()))
            .collect();

        assert_eq!(names, ["국선도 초급", "요가 초급", "요가 중급"]);
    }

    #[tokio::test]
    async fn br_t3_same_course_in_term_is_duplicate() {
        let f = fixture().await;
        let course = new_course(f.yoga_id, None, vec![row(Weekday::Monday, None)]);
        insert(&f.pool, f.term_id, &course, now()).await.unwrap();

        let error = insert(&f.pool, f.term_id, &course, now())
            .await
            .unwrap_err();

        assert!(matches!(error, SaveCourseError::Duplicate), "{error}");
        // 다른 학기에는 같은 강좌를 넣을 수 있다
        insert(&f.pool, f.other_term_id, &course, now())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn missing_subject_and_rooms_are_all_reported() {
        let f = fixture().await;
        let course = new_course(
            999,
            None,
            vec![
                row(Weekday::Monday, Some(f.room_id)),
                row(Weekday::Wednesday, Some(998)),
            ],
        );

        let error = insert(&f.pool, f.term_id, &course, now())
            .await
            .unwrap_err();

        let SaveCourseError::MissingReferences(missing) = error else {
            panic!("{error}");
        };
        assert_eq!(
            missing,
            [MissingReference::Subject, MissingReference::Room { row: 2 }]
        );
        assert!(list(&f.pool, f.term_id).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn unknown_term_is_told_apart() {
        let f = fixture().await;
        let course = new_course(f.yoga_id, None, vec![row(Weekday::Monday, None)]);

        let error = insert(&f.pool, 999, &course, now()).await.unwrap_err();

        assert!(matches!(error, SaveCourseError::TermNotFound), "{error}");
    }

    #[tokio::test]
    async fn update_replaces_all_schedules() {
        let f = fixture().await;
        let before = new_course(
            f.yoga_id,
            None,
            vec![row(Weekday::Monday, None), row(Weekday::Wednesday, None)],
        );
        let id = insert(&f.pool, f.term_id, &before, now()).await.unwrap();

        let after = new_course(
            f.kuksundo_id,
            Some("중급"),
            vec![row(Weekday::Friday, Some(f.room_id))],
        );
        let record = update(&f.pool, id, &after, now()).await.unwrap().unwrap();

        assert_eq!(record.subject_name, "국선도");
        assert_eq!(record.level.as_deref(), Some("중급"));
        let weekdays: Vec<Weekday> = record.schedules.iter().map(|s| s.weekday).collect();
        assert_eq!(weekdays, [Weekday::Friday]);
        assert!(update(&f.pool, 999, &after, now()).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn deletes_course() {
        let f = fixture().await;
        let course = new_course(f.yoga_id, None, vec![row(Weekday::Monday, None)]);
        let id = insert(&f.pool, f.term_id, &course, now()).await.unwrap();

        assert!(delete(&f.pool, id).await.unwrap());
        assert!(!delete(&f.pool, id).await.unwrap());
        assert!(get(&f.pool, id).await.unwrap().is_none());
    }
}
