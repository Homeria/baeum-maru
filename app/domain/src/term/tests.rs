use time::Month;

use super::*;

fn date(month: u8, day: u8) -> Date {
    Date::from_calendar_date(2027, Month::try_from(month).unwrap(), day).unwrap()
}

fn input() -> TermInput<'static> {
    TermInput {
        name: "2027년 1학기",
        starts_on: Some(date(3, 2)),
        ends_on: Some(date(6, 30)),
    }
}

#[test]
fn name_trims_and_limits_length() {
    assert_eq!(
        TermName::parse(" 2027년 1학기 ").unwrap().as_str(),
        "2027년 1학기"
    );
    assert_eq!(TermName::parse(" "), Err(TermError::EmptyName));
    assert!(TermName::parse(&"가".repeat(30)).is_ok());
    assert_eq!(
        TermName::parse(&"가".repeat(31)),
        Err(TermError::NameTooLong)
    );
}

#[test]
fn br_t2_end_can_not_come_before_start() {
    assert_eq!(
        TermPeriod::new(date(6, 30), date(3, 2)),
        Err(TermError::EndBeforeStart)
    );
}

#[test]
fn br_t2_one_day_term_is_fine() {
    let period = TermPeriod::new(date(3, 2), date(3, 2)).unwrap();

    assert_eq!(period.starts_on(), period.ends_on());
}

#[test]
fn parse_keeps_every_field() {
    let term = NewTerm::parse(input()).unwrap();

    assert_eq!(term.name().as_str(), "2027년 1학기");
    assert_eq!(term.period().starts_on(), date(3, 2));
    assert_eq!(term.period().ends_on(), date(6, 30));
}

#[test]
fn dates_are_required() {
    let errors = NewTerm::parse(TermInput {
        starts_on: None,
        ends_on: None,
        ..input()
    })
    .unwrap_err();

    assert_eq!(errors, [TermError::MissingStart, TermError::MissingEnd]);
}

#[test]
fn br_t2_is_reported_with_other_errors() {
    let errors = NewTerm::parse(TermInput {
        name: "",
        starts_on: Some(date(6, 30)),
        ends_on: Some(date(3, 2)),
    })
    .unwrap_err();

    assert_eq!(errors, [TermError::EmptyName, TermError::EndBeforeStart]);
}

#[test]
fn br_t2_alone_is_reported() {
    let errors = NewTerm::parse(TermInput {
        starts_on: Some(date(6, 30)),
        ends_on: Some(date(3, 2)),
        ..input()
    })
    .unwrap_err();

    assert_eq!(errors, [TermError::EndBeforeStart]);
}
