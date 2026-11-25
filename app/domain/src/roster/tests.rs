use super::*;
use RosterColumn::*;

// `use RosterColumn::*`로 가져온 `Result` 칸과 이름이 겹쳐 std::result::Result로 쓴다
fn spec(
    kind: RosterKind,
    include: &[RosterColumn],
    mask: &[RosterColumn],
) -> std::result::Result<RosterSpec, RosterError> {
    RosterSpec::new(RosterChoice {
        kind,
        include,
        mask,
        mask_char: MaskChar::Circle,
    })
}

#[test]
fn br_p2_raw_keeps_fixed_columns_and_adds_chosen_in_order() {
    let raw = spec(RosterKind::Raw, &[Gender, PhoneNumber], &[]).unwrap();

    assert_eq!(
        raw.columns(),
        [
            CourseName,
            Capacity,
            Result,
            WaitlistNumber,
            Name,
            MemberNo,
            PhoneNumber,
            Gender,
            Guaranteed
        ]
    );
    assert!(raw.masked().is_empty());
}

#[test]
fn br_p3_p4_posted_always_has_capacity_waitlist_and_guaranteed() {
    let posted = spec(RosterKind::PostedByMember, &[], &[Name]).unwrap();

    assert_eq!(
        posted.columns(),
        [
            CourseName,
            Capacity,
            Result,
            WaitlistNumber,
            Name,
            MemberNo,
            Guaranteed
        ]
    );
    assert_eq!(posted.masked(), [Name]);
}

#[test]
fn br_p5_posted_masks_phone_and_birth_date_even_if_not_asked() {
    let posted = spec(
        RosterKind::PostedByCourse,
        &[PhoneNumber, BirthDate, Gender],
        &[],
    )
    .unwrap();

    assert_eq!(posted.masked(), [PhoneNumber, BirthDate]);
    assert!(posted.includes(Gender));
}

#[test]
fn br_p6_mask_of_column_not_included_is_ignored() {
    let raw = spec(RosterKind::Raw, &[], &[PhoneNumber, Name]).unwrap();

    assert_eq!(raw.masked(), [Name]);
}

#[test]
fn refuses_columns_the_kind_cannot_choose() {
    assert_eq!(
        spec(RosterKind::PostedByCourse, &[Guaranteed], &[]).ok(),
        spec(RosterKind::PostedByCourse, &[], &[]).ok(),
        "늘 넣는 칸을 다시 고르는 것은 괜찮다"
    );
    assert_eq!(
        spec(RosterKind::Raw, &[], &[Gender]),
        Err(RosterError::NotMaskable(Gender))
    );
    assert_eq!(
        spec(RosterKind::PostedByMember, &[], &[Capacity]),
        Err(RosterError::NotMaskable(Capacity))
    );
}

#[test]
fn br_p7_masks_names() {
    let cases = [
        ("홍길동", "홍○동"),
        ("남궁민수", "남○○수"),
        ("김철", "김○"),
        ("김", "김"),
        ("", ""),
    ];
    for (name, masked) in cases {
        assert_eq!(mask_name(name, MaskChar::Circle), masked, "{name}");
    }
}

#[test]
fn br_p7_masks_member_numbers_by_digits() {
    let cases = [
        ("12-00034", "12-***34"),
        ("2027015", "20***15"),
        ("A12345", "A12*45"),
        ("1234", "12**"),
        ("7", "*"),
    ];
    for (no, masked) in cases {
        assert_eq!(mask_member_no(no, MaskChar::Asterisk), masked, "{no}");
    }
}

#[test]
fn br_p7_masks_middle_of_phone_numbers() {
    let cases = [
        ("01012345678", "010-XXXX-5678"),
        ("0311234567", "031-XXX-4567"),
        ("021234567", "02-XXX-4567"),
        ("0212345678", "02-XXXX-5678"),
    ];
    for (digits, masked) in cases {
        assert_eq!(mask_phone_number(digits, MaskChar::X), masked, "{digits}");
    }
    assert_eq!(format_phone_number("0311234567"), "031-123-4567");
}

#[test]
fn phone_numbers_that_cannot_be_split_keep_last_four() {
    assert_eq!(mask_phone_number("12345", MaskChar::X), "X2345");
    assert_eq!(format_phone_number("12345"), "12345");
}

#[test]
fn br_p7_birth_date_keeps_year() {
    assert_eq!(masked_birth_date(1950), "1950년");
}

#[test]
fn codes_are_stable() {
    let kinds: Vec<&str> = RosterKind::ALL.iter().map(|k| k.code()).collect();
    let chars: String = MaskChar::ALL.iter().map(|m| m.as_char()).collect();

    assert_eq!(kinds, ["raw", "posted_by_course", "posted_by_member"]);
    assert_eq!(chars, "○*X");
    assert_eq!(RosterKind::PostedByMember.to_string(), "게시용 회원별");
}
