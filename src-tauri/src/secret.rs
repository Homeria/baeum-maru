//! PIN과 복구 코드를 DB에 두는 모양(해시)과 맞춰 보기, 새 복구 코드 만들기.
//!
//! ADR 0014: argon2id, 설정값은 crate 기본값. 해시는 일부러 느리게 만들어져서
//! 요청 처리에서는 [`tokio::task::spawn_blocking`]으로 따로 돌린다.

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use baeum_domain::RecoveryCode;

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
// 로그인을 만들면 쓰이므로 그때 이 줄을 지운다 (쓰이면 expect가 경고를 낸다)
#[cfg_attr(not(test), expect(dead_code, reason = "로그인에서 처음 쓴다"))]
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

#[cfg(test)]
mod tests {
    use super::*;

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
