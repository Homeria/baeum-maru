//! 호스트 넘기기의 데이터 묶음 ID와 세대 번호(BR-H4, BR-H5).

use std::fmt;

/// 한 기관 데이터의 고유 번호(BR-H4). 무작위 16바이트이고, 넘겨도 바뀌지 않는다.
///
/// 글자로는 소문자 16진수 32자다. 개인정보가 아니라서 승인 전 버전 확인에도 알려 준다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DataBundleId([u8; 16]);

/// 묶음 ID 글자가 소문자 16진수 32자가 아니다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("데이터 묶음 ID는 소문자 16진수 32자입니다")]
pub struct DataBundleIdError;

impl DataBundleId {
    /// 바이트에서 만든다. 새 묶음 ID는 DB가 무작위로 정한다.
    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        DataBundleId(bytes)
    }

    /// 소문자 16진수 32자를 읽는다.
    ///
    /// # Errors
    ///
    /// 길이가 다르거나 소문자 16진수가 아닌 글자가 있으면 [`DataBundleIdError`].
    pub fn parse(text: &str) -> Result<Self, DataBundleIdError> {
        let (pairs, []) = text.as_bytes().as_chunks::<2>() else {
            return Err(DataBundleIdError);
        };
        if pairs.len() != 16 {
            return Err(DataBundleIdError);
        }
        let mut bytes = [0u8; 16];
        for (byte, pair) in bytes.iter_mut().zip(pairs) {
            *byte = (hex_digit(pair[0])? << 4) | hex_digit(pair[1])?;
        }
        Ok(DataBundleId(bytes))
    }
}

/// 소문자 16진수 한 글자의 값.
fn hex_digit(digit: u8) -> Result<u8, DataBundleIdError> {
    match digit {
        b'0'..=b'9' => Ok(digit - b'0'),
        b'a'..=b'f' => Ok(digit - b'a' + 10),
        _ => Err(DataBundleIdError),
    }
}

impl fmt::Display for DataBundleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// 호스트가 가진 데이터가 무엇이고 몇 번째 호스트인지(BR-H4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostIdentity {
    /// 어느 기관 데이터인지.
    pub bundle_id: DataBundleId,
    /// 넘겨받은 횟수 + 1. 처음 만든 데이터는 1이다.
    pub generation: u32,
}

/// 접속 PC가 기억한 호스트와 지금 만난 호스트를 견준 결과(BR-H5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostCheck {
    /// 처음 붙는다. 기억해 둔다.
    First,
    /// 같은 묶음, 같은 세대.
    Same,
    /// 같은 묶음, 더 새 세대. 넘겨받은 새 호스트다. 새 세대를 기억한다.
    Newer,
    /// 같은 묶음, 더 옛 세대. 넘기기 전 옛 호스트다. 붙지 않는다.
    Older,
    /// 다른 묶음. 다른 데이터의 호스트다. 붙지 않는다.
    OtherBundle,
}

impl HostCheck {
    /// 이 호스트에 붙어도 되는지.
    pub fn can_connect(self) -> bool {
        matches!(self, HostCheck::First | HostCheck::Same | HostCheck::Newer)
    }
}

/// 기억한 호스트(`None`이면 처음)와 만난 호스트를 견준다(BR-H5).
pub fn check_host(remembered: Option<HostIdentity>, seen: HostIdentity) -> HostCheck {
    let Some(remembered) = remembered else {
        return HostCheck::First;
    };
    if remembered.bundle_id != seen.bundle_id {
        return HostCheck::OtherBundle;
    }
    match seen.generation.cmp(&remembered.generation) {
        std::cmp::Ordering::Equal => HostCheck::Same,
        std::cmp::Ordering::Greater => HostCheck::Newer,
        std::cmp::Ordering::Less => HostCheck::Older,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: &str = "00112233445566778899aabbccddeeff";

    fn identity(byte: u8, generation: u32) -> HostIdentity {
        HostIdentity {
            bundle_id: DataBundleId::from_bytes([byte; 16]),
            generation,
        }
    }

    #[test]
    fn bundle_id_reads_and_writes_lowercase_hex() {
        let id = DataBundleId::parse(TEXT).unwrap();

        assert_eq!(id.to_string(), TEXT);
    }

    #[test]
    fn bundle_id_refuses_other_text() {
        for text in [
            "",
            "0011",
            &TEXT.to_uppercase(),
            &format!("{}zz", &TEXT[..30]),
        ] {
            assert_eq!(DataBundleId::parse(text), Err(DataBundleIdError), "{text}");
        }
    }

    #[test]
    fn br_h5_first_host_is_remembered() {
        assert_eq!(check_host(None, identity(1, 3)), HostCheck::First);
    }

    #[test]
    fn br_h5_same_or_newer_generation_connects() {
        assert_eq!(
            check_host(Some(identity(1, 2)), identity(1, 2)),
            HostCheck::Same
        );
        assert_eq!(
            check_host(Some(identity(1, 2)), identity(1, 3)),
            HostCheck::Newer
        );
        assert!(HostCheck::Newer.can_connect());
    }

    #[test]
    fn br_h5_older_generation_or_other_bundle_is_refused() {
        let older = check_host(Some(identity(1, 3)), identity(1, 2));
        let other = check_host(Some(identity(1, 3)), identity(2, 9));

        assert_eq!(older, HostCheck::Older);
        assert_eq!(other, HostCheck::OtherBundle);
        assert!(!older.can_connect());
        assert!(!other.can_connect());
    }
}
