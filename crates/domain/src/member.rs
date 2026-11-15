//! 회원 정보 항목과 형식 확인.

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

#[cfg(test)]
mod tests {
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
}
