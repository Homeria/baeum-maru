//! 결과 명단(BR-P1~P11): 명단 종류, 넣을 칸, 가림.
//!
//! 명단 종류마다 늘 넣는 칸과 넣을지 고르는 칸이 다르다. 게시용은 연락처와 생년월일을
//! 넣으면 늘 가린다(BR-P5). 이 규칙은 [`RosterSpec::new`]가 지킨다.

use std::fmt;

/// 결과 명단의 종류(BR-P1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RosterKind {
    /// 원본 명단. 한 줄이 신청 하나인 긴 표.
    Raw,
    /// 게시용 강좌별. 강좌마다 시트 하나.
    PostedByCourse,
    /// 게시용 회원별. 회원 한 명이 한 줄.
    PostedByMember,
}

impl RosterKind {
    /// 모든 종류. 화면 순서.
    pub const ALL: [RosterKind; 3] = [
        RosterKind::Raw,
        RosterKind::PostedByCourse,
        RosterKind::PostedByMember,
    ];

    /// DB에 쓰는 이름. 예: `posted_by_course`
    pub fn code(self) -> &'static str {
        match self {
            RosterKind::Raw => "raw",
            RosterKind::PostedByCourse => "posted_by_course",
            RosterKind::PostedByMember => "posted_by_member",
        }
    }

    /// 게시용인지. 게시용은 포기한 신청을 넣지 않는다(BR-P3).
    pub fn is_posted(self) -> bool {
        self != RosterKind::Raw
    }

    /// 이 종류에 늘 들어가는 칸(BR-P2, BR-P3, BR-P4).
    pub fn fixed_columns(self) -> &'static [RosterColumn] {
        use RosterColumn::*;
        match self {
            RosterKind::Raw => &[CourseName, Result, Name, MemberNo],
            RosterKind::PostedByCourse | RosterKind::PostedByMember => &[
                CourseName,
                Capacity,
                Result,
                WaitlistNumber,
                Name,
                MemberNo,
                Guaranteed,
            ],
        }
    }

    /// 이 종류에서 넣을지 고르는 칸(BR-P2, BR-P5).
    pub fn optional_columns(self) -> &'static [RosterColumn] {
        use RosterColumn::*;
        match self {
            RosterKind::Raw => &[
                Capacity,
                WaitlistNumber,
                PhoneNumber,
                BirthDate,
                Gender,
                Guaranteed,
            ],
            RosterKind::PostedByCourse | RosterKind::PostedByMember => {
                &[PhoneNumber, BirthDate, Gender]
            }
        }
    }

    /// 이 종류에서 가릴지 고르는 칸(BR-P6).
    pub fn maskable_columns(self) -> &'static [RosterColumn] {
        use RosterColumn::*;
        match self {
            RosterKind::Raw => &[Name, MemberNo, PhoneNumber, BirthDate],
            RosterKind::PostedByCourse | RosterKind::PostedByMember => &[Name, MemberNo],
        }
    }

    /// 이 종류에서 넣으면 늘 가리는 칸(BR-P5).
    pub fn always_masked_columns(self) -> &'static [RosterColumn] {
        use RosterColumn::*;
        match self {
            RosterKind::Raw => &[],
            RosterKind::PostedByCourse | RosterKind::PostedByMember => &[PhoneNumber, BirthDate],
        }
    }
}

/// 결과 명단에 넣을 수 있는 칸. [`RosterColumn::ALL`]의 순서가 원본 명단의 칸 순서다.
///
/// 우선 배정 사유와 포기 사유는 어느 명단에도 넣지 않으므로 없다(BR-P8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RosterColumn {
    /// 강좌 이름.
    CourseName,
    /// 정원.
    Capacity,
    /// 결과(당첨·대기·포기).
    Result,
    /// 대기 순번.
    WaitlistNumber,
    /// 회원 이름.
    Name,
    /// 회원번호.
    MemberNo,
    /// 연락처.
    PhoneNumber,
    /// 생년월일.
    BirthDate,
    /// 성별.
    Gender,
    /// 우선(자리 보장으로 당첨됐는지). 게시용은 색칠로 보인다(BR-P4).
    Guaranteed,
}

impl RosterColumn {
    /// 모든 칸. 원본 명단의 칸 순서.
    pub const ALL: [RosterColumn; 10] = [
        RosterColumn::CourseName,
        RosterColumn::Capacity,
        RosterColumn::Result,
        RosterColumn::WaitlistNumber,
        RosterColumn::Name,
        RosterColumn::MemberNo,
        RosterColumn::PhoneNumber,
        RosterColumn::BirthDate,
        RosterColumn::Gender,
        RosterColumn::Guaranteed,
    ];

    /// DB와 JSON에 쓰는 이름. 예: `phone_number`
    pub fn code(self) -> &'static str {
        match self {
            RosterColumn::CourseName => "course_name",
            RosterColumn::Capacity => "capacity",
            RosterColumn::Result => "result",
            RosterColumn::WaitlistNumber => "waitlist_number",
            RosterColumn::Name => "name",
            RosterColumn::MemberNo => "member_no",
            RosterColumn::PhoneNumber => "phone_number",
            RosterColumn::BirthDate => "birth_date",
            RosterColumn::Gender => "gender",
            RosterColumn::Guaranteed => "guaranteed",
        }
    }

    /// 명단 머리글. 예: `연락처`
    pub fn label(self) -> &'static str {
        match self {
            RosterColumn::CourseName => "강좌",
            RosterColumn::Capacity => "정원",
            RosterColumn::Result => "결과",
            RosterColumn::WaitlistNumber => "대기 순번",
            RosterColumn::Name => "이름",
            RosterColumn::MemberNo => "회원번호",
            RosterColumn::PhoneNumber => "연락처",
            RosterColumn::BirthDate => "생년월일",
            RosterColumn::Gender => "성별",
            RosterColumn::Guaranteed => "우선",
        }
    }
}

/// 가린 자리에 넣는 글자(BR-P7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaskChar {
    /// `○`. 처음 값.
    Circle,
    /// `*`.
    Asterisk,
    /// `X`.
    X,
}

impl MaskChar {
    /// 모든 가림 글자. 화면 순서.
    pub const ALL: [MaskChar; 3] = [MaskChar::Circle, MaskChar::Asterisk, MaskChar::X];

    /// DB에 쓰는 이름. 예: `circle`
    pub fn code(self) -> &'static str {
        match self {
            MaskChar::Circle => "circle",
            MaskChar::Asterisk => "asterisk",
            MaskChar::X => "x",
        }
    }

    /// 실제로 넣는 글자.
    pub fn as_char(self) -> char {
        match self {
            MaskChar::Circle => '○',
            MaskChar::Asterisk => '*',
            MaskChar::X => 'X',
        }
    }
}

/// 직원이 내보내기 창에서 고른 것. 확인 전이다.
#[derive(Debug, Clone, Copy)]
pub struct RosterChoice<'a> {
    /// 명단 종류.
    pub kind: RosterKind,
    /// 넣겠다고 고른 칸. 늘 넣는 칸은 적지 않아도 들어간다.
    pub include: &'a [RosterColumn],
    /// 가리겠다고 고른 칸. 넣지 않은 칸은 무시한다.
    pub mask: &'a [RosterColumn],
    /// 가림 글자.
    pub mask_char: MaskChar,
}

/// 명단 종류에서 고를 수 없는 칸을 골랐다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RosterError {
    /// 그 종류에서 넣을지 고를 수 없는 칸. 예: 게시용에 우선 배정 사유
    #[error("이 명단에서 넣을지 고를 수 없는 칸입니다: {}", .0.label())]
    NotIncludable(RosterColumn),
    /// 그 종류에서 가릴지 고를 수 없는 칸. 예: 원본의 성별
    #[error("이 명단에서 가릴지 고를 수 없는 칸입니다: {}", .0.label())]
    NotMaskable(RosterColumn),
}

/// 규칙(BR-P2~P7)을 지킨 명단 설정. 무엇을 넣고 무엇을 가릴지 정해져 있다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RosterSpec {
    kind: RosterKind,
    columns: Vec<RosterColumn>,
    masked: Vec<RosterColumn>,
    mask_char: MaskChar,
}

impl RosterSpec {
    /// 고른 것을 확인해 명단 설정을 만든다.
    ///
    /// 늘 넣는 칸과 늘 가리는 칸은 고르지 않아도 더한다. 칸은 [`RosterColumn::ALL`] 순서로 둔다.
    ///
    /// # Errors
    ///
    /// 그 종류에서 고를 수 없는 칸을 넣거나 가리겠다고 하면 [`RosterError`].
    pub fn new(choice: RosterChoice<'_>) -> Result<Self, RosterError> {
        let kind = choice.kind;
        if let Some(&column) = choice
            .include
            .iter()
            .find(|c| !kind.fixed_columns().contains(c) && !kind.optional_columns().contains(c))
        {
            return Err(RosterError::NotIncludable(column));
        }
        if let Some(&column) = choice.mask.iter().find(|c| {
            !kind.maskable_columns().contains(c) && !kind.always_masked_columns().contains(c)
        }) {
            return Err(RosterError::NotMaskable(column));
        }
        let columns: Vec<RosterColumn> = RosterColumn::ALL
            .into_iter()
            .filter(|c| kind.fixed_columns().contains(c) || choice.include.contains(c))
            .collect();
        let masked = columns
            .iter()
            .copied()
            .filter(|c| choice.mask.contains(c) || kind.always_masked_columns().contains(c))
            .collect();
        Ok(RosterSpec {
            kind,
            columns,
            masked,
            mask_char: choice.mask_char,
        })
    }

    /// 명단 종류.
    pub fn kind(&self) -> RosterKind {
        self.kind
    }

    /// 넣는 칸. [`RosterColumn::ALL`] 순서.
    pub fn columns(&self) -> &[RosterColumn] {
        &self.columns
    }

    /// 가리는 칸. [`RosterColumn::ALL`] 순서.
    pub fn masked(&self) -> &[RosterColumn] {
        &self.masked
    }

    /// 가림 글자.
    pub fn mask_char(&self) -> MaskChar {
        self.mask_char
    }

    /// `column`을 넣는지.
    pub fn includes(&self, column: RosterColumn) -> bool {
        self.columns.contains(&column)
    }

    /// `column`을 가리는지.
    pub fn is_masked(&self, column: RosterColumn) -> bool {
        self.masked.contains(&column)
    }
}

/// 이름을 가린다(BR-P7). 첫 글자와 끝 글자를 남기고, 두 글자면 끝 글자를 가린다.
///
/// # Examples
///
/// ```
/// use baeum_domain::{MaskChar, mask_name};
///
/// assert_eq!(mask_name("홍길동", MaskChar::Circle), "홍○동");
/// assert_eq!(mask_name("김철", MaskChar::Asterisk), "김*");
/// ```
pub fn mask_name(name: &str, mask: MaskChar) -> String {
    let chars: Vec<char> = name.chars().collect();
    match chars.as_slice() {
        [] | [_] => name.to_owned(),
        [first, _] => [*first, mask.as_char()].iter().collect(),
        [first, middle @ .., last] => std::iter::once(*first)
            .chain(middle.iter().map(|_| mask.as_char()))
            .chain(std::iter::once(*last))
            .collect(),
    }
}

/// 회원번호를 가린다(BR-P7). 숫자만 세어 앞 2개와 끝 2개를 남기고 사이를 가린다.
/// 숫자가 4개 이하면 끝 2개를 가린다. 숫자가 아닌 글자는 그대로 둔다.
///
/// # Examples
///
/// ```
/// use baeum_domain::{MaskChar, mask_member_no};
///
/// assert_eq!(mask_member_no("12-00034", MaskChar::Circle), "12-○○○34");
/// assert_eq!(mask_member_no("1234", MaskChar::X), "12XX");
/// ```
pub fn mask_member_no(member_no: &str, mask: MaskChar) -> String {
    let digits = member_no.chars().filter(char::is_ascii_digit).count();
    let hidden = |nth: usize| {
        if digits <= 4 {
            nth + 2 >= digits
        } else {
            nth >= 2 && nth + 2 < digits
        }
    };
    let mut nth = 0;
    member_no
        .chars()
        .map(|c| {
            if !c.is_ascii_digit() {
                return c;
            }
            let shown = if hidden(nth) { mask.as_char() } else { c };
            nth += 1;
            shown
        })
        .collect()
}

/// 숫자만 있는 연락처를 세 묶음으로 나눈다. 예: `01012345678` → `010`, `1234`, `5678`
///
/// 서울(02)은 앞 묶음이 2자리, 나머지는 3자리다. 끝 묶음은 4자리다.
/// 이 모양이 아니면(숫자가 아니거나 7자리보다 짧음) `None`.
fn phone_groups(digits: &str) -> Option<(&str, &str, &str)> {
    if digits.len() < 7 || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let area = if digits.starts_with("02") { 2 } else { 3 };
    let (head, rest) = digits.split_at(area);
    let (middle, last) = rest.split_at(rest.len().checked_sub(4)?);
    if middle.is_empty() {
        return None;
    }
    Some((head, middle, last))
}

/// 숫자만 있는 연락처를 하이픈으로 나눠 보여 준다. 나눌 수 없으면 그대로.
///
/// # Examples
///
/// ```
/// use baeum_domain::format_phone_number;
///
/// assert_eq!(format_phone_number("01012345678"), "010-1234-5678");
/// assert_eq!(format_phone_number("021234567"), "02-123-4567");
/// ```
pub fn format_phone_number(digits: &str) -> String {
    match phone_groups(digits) {
        Some((head, middle, last)) => format!("{head}-{middle}-{last}"),
        None => digits.to_owned(),
    }
}

/// 연락처의 가운데 묶음을 가린다(BR-P7). 나눌 수 없으면 끝 4자리만 남기고 가린다.
///
/// # Examples
///
/// ```
/// use baeum_domain::{MaskChar, mask_phone_number};
///
/// assert_eq!(mask_phone_number("01012345678", MaskChar::Circle), "010-○○○○-5678");
/// ```
pub fn mask_phone_number(digits: &str, mask: MaskChar) -> String {
    let hide = |text: &str| -> String { text.chars().map(|_| mask.as_char()).collect() };
    match phone_groups(digits) {
        Some((head, middle, last)) => format!("{head}-{}-{last}", hide(middle)),
        None => {
            let keep = digits.chars().count().saturating_sub(4);
            digits
                .chars()
                .enumerate()
                .map(|(i, c)| if i < keep { mask.as_char() } else { c })
                .collect()
        }
    }
}

/// 가린 생년월일: 연도만 남긴다(BR-P7). 예: `1950년`
pub fn masked_birth_date(year: i32) -> String {
    format!("{year}년")
}

impl fmt::Display for RosterKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            RosterKind::Raw => "원본",
            RosterKind::PostedByCourse => "게시용 강좌별",
            RosterKind::PostedByMember => "게시용 회원별",
        })
    }
}

#[cfg(test)]
mod tests {
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
        let raw = spec(RosterKind::Raw, &[PhoneNumber, Capacity], &[]).unwrap();

        assert_eq!(
            raw.columns(),
            [CourseName, Capacity, Result, Name, MemberNo, PhoneNumber]
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
}
