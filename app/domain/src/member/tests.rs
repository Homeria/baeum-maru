use time::macros::date;

use super::*;

#[test]
fn member_no_keeps_leading_zeros() {
    assert_eq!(MemberNo::parse("00123").unwrap().as_str(), "00123");
}

#[test]
fn member_no_accepts_letters_and_hyphen() {
    for ok in ["2024-0153", "A-17", "a-17", "1", "12345678901234567890"] {
        assert!(MemberNo::parse(ok).is_ok(), "{ok}");
    }
}

#[test]
fn member_no_trims_spaces() {
    assert_eq!(MemberNo::parse("  A-17 ").unwrap().as_str(), "A-17");
}

#[test]
fn member_no_rejects_wrong_shapes() {
    for bad in ["", "   ", "12 34", "회원1", "A_17", "123456789012345678901"] {
        assert_eq!(
            MemberNo::parse(bad),
            Err(MemberError::InvalidMemberNo),
            "{bad}"
        );
    }
}

#[test]
fn member_no_case_is_kept() {
    assert_ne!(MemberNo::parse("a-17"), MemberNo::parse("A-17"));
}

#[test]
fn name_trims_spaces() {
    assert_eq!(MemberName::parse(" 회원1 ").unwrap().as_str(), "회원1");
}

#[test]
fn name_rejects_empty() {
    assert_eq!(MemberName::parse(""), Err(MemberError::EmptyName));
    assert_eq!(MemberName::parse("   "), Err(MemberError::EmptyName));
}

#[test]
fn name_counts_characters_not_bytes() {
    let thirty = "가".repeat(30);
    let thirty_one = "가".repeat(31);

    assert!(MemberName::parse(&thirty).is_ok());
    assert_eq!(
        MemberName::parse(&thirty_one),
        Err(MemberError::NameTooLong)
    );
}

#[test]
fn phone_number_keeps_only_digits() {
    for input in ["010-0000-0001", "01000000001", "010 0000 0001"] {
        assert_eq!(
            PhoneNumber::parse(input).unwrap().as_str(),
            "01000000001",
            "{input}"
        );
    }
}

#[test]
fn phone_number_accepts_home_phones() {
    for ok in ["02-000-0001", "032-000-0001", "0320000001"] {
        assert!(PhoneNumber::parse(ok).is_ok(), "{ok}");
    }
}

#[test]
fn phone_number_rejects_wrong_shapes() {
    for bad in [
        "",
        "1000000001",
        "010-0000",
        "010-0000-00001",
        "010-0000-000a",
        "(010)0000-0001",
        "+82-10-0000-0001",
    ] {
        assert_eq!(
            PhoneNumber::parse(bad),
            Err(MemberError::InvalidPhoneNumber),
            "{bad}"
        );
    }
}

fn today() -> Date {
    date!(2026 - 11 - 15)
}

fn input() -> MemberInput<'static> {
    MemberInput {
        member_no: "00001",
        name: "회원1",
        phone_number: "010-0000-0001",
        birth_date: None,
        gender: None,
    }
}

#[test]
fn br_m4_allows_only_required_fields() {
    let member = NewMember::parse(input(), today()).unwrap();

    assert_eq!(member.member_no().expose().as_str(), "00001");
    assert_eq!(member.phone_number().expose().as_str(), "01000000001");
    assert!(member.birth_date().is_none());
    assert!(member.gender().is_none());
}

#[test]
fn br_m4_allows_optional_fields() {
    let birth_date = date!(1950 - 03 - 01);
    let member = NewMember::parse(
        MemberInput {
            birth_date: Some(birth_date),
            gender: Some(Gender::Female),
            ..input()
        },
        today(),
    )
    .unwrap();

    assert_eq!(member.birth_date().map(|d| *d.expose()), Some(birth_date));
    assert_eq!(member.gender().map(|g| *g.expose()), Some(Gender::Female));
}

#[test]
fn br_m4_rejects_missing_required_fields() {
    let result = NewMember::parse(
        MemberInput {
            member_no: "",
            name: " ",
            phone_number: "",
            ..input()
        },
        today(),
    );

    assert_eq!(
        result,
        Err(vec![
            MemberError::InvalidMemberNo,
            MemberError::EmptyName,
            MemberError::InvalidPhoneNumber,
        ])
    );
}

#[test]
fn collects_every_error_in_field_order() {
    let result = NewMember::parse(
        MemberInput {
            phone_number: "123",
            birth_date: Some(date!(2026 - 11 - 16)),
            ..input()
        },
        today(),
    );

    assert_eq!(
        result,
        Err(vec![
            MemberError::InvalidPhoneNumber,
            MemberError::BirthDateInFuture,
        ])
    );
}

#[test]
fn birth_date_today_is_allowed() {
    let result = NewMember::parse(
        MemberInput {
            birth_date: Some(today()),
            ..input()
        },
        today(),
    );

    assert!(result.is_ok());
}

#[test]
fn debug_output_hides_member_details() {
    let member = NewMember::parse(input(), today()).unwrap();

    let printed = format!("{member:?}");

    assert!(!printed.contains("회원1"));
    assert!(!printed.contains("00001"));
    assert!(!printed.contains("01000000001"));
}
