//! 회원 기능의 오류. 같은 기능의 모든 층이 쓴다 (ADR 0023).

use crate::infra::db::is_unique_violation;

/// 회원을 저장하거나 고치지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum SaveMemberError {
    /// BR-M2: 회원번호는 겹치지 않는다.
    #[error("이미 있는 회원번호입니다")]
    DuplicateMemberNo,
    #[error("회원을 저장하지 못했습니다: {0}")]
    Db(sqlx::Error),
}

impl From<sqlx::Error> for SaveMemberError {
    fn from(error: sqlx::Error) -> Self {
        // members에서 UNIQUE는 member_no 하나뿐이다
        if is_unique_violation(&error) {
            SaveMemberError::DuplicateMemberNo
        } else {
            SaveMemberError::Db(error)
        }
    }
}
