use time::macros::{date, datetime};

use super::*;

fn input(name: &str, role: Role, valid_until: Option<Date>) -> StaffInput<'_> {
    StaffInput {
        name,
        role,
        valid_until,
    }
}

#[test]
fn br_k2_name_key_ignores_spaces() {
    assert_eq!(StaffName::parse(" 직원 1 ").unwrap().key(), "직원1");
}

#[test]
fn br_k3_role_table() {
    use Action::*;
    let all = [
        View,
        EditMembers,
        DeleteMembers,
        EditCourses,
        EditReferenceData,
        TakeApplications,
        OverrideRules,
        AssignPriority,
        RunLottery,
        RecordWithdrawals,
        CorrectWithdrawals,
        ExportRawRoster,
        ManagePriorityReasons,
        ManageStaff,
        ViewAccessLog,
        ManageBackups,
    ];
    let allowed = |role: Role| -> Vec<Action> {
        all.into_iter().filter(|action| role.can(*action)).collect()
    };

    assert_eq!(allowed(Role::Admin), all);
    assert_eq!(
        allowed(Role::Staff),
        [
            View,
            EditMembers,
            DeleteMembers,
            EditCourses,
            EditReferenceData,
            TakeApplications,
            OverrideRules,
            AssignPriority,
            RunLottery,
            RecordWithdrawals,
            ExportRawRoster
        ]
    );
    assert_eq!(
        allowed(Role::TempStaff),
        [View, EditMembers, TakeApplications]
    );
    assert_eq!(allowed(Role::ReadOnly), [View]);
}

#[test]
fn role_code_round_trips() {
    for role in Role::ALL {
        assert_eq!(Role::from_code(role.code()), Some(role));
    }
    assert_eq!(Role::from_code("boss"), None);
}

#[test]
fn br_k4_pin_is_four_to_eight_digits() {
    for ok in ["1234", "12345678", " 0000 "] {
        assert!(Pin::parse(ok, Role::Staff).is_ok(), "{ok}");
    }
    for bad in ["123", "123456789", "12a4", "", "１２３４"] {
        assert_eq!(
            Pin::parse(bad, Role::Staff),
            Err(StaffError::InvalidPin),
            "{bad}"
        );
    }
}

#[test]
fn br_k4_admin_pin_is_six_digits_or_more() {
    assert_eq!(
        Pin::parse("12345", Role::Admin),
        Err(StaffError::AdminPinTooShort)
    );
    assert!(Pin::parse("123456", Role::Admin).is_ok());
}

#[test]
fn pin_debug_hides_digits() {
    let pin = Pin::parse("1234", Role::Staff).unwrap();
    assert_eq!(format!("{pin:?}"), "Pin(***)");
}

#[test]
fn br_k5_temp_staff_needs_valid_until() {
    assert_eq!(
        StaffProfile::parse(input("직원1", Role::TempStaff, None)),
        Err(vec![StaffError::MissingValidUntil])
    );
    assert!(
        StaffProfile::parse(input("직원1", Role::TempStaff, Some(date!(2027 - 03 - 31)))).is_ok()
    );
    assert!(StaffProfile::parse(input("직원1", Role::Staff, None)).is_ok());
}

#[test]
fn new_staff_collects_every_error_in_screen_order() {
    let errors = NewStaff::parse(input("", Role::TempStaff, None), "12").unwrap_err();

    assert_eq!(
        errors,
        [
            StaffError::EmptyName,
            StaffError::InvalidPin,
            StaffError::MissingValidUntil
        ]
    );
}

#[test]
fn new_staff_pin_error_comes_first_without_name_error() {
    let errors = NewStaff::parse(input("직원1", Role::TempStaff, None), "12").unwrap_err();

    assert_eq!(
        errors,
        [StaffError::InvalidPin, StaffError::MissingValidUntil]
    );
}

#[test]
fn br_k5_valid_through_last_day() {
    let last_day = Some(date!(2027 - 03 - 31));

    assert!(is_valid_on(last_day, date!(2027 - 03 - 31)));
    assert!(!is_valid_on(last_day, date!(2027 - 04 - 01)));
    assert!(is_valid_on(None, date!(2099 - 01 - 01)));
}

#[test]
fn br_k7_fifth_failure_locks_for_five_minutes() {
    let now = datetime!(2027-03-02 09:00:00 UTC);

    assert_eq!(
        record_failed_pin(3, now),
        FailedPin {
            failed_count: 4,
            locked_until: None
        }
    );
    assert_eq!(
        record_failed_pin(4, now),
        FailedPin {
            failed_count: 0,
            locked_until: Some(now + Duration::minutes(5))
        }
    );
}

fn at(hour: u8, minute: u8, second: u8) -> OffsetDateTime {
    date!(2026 - 11 - 15)
        .with_hms(hour, minute, second)
        .unwrap()
        .assume_utc()
}

#[test]
fn account_unlocks_at_locked_until() {
    let until = Some(at(1, 5, 0));

    assert!(is_account_locked(until, at(1, 4, 59)));
    assert!(!is_account_locked(until, at(1, 5, 0)));
    assert!(!is_account_locked(None, at(1, 0, 0)));
}

#[test]
fn screen_locks_ten_minutes_after_last_request() {
    let last_seen = at(1, 0, 0);

    assert!(!is_screen_locked(last_seen, at(1, 9, 59)));
    assert!(is_screen_locked(last_seen, at(1, 10, 0)));
}
