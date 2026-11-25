use time::Weekday;

use super::*;
use crate::course::ClassTime;

fn schedule(weekday: Weekday, start: &str, end: &str) -> Schedule {
    let time = |text| ClassTime::parse(text).unwrap();
    Schedule::new(weekday, time(start), time(end), None).unwrap()
}

fn course(course_id: i64, subject_id: i64, schedules: &[Schedule]) -> CourseForRules<'_> {
    CourseForRules {
        course_id,
        subject_id,
        schedules,
    }
}

fn violation(course_id: i64, rule: ApplicationRule, other_course_id: i64) -> RuleViolation {
    RuleViolation {
        course_id,
        rule,
        other_course_id,
    }
}

#[test]
fn different_time_and_subject_breaks_nothing() {
    let monday = [schedule(Weekday::Monday, "10:00", "11:00")];
    let tuesday = [schedule(Weekday::Tuesday, "10:00", "11:00")];

    let violations = check_application(&[course(1, 10, &monday)], &[course(2, 20, &tuesday)]);

    assert_eq!(violations, []);
}

#[test]
fn br_a4_same_course_again_is_only_duplicate() {
    let monday = [schedule(Weekday::Monday, "10:00", "11:00")];
    let yoga = course(1, 10, &monday);

    assert_eq!(
        check_application(&[yoga], &[yoga]),
        [violation(1, ApplicationRule::NoDuplicate, 1)]
    );
}

#[test]
fn br_a4_same_course_chosen_twice_is_duplicate() {
    let monday = [schedule(Weekday::Monday, "10:00", "11:00")];
    let yoga = course(1, 10, &monday);

    assert_eq!(
        check_application(&[], &[yoga, yoga]),
        [violation(1, ApplicationRule::NoDuplicate, 1)]
    );
}

#[test]
fn br_a1_compares_every_schedule_of_twice_weekly_course() {
    let mon_wed = [
        schedule(Weekday::Monday, "10:00", "11:00"),
        schedule(Weekday::Wednesday, "10:00", "11:00"),
    ];
    let wednesday = [schedule(Weekday::Wednesday, "10:30", "12:00")];

    assert_eq!(
        check_application(&[course(1, 10, &mon_wed)], &[course(2, 20, &wednesday)]),
        [violation(2, ApplicationRule::NoTimeOverlap, 1)]
    );
}

#[test]
fn br_a1_back_to_back_courses_do_not_overlap() {
    let first = [schedule(Weekday::Monday, "10:00", "11:00")];
    let next = [schedule(Weekday::Monday, "11:00", "12:00")];

    assert_eq!(
        check_application(&[course(1, 10, &first)], &[course(2, 20, &next)]),
        []
    );
}

#[test]
fn br_a2_same_subject_at_different_level_is_broken() {
    let monday = [schedule(Weekday::Monday, "10:00", "11:00")];
    let tuesday = [schedule(Weekday::Tuesday, "10:00", "11:00")];

    assert_eq!(
        check_application(&[course(1, 10, &monday)], &[course(2, 10, &tuesday)]),
        [violation(2, ApplicationRule::NoSameSubject, 1)]
    );
}

#[test]
fn reports_every_broken_rule_and_every_other_course() {
    let monday = [schedule(Weekday::Monday, "10:00", "11:00")];
    let applied = [course(1, 10, &monday), course(2, 20, &monday)];

    assert_eq!(
        check_application(&applied, &[course(3, 10, &monday)]),
        [
            violation(3, ApplicationRule::NoTimeOverlap, 1),
            violation(3, ApplicationRule::NoSameSubject, 1),
            violation(3, ApplicationRule::NoTimeOverlap, 2),
        ]
    );
}

#[test]
fn chosen_courses_are_compared_with_each_other() {
    let monday = [schedule(Weekday::Monday, "10:00", "11:00")];

    assert_eq!(
        check_application(&[], &[course(1, 10, &monday), course(2, 20, &monday)]),
        [violation(2, ApplicationRule::NoTimeOverlap, 1)]
    );
}

#[test]
fn reason_is_trimmed_and_up_to_100_chars() {
    assert_eq!(Reason::parse("  "), Ok(None));
    assert_eq!(
        Reason::parse(" 강사 허락 ").unwrap().unwrap().as_str(),
        "강사 허락"
    );
    assert!(Reason::parse(&"가".repeat(100)).unwrap().is_some());
    assert_eq!(Reason::parse(&"가".repeat(101)), Err(ReasonError::TooLong));
}

#[test]
fn rule_code_round_trips() {
    for rule in ApplicationRule::ALL {
        assert_eq!(ApplicationRule::from_code(rule.code()), Some(rule));
    }
    assert_eq!(ApplicationRule::from_code("unknown"), None);
}

#[test]
fn br_a7_only_duplicate_cannot_be_overridden() {
    assert!(!ApplicationRule::NoDuplicate.can_override());
    assert!(ApplicationRule::NoTimeOverlap.can_override());
    assert!(ApplicationRule::NoSameSubject.can_override());
}
