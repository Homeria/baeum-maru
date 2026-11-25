//! 강좌 SQL 테스트.

use baeum_domain::{ClassTime, NewCourse};
use baeum_domain::{CourseInput, ScheduleInput};
use sqlx::SqlitePool;
use time::macros::datetime;
use time::{OffsetDateTime, Weekday};

use crate::db::test_support::{insert_room, insert_subject, insert_term};
use crate::features::courses::repository::*;
use crate::infra::db::open_temp;

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

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn saves_all_when_many_save_at_once() {
    let f = fixture().await;
    let mut saves = tokio::task::JoinSet::new();
    for number in 0..16 {
        let pool = f.pool.clone();
        let (term_id, subject_id) = (f.term_id, f.yoga_id);
        saves.spawn(async move {
            let level = format!("{number}반");
            let course = new_course(subject_id, Some(&level), vec![row(Weekday::Monday, None)]);
            insert(&pool, term_id, &course, now()).await
        });
    }

    while let Some(saved) = saves.join_next().await {
        saved.unwrap().unwrap();
    }
    assert_eq!(list(&f.pool, f.term_id).await.unwrap().len(), 16);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn updates_all_when_many_update_at_once() {
    let f = fixture().await;
    let mut ids = Vec::new();
    for number in 0..16 {
        let level = format!("{number}반");
        let course = new_course(f.yoga_id, Some(&level), vec![row(Weekday::Monday, None)]);
        ids.push(insert(&f.pool, f.term_id, &course, now()).await.unwrap());
    }

    let mut updates = tokio::task::JoinSet::new();
    for (number, id) in ids.into_iter().enumerate() {
        let pool = f.pool.clone();
        let subject_id = f.kuksundo_id;
        updates.spawn(async move {
            let level = format!("{number}반");
            let course = new_course(subject_id, Some(&level), vec![row(Weekday::Friday, None)]);
            update(&pool, id, &course, now()).await
        });
    }

    while let Some(updated) = updates.join_next().await {
        assert!(updated.unwrap().unwrap().is_some());
    }
    let courses = list(&f.pool, f.term_id).await.unwrap();
    assert!(
        courses
            .iter()
            .all(|course| course.subject_id == f.kuksundo_id)
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

#[test]
fn course_name_skips_missing_parts() {
    assert_eq!(
        course_name("요가", Some("초급"), Some("1반")),
        "요가 초급 1반"
    );
    assert_eq!(course_name("서예", None, Some("A")), "서예 A");
    assert_eq!(course_name("탁구", None, None), "탁구");
}
