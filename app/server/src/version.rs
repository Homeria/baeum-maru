//! 버전 번호와 호환 판단.
//!
//! ADR 0010: 앞 두 자리가 같으면 호환된다.

use std::str::FromStr;

/// 이 앱의 버전. workspace `Cargo.toml`에서 읽는다.
pub const CURRENT: &str = env!("CARGO_PKG_VERSION");

/// `앞.둘째.셋째` 형식의 버전 번호.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

impl Version {
    /// 다른 버전과 함께 쓸 수 있는지 본다. 셋째 자리는 보지 않는다.
    pub fn is_compatible_with(self, other: Version) -> bool {
        self.major == other.major && self.minor == other.minor
    }
}

/// 버전 형식이 아닌 글자를 읽으려 했다.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("버전 형식이 아닙니다: {0}")]
pub struct ParseVersionError(String);

impl FromStr for Version {
    type Err = ParseVersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('.').collect();
        let [major, minor, patch] = parts.as_slice() else {
            return Err(ParseVersionError(s.to_owned()));
        };
        let number = |part: &str| {
            part.parse::<u32>()
                .map_err(|_| ParseVersionError(s.to_owned()))
        };
        Ok(Version {
            major: number(major)?,
            minor: number(minor)?,
            patch: number(patch)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Version {
        s.parse().unwrap()
    }

    #[test]
    fn parses_three_numbers() {
        assert_eq!(
            v("1.20.3"),
            Version {
                major: 1,
                minor: 20,
                patch: 3
            }
        );
    }

    #[test]
    fn rejects_wrong_shapes() {
        for bad in ["", "0.1", "0.1.0.1", "0.a.1", "0.1.0-beta", "v0.1.0"] {
            assert!(bad.parse::<Version>().is_err(), "{bad}");
        }
    }

    #[test]
    fn current_version_parses() {
        CURRENT.parse::<Version>().unwrap();
    }

    #[test]
    fn compatible_when_only_patch_differs() {
        assert!(v("0.3.0").is_compatible_with(v("0.3.7")));
    }

    #[test]
    fn incompatible_when_minor_or_major_differs() {
        assert!(!v("0.3.0").is_compatible_with(v("0.4.0")));
        assert!(!v("1.3.0").is_compatible_with(v("0.3.0")));
    }
}
