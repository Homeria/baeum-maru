//! 비밀 다루기: PIN과 복구 코드의 해시, 새 복구 코드, 세션 증표.
//!
//! ADR 0014: PIN과 복구 코드는 argon2id(설정값은 crate 기본값). 해시는 일부러 느리게
//! 만들어져서 요청 처리에서는 [`tokio::task::spawn_blocking`]으로 따로 돌린다.
//! 세션 증표는 길고 무작위라 빠른 SHA-256으로 충분하다.

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use baeum_domain::RecoveryCode;
use sha2::{Digest, Sha256};

/// 해시나 난수를 만들지 못했다. 오류 문장에 비밀 글자는 들어가지 않는다.
#[derive(Debug, thiserror::Error)]
pub enum SecretError {
    #[error("해시를 만들지 못했습니다: {0}")]
    Hash(argon2::password_hash::Error),
    #[error("운영체제의 난수를 얻지 못했습니다: {0}")]
    Random(getrandom::Error),
}

/// `secret`의 argon2id 해시를 PHC 문자열(`$argon2id$v=19$...`)로 돌려준다.
///
/// 소금(salt)을 매번 새로 뽑아서 같은 글자도 해시가 매번 다르다.
pub fn hash(secret: &str) -> Result<String, SecretError> {
    let hash: PasswordHash = Argon2::default()
        .hash_password(secret.as_bytes())
        .map_err(SecretError::Hash)?;
    Ok(hash.to_string())
}

/// `secret`이 저장된 해시 `stored`와 맞는지. 저장된 해시가 망가졌으면 `false`.
pub fn verify(secret: &str, stored: &str) -> bool {
    PasswordHash::new(stored).is_ok_and(|hash| {
        Argon2::default()
            .verify_password(secret.as_bytes(), &hash)
            .is_ok()
    })
}

/// 운영체제의 난수로 새 복구 코드를 만든다.
pub fn new_recovery_code() -> Result<RecoveryCode, SecretError> {
    let mut bytes = [0; RecoveryCode::LEN];
    getrandom::fill(&mut bytes).map_err(SecretError::Random)?;
    Ok(RecoveryCode::from_random(bytes))
}

/// 로그인하면 앱에 주는 세션 증표. 운영체제의 난수 32바이트다.
///
/// 앱과는 16진수 64자로 주고받고, DB에는 [`SessionToken::hash`]만 둔다.
/// 비밀이라 `Debug`로 찍어도 보이지 않는다.
#[derive(PartialEq, Eq)]
pub struct SessionToken([u8; SessionToken::LEN]);

impl SessionToken {
    /// 바이트 수.
    const LEN: usize = 32;

    /// 새 증표를 만든다.
    pub fn new() -> Result<Self, SecretError> {
        let mut bytes = [0; Self::LEN];
        getrandom::fill(&mut bytes).map_err(SecretError::Random)?;
        Ok(SessionToken(bytes))
    }

    /// 앱이 보낸 16진수 64자를 읽는다. 모양이 틀리면 `None`.
    pub fn parse(hex: &str) -> Option<Self> {
        let hex = hex.as_bytes();
        if hex.len() != Self::LEN * 2 {
            return None;
        }
        // 길이를 위에서 확인해서 남는 조각은 없다
        let (pairs, _) = hex.as_chunks::<2>();
        let mut bytes = [0; Self::LEN];
        for (byte, &[high, low]) in bytes.iter_mut().zip(pairs) {
            *byte = (hex_value(high)? << 4) | hex_value(low)?;
        }
        Some(SessionToken(bytes))
    }

    /// DB에 두는 SHA-256 해시 32바이트.
    pub fn hash(&self) -> [u8; 32] {
        Sha256::digest(self.0).into()
    }

    /// 앱에 줄 16진수 64자(소문자).
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|byte| format!("{byte:02x}")).collect()
    }
}

impl std::fmt::Debug for SessionToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SessionToken(***)")
    }
}

/// 16진수 글자 하나의 값. 대문자도 받는다.
fn hex_value(digit: u8) -> Option<u8> {
    char::from(digit)
        .to_digit(16)
        .and_then(|value| u8::try_from(value).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_token_round_trips_through_hex() {
        let token = SessionToken::new().unwrap();
        let hex = token.to_hex();

        assert_eq!(hex.len(), 64);
        assert_eq!(SessionToken::parse(&hex), Some(token));
    }

    #[test]
    fn session_tokens_differ() {
        let first = SessionToken::new().unwrap();
        let second = SessionToken::new().unwrap();

        assert_ne!(first, second);
        assert_ne!(first.hash(), second.hash());
    }

    #[test]
    fn rejects_malformed_session_token() {
        assert_eq!(SessionToken::parse(""), None);
        assert_eq!(SessionToken::parse(&"0".repeat(63)), None);
        assert_eq!(SessionToken::parse(&"g".repeat(64)), None);
        assert_eq!(SessionToken::parse(&"가".repeat(32)), None);
        assert!(SessionToken::parse(&"aB".repeat(32)).is_some());
    }

    #[test]
    fn session_token_hash_is_sha256() {
        let token = SessionToken::parse(&"00".repeat(32)).unwrap();

        // 0x00 32바이트의 SHA-256
        let expected = "66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925";
        let hex: String = token.hash().iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(hex, expected);
        assert!(!format!("{token:?}").contains("00"));
    }

    #[test]
    fn verifies_only_the_same_secret() {
        let stored = hash("123456").unwrap();

        assert!(verify("123456", &stored));
        assert!(!verify("123457", &stored));
    }

    #[test]
    fn stores_argon2id_without_the_secret() {
        let stored = hash("123456").unwrap();

        assert!(stored.starts_with("$argon2id$"), "{stored}");
        assert!(!stored.contains("123456"));
    }

    #[test]
    fn same_secret_hashes_differently_each_time() {
        assert_ne!(hash("123456").unwrap(), hash("123456").unwrap());
    }

    #[test]
    fn broken_stored_hash_never_matches() {
        assert!(!verify("123456", "망가진 값"));
        assert!(!verify("", ""));
    }

    #[test]
    fn makes_different_recovery_codes() {
        let first = new_recovery_code().unwrap();
        let second = new_recovery_code().unwrap();

        assert_ne!(first, second);
        assert_eq!(first.as_str().len(), RecoveryCode::LEN);
    }
}
