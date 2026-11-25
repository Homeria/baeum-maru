use super::*;

fn time(text: &str) -> ClassTime {
    ClassTime::parse(text).unwrap()
}

fn row(weekday: Weekday, start: &'static str, end: &'static str) -> ScheduleInput<'static> {
    ScheduleInput {
        weekday,
        start,
        end,
        room_id: None,
    }
}

fn input() -> CourseInput<'static> {
    CourseInput {
        subject_id: Some(1),
        level: Some("초급"),
        section: Some("1반"),
        capacity: Some(20),
        schedules: vec![
            row(Weekday::Monday, "10:00", "11:00"),
            row(Weekday::Wednesday, "10:00", "11:00"),
        ],
    }
}

#[test]
fn class_time_reads_hh_mm() {
    assert_eq!(time("00:00").minutes(), 0);
    assert_eq!(time("09:05").minutes(), 545);
    assert_eq!(time("23:59").minutes(), 1439);
    assert_eq!(time("13:30").to_string(), "13:30");
}

#[test]
fn class_time_rejects_wrong_shape() {
    for bad in ["", "9:00", "24:00", "10:60", "10-00", "가나:다라", "10:000"] {
        assert_eq!(ClassTime::parse(bad), None, "{bad}");
    }
}

#[test]
fn class_time_from_minutes_stays_in_one_day() {
    assert_eq!(ClassTime::from_minutes(600), Some(time("10:00")));
    assert_eq!(ClassTime::from_minutes(1440), None);
}

#[test]
fn capacity_is_1_to_200() {
    assert_eq!(Capacity::new(1).unwrap().value(), 1);
    assert_eq!(Capacity::new(200).unwrap().value(), 200);
    for bad in [0, 201, -1, i64::MAX] {
        assert_eq!(
            Capacity::new(bad),
            Err(CourseError::CapacityOutOfRange),
            "{bad}"
        );
    }
}

#[test]
fn level_and_section_are_optional() {
    assert_eq!(LevelName::parse("  "), Ok(None));
    assert_eq!(SectionName::parse(""), Ok(None));
    assert_eq!(
        LevelName::parse(" 초급 ").unwrap().unwrap().as_str(),
        "초급"
    );
    assert_eq!(
        SectionName::parse(&"가".repeat(21)),
        Err(CourseError::SectionTooLong)
    );
}

#[test]
fn br_t4_end_must_be_after_start() {
    let same = Schedule::new(Weekday::Monday, time("10:00"), time("10:00"), None);
    let reversed = Schedule::new(Weekday::Monday, time("11:00"), time("10:00"), None);

    assert_eq!(same, None);
    assert_eq!(reversed, None);
}

#[test]
fn overlaps_only_on_same_weekday() {
    let monday = Schedule::new(Weekday::Monday, time("10:00"), time("11:00"), None).unwrap();
    let tuesday = Schedule::new(Weekday::Tuesday, time("10:00"), time("11:00"), None).unwrap();

    assert!(!monday.overlaps(tuesday));
}

#[test]
fn overlaps_when_times_cross() {
    let first = Schedule::new(Weekday::Monday, time("10:00"), time("11:00"), None).unwrap();
    let second = Schedule::new(Weekday::Monday, time("10:30"), time("12:00"), None).unwrap();
    let inside = Schedule::new(Weekday::Monday, time("10:15"), time("10:45"), None).unwrap();

    assert!(first.overlaps(second));
    assert!(second.overlaps(first));
    assert!(first.overlaps(inside));
}

#[test]
fn back_to_back_does_not_overlap() {
    let first = Schedule::new(Weekday::Monday, time("10:00"), time("11:00"), None).unwrap();
    let next = Schedule::new(Weekday::Monday, time("11:00"), time("12:00"), None).unwrap();

    assert!(!first.overlaps(next));
}

#[test]
fn parse_keeps_every_field() {
    let course = NewCourse::parse(input()).unwrap();

    assert_eq!(course.subject_id(), 1);
    assert_eq!(course.level().map(LevelName::as_str), Some("초급"));
    assert_eq!(course.section().map(SectionName::as_str), Some("1반"));
    assert_eq!(course.capacity().value(), 20);
    assert_eq!(course.schedules().len(), 2);
    assert_eq!(course.schedules()[1].weekday(), Weekday::Wednesday);
}

#[test]
fn rooms_can_differ_by_row() {
    let course = NewCourse::parse(CourseInput {
        schedules: vec![
            ScheduleInput {
                room_id: Some(5),
                ..row(Weekday::Monday, "10:00", "11:00")
            },
            ScheduleInput {
                room_id: Some(7),
                ..row(Weekday::Wednesday, "10:00", "11:00")
            },
        ],
        ..input()
    })
    .unwrap();

    let rooms: Vec<Option<i64>> = course.schedules().iter().map(|s| s.room_id()).collect();
    assert_eq!(rooms, [Some(5), Some(7)]);
}

#[test]
fn br_t4_needs_at_least_one_schedule() {
    let errors = NewCourse::parse(CourseInput {
        schedules: Vec::new(),
        ..input()
    })
    .unwrap_err();

    assert_eq!(errors, [CourseError::NoSchedule]);
}

#[test]
fn br_t4_reports_each_bad_row() {
    let errors = NewCourse::parse(CourseInput {
        schedules: vec![
            row(Weekday::Monday, "10:00", "11:00"),
            row(Weekday::Tuesday, "", "11:00"),
            row(Weekday::Wednesday, "11:00", "10:00"),
        ],
        ..input()
    })
    .unwrap_err();

    assert_eq!(
        errors,
        [
            CourseError::InvalidTime { row: 2 },
            CourseError::EndNotAfterStart { row: 3 }
        ]
    );
}

#[test]
fn br_t4_rows_in_one_course_must_not_overlap() {
    let errors = NewCourse::parse(CourseInput {
        schedules: vec![
            row(Weekday::Monday, "10:00", "11:00"),
            row(Weekday::Wednesday, "10:00", "11:00"),
            row(Weekday::Monday, "10:30", "11:30"),
        ],
        ..input()
    })
    .unwrap_err();

    assert_eq!(
        errors,
        [CourseError::SchedulesOverlap {
            first: 1,
            second: 3
        }]
    );
}

#[test]
fn collects_every_error_in_screen_order() {
    let errors = NewCourse::parse(CourseInput {
        subject_id: None,
        level: Some(&"가".repeat(21)),
        section: None,
        capacity: None,
        schedules: Vec::new(),
    })
    .unwrap_err();

    assert_eq!(
        errors,
        [
            CourseError::MissingSubject,
            CourseError::LevelTooLong,
            CourseError::MissingCapacity,
            CourseError::NoSchedule
        ]
    );
}

#[test]
fn error_message_names_the_row() {
    assert_eq!(
        CourseError::EndNotAfterStart { row: 2 }.to_string(),
        "2번째 수업 시간: 종료 시각은 시작 시각보다 늦어야 합니다"
    );
}
