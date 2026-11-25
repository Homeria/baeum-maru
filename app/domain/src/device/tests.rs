use time::macros::datetime;

use super::*;

#[test]
fn approval_closes_at_five_minutes() {
    let requested = datetime!(2026-11-24 01:00 UTC);

    assert!(is_approval_open(requested, requested));
    assert!(is_approval_open(
        requested,
        datetime!(2026-11-24 01:04:59 UTC)
    ));
    assert!(!is_approval_open(
        requested,
        datetime!(2026-11-24 01:05 UTC)
    ));
}

#[test]
fn device_name_is_trimmed_and_one_line() {
    let name = DeviceName::parse("  접수\n노트북 ").unwrap();

    assert_eq!(name.as_str(), "접수 노트북");
}

#[test]
fn device_name_length() {
    assert_eq!(DeviceName::parse(" \t"), Err(DeviceError::EmptyName));
    assert!(DeviceName::parse(&"가".repeat(40)).is_ok());
    assert_eq!(
        DeviceName::parse(&"가".repeat(41)),
        Err(DeviceError::NameTooLong)
    );
}

#[test]
fn approval_code_is_six_digits() {
    assert_eq!(ApprovalCode::from_random(42).as_str(), "000042");
    assert_eq!(ApprovalCode::from_random(4_294_967_295).as_str(), "967295");
}
