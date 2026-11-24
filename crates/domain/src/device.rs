//! 기기 승인 (BR-K14~K16, ADR 0003).

use time::{Duration, OffsetDateTime};

/// BR-K15: 승인 요청을 허락할 수 있는 시간.
pub const APPROVAL_WINDOW: Duration = Duration::minutes(5);

/// BR-K15: `requested_at`에 온 승인 요청이 `now`에도 허락할 수 있는지. 5분이 되면 끝난다.
pub fn is_approval_open(requested_at: OffsetDateTime, now: OffsetDateTime) -> bool {
    now - requested_at < APPROVAL_WINDOW
}

/// 기기 이름이 형식에 맞지 않는다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum DeviceError {
    /// 이름이 비었다.
    #[error("기기 이름이 비었습니다")]
    EmptyName,
    /// 이름이 너무 길다.
    #[error("기기 이름은 40자까지 씁니다")]
    NameTooLong,
}

/// 승인을 요청한 기기의 이름(예: 컴퓨터 이름). 1~40자이고 앞뒤 공백은 지운다.
///
/// 줄바꿈 같은 제어 문자는 빈칸으로 바꾼다. 호스트 화면에 한 줄로 보여 주기 때문이다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceName(String);

impl DeviceName {
    /// 가장 긴 글자 수.
    pub const MAX_CHARS: usize = 40;

    /// 받은 글자에서 이름을 만든다.
    ///
    /// # Errors
    ///
    /// 비었으면 [`DeviceError::EmptyName`], 40자를 넘으면 [`DeviceError::NameTooLong`].
    pub fn parse(input: &str) -> Result<Self, DeviceError> {
        let cleaned: String = input
            .chars()
            .map(|c| if c.is_control() { ' ' } else { c })
            .collect();
        let trimmed = cleaned.trim();
        match trimmed.chars().count() {
            0 => Err(DeviceError::EmptyName),
            n if n > Self::MAX_CHARS => Err(DeviceError::NameTooLong),
            _ => Ok(DeviceName(trimmed.to_owned())),
        }
    }

    /// 이름 글자.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 승인 번호(BR-K15). 숫자 6자리라 두 화면에서 눈으로 견주기 쉽다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalCode(String);

impl ApprovalCode {
    /// 자리 수.
    pub const DIGITS: usize = 6;

    /// 난수에서 만든다. 앞자리가 0이어도 6자리로 채운다.
    pub fn from_random(random: u32) -> Self {
        ApprovalCode(format!("{:06}", random % 1_000_000))
    }

    /// 번호 글자.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
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
}
