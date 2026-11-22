//! 회원 정보 항목과 형식 확인.

use time::Date;

use crate::Personal;

/// 회원 정보 항목이 형식에 맞지 않는다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum MemberError {
    /// 회원번호에 허용하지 않는 글자가 있거나 길이가 맞지 않는다.
    #[error("회원번호는 숫자, 영문, 하이픈으로 1~20자까지 씁니다")]
    InvalidMemberNo,
    /// 이름이 비었다.
    #[error("이름을 입력해 주세요")]
    EmptyName,
    /// 이름이 너무 길다.
    #[error("이름은 30자까지 씁니다")]
    NameTooLong,
    /// 연락처가 전화번호 모양이 아니다.
    #[error("연락처는 0으로 시작하는 숫자 9~11자리입니다")]
    InvalidPhoneNumber,
    /// 생년월일이 오늘보다 뒤다.
    #[error("생년월일이 오늘보다 뒤입니다")]
    BirthDateInFuture,
}

/// 기관이 회원에게 붙인 번호. 글자로 다뤄서 앞자리 0을 지킨다.
///
/// 숫자, 영문, 하이픈으로 1~20자다. 앞뒤 공백은 지운다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberNo(String);

impl MemberNo {
    /// 가장 긴 길이.
    pub const MAX_LEN: usize = 20;

    /// 입력한 글자에서 회원번호를 만든다.
    ///
    /// # Errors
    ///
    /// 형식이 맞지 않으면 [`MemberError::InvalidMemberNo`].
    ///
    /// # Examples
    ///
    /// ```
    /// use baeum_domain::MemberNo;
    ///
    /// assert_eq!(MemberNo::parse(" 00123 ").unwrap().as_str(), "00123");
    /// assert!(MemberNo::parse("12 34").is_err());
    /// ```
    pub fn parse(input: &str) -> Result<Self, MemberError> {
        let trimmed = input.trim();
        let allowed = trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-');
        if allowed && (1..=Self::MAX_LEN).contains(&trimmed.len()) {
            Ok(MemberNo(trimmed.to_owned()))
        } else {
            Err(MemberError::InvalidMemberNo)
        }
    }

    /// 회원번호 글자.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 회원의 이름. 1~30자이고 앞뒤 공백은 지운다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberName(String);

impl MemberName {
    /// 가장 긴 글자 수.
    pub const MAX_CHARS: usize = 30;

    /// 입력한 글자에서 이름을 만든다.
    ///
    /// # Errors
    ///
    /// 비었으면 [`MemberError::EmptyName`], 30자를 넘으면 [`MemberError::NameTooLong`].
    pub fn parse(input: &str) -> Result<Self, MemberError> {
        let trimmed = input.trim();
        match trimmed.chars().count() {
            0 => Err(MemberError::EmptyName),
            n if n > Self::MAX_CHARS => Err(MemberError::NameTooLong),
            _ => Ok(MemberName(trimmed.to_owned())),
        }
    }

    /// 이름 글자.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 휴대폰이나 집 전화번호. 숫자만 저장한다(`01012345678`).
///
/// 0으로 시작하는 숫자 9~11자리다. 입력할 때 하이픈과 공백은 지운다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhoneNumber(String);

impl PhoneNumber {
    /// 입력한 글자에서 연락처를 만든다.
    ///
    /// # Errors
    ///
    /// 전화번호 모양이 아니면 [`MemberError::InvalidPhoneNumber`].
    ///
    /// # Examples
    ///
    /// ```
    /// use baeum_domain::PhoneNumber;
    ///
    /// let phone = PhoneNumber::parse("010-0000-0001").unwrap();
    /// assert_eq!(phone.as_str(), "01000000001");
    /// ```
    pub fn parse(input: &str) -> Result<Self, MemberError> {
        let digits: String = input.chars().filter(|c| *c != '-' && *c != ' ').collect();
        let valid = digits.starts_with('0')
            && (9..=11).contains(&digits.len())
            && digits.chars().all(|c| c.is_ascii_digit());
        if valid {
            Ok(PhoneNumber(digits))
        } else {
            Err(MemberError::InvalidPhoneNumber)
        }
    }

    /// 숫자만 있는 연락처.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 성별.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gender {
    /// 남.
    Male,
    /// 여.
    Female,
}

/// 화면에서 받은 새 회원 입력. 확인 전이라 글자 그대로다.
#[derive(Debug, Clone, Copy)]
pub struct MemberInput<'a> {
    /// 회원번호.
    pub member_no: &'a str,
    /// 이름.
    pub name: &'a str,
    /// 연락처.
    pub phone_number: &'a str,
    /// 생년월일. 선택.
    pub birth_date: Option<Date>,
    /// 성별. 선택.
    pub gender: Option<Gender>,
}

/// 확인을 통과한 새 회원. DB에 저장하기 전의 모양이다.
///
/// BR-M4: 이름, 회원번호, 연락처는 반드시 있다. 생년월일과 성별은 없어도 된다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewMember {
    member_no: Personal<MemberNo>,
    name: Personal<MemberName>,
    phone_number: Personal<PhoneNumber>,
    birth_date: Option<Personal<Date>>,
    gender: Option<Personal<Gender>>,
}

impl NewMember {
    /// 입력을 모두 확인한다. `today`는 생년월일이 미래인지 볼 때 쓴다.
    ///
    /// # Errors
    ///
    /// 틀린 항목이 있으면 [`MemberError`]를 항목 순서대로 **모두** 모아 돌려준다.
    /// 직원이 한 번에 모든 칸을 고칠 수 있게 하려는 것이다.
    pub fn parse(input: MemberInput<'_>, today: Date) -> Result<Self, Vec<MemberError>> {
        let member_no = MemberNo::parse(input.member_no);
        let name = MemberName::parse(input.name);
        let phone_number = PhoneNumber::parse(input.phone_number);
        let birth_date_ok = input.birth_date.is_none_or(|date| date <= today);

        match (member_no, name, phone_number) {
            (Ok(member_no), Ok(name), Ok(phone_number)) if birth_date_ok => Ok(NewMember {
                member_no: Personal::new(member_no),
                name: Personal::new(name),
                phone_number: Personal::new(phone_number),
                birth_date: input.birth_date.map(Personal::new),
                gender: input.gender.map(Personal::new),
            }),
            (member_no, name, phone_number) => {
                let mut errors: Vec<MemberError> =
                    [member_no.err(), name.err(), phone_number.err()]
                        .into_iter()
                        .flatten()
                        .collect();
                if !birth_date_ok {
                    errors.push(MemberError::BirthDateInFuture);
                }
                Err(errors)
            }
        }
    }

    /// 회원번호.
    pub fn member_no(&self) -> &Personal<MemberNo> {
        &self.member_no
    }

    /// 이름.
    pub fn name(&self) -> &Personal<MemberName> {
        &self.name
    }

    /// 연락처.
    pub fn phone_number(&self) -> &Personal<PhoneNumber> {
        &self.phone_number
    }

    /// 생년월일.
    pub fn birth_date(&self) -> Option<&Personal<Date>> {
        self.birth_date.as_ref()
    }

    /// 성별.
    pub fn gender(&self) -> Option<&Personal<Gender>> {
        self.gender.as_ref()
    }
}

#[cfg(test)]
mod tests {
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
}
