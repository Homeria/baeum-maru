//! 복구 코드의 모양 (BR-K11, BR-K12). 난수는 앱이 만들어 넘긴다.

/// 복구 코드에 쓰는 글자 32개. 헷갈리는 0·1·O·I는 뺐다.
const ALPHABET: &[u8; 32] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZ";

/// 복구 코드. 대문자·숫자 16자이고 4자씩 끊어 보여 준다. 예: `K7QF-M2XA-9TRE-HW4C`
///
/// 비밀이라 `Debug`로 찍어도 글자가 보이지 않는다. DB에는 해시만 둔다 (ADR 0014).
///
/// # Examples
///
/// ```
/// use baeum_domain::RecoveryCode;
///
/// let code = RecoveryCode::from_random([0; 16]);
/// assert_eq!(code.to_string(), "2222-2222-2222-2222");
/// assert_eq!(RecoveryCode::parse("2222 2222 2222 2222"), Some(code));
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct RecoveryCode(String);

impl RecoveryCode {
    /// 글자 수 (끊는 `-`는 빼고).
    pub const LEN: usize = 16;

    /// 난수 16바이트로 코드를 만든다. 바이트마다 글자 하나를 고른다.
    ///
    /// 글자가 32개라 바이트의 아래 5비트만 쓰면 모든 글자가 고르게 나온다.
    pub fn from_random(bytes: [u8; Self::LEN]) -> Self {
        let code = bytes
            .iter()
            .map(|byte| char::from(ALPHABET[usize::from(byte & 0b1_1111)]))
            .collect();
        RecoveryCode(code)
    }

    /// 직원이 입력한 글자에서 코드를 읽는다. `-`와 공백은 무시하고 소문자도 받는다.
    /// 모양이 틀리면 `None`.
    pub fn parse(input: &str) -> Option<Self> {
        let code: String = input
            .chars()
            .filter(|c| *c != '-' && !c.is_whitespace())
            .map(|c| c.to_ascii_uppercase())
            .collect();
        let valid = code.len() == Self::LEN && code.bytes().all(|b| ALPHABET.contains(&b));
        valid.then_some(RecoveryCode(code))
    }

    /// 끊지 않은 16자. 해시를 만들 때만 쓴다.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 4자씩 `-`로 끊어 쓴다.
impl std::fmt::Display for RecoveryCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, chunk) in self.0.as_bytes().chunks(4).enumerate() {
            if index > 0 {
                f.write_str("-")?;
            }
            // ALPHABET의 글자만 담겨 있어 늘 ASCII다
            f.write_str(std::str::from_utf8(chunk).map_err(|_| std::fmt::Error)?)?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for RecoveryCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RecoveryCode(***)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_every_letter_and_no_confusing_ones() {
        let bytes: [u8; 16] = std::array::from_fn(|i| u8::try_from(i * 2).unwrap());
        let code = RecoveryCode::from_random(bytes).to_string();

        assert_eq!(code, "2468-ACEG-JLNQ-SUWY");
        for confusing in ['0', '1', 'O', 'I'] {
            assert!(!ALPHABET.contains(&(confusing as u8)), "{confusing}");
        }
    }

    #[test]
    fn high_bits_are_ignored() {
        assert_eq!(
            RecoveryCode::from_random([0b1110_0000; 16]),
            RecoveryCode::from_random([0; 16])
        );
    }

    #[test]
    fn parse_accepts_lowercase_and_spaces() {
        let code = RecoveryCode::from_random([7; 16]);
        let typed = code.to_string().to_lowercase().replace('-', " ");

        assert_eq!(RecoveryCode::parse(&typed), Some(code));
    }

    #[test]
    fn parse_rejects_wrong_shape() {
        for bad in [
            "",
            "2222-2222-2222",
            "2222-2222-2222-2220",
            "2222-2222-2222-22222",
        ] {
            assert_eq!(RecoveryCode::parse(bad), None, "{bad}");
        }
    }

    #[test]
    fn debug_hides_code() {
        let code = RecoveryCode::from_random([0; 16]);
        assert_eq!(format!("{code:?}"), "RecoveryCode(***)");
    }
}
