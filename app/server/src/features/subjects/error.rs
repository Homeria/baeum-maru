//! 과목 기능의 오류. 같은 기능의 모든 층이 쓴다 (ADR 0023).

use crate::infra::db::is_unique_violation;

/// 과목을 저장하지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum SaveSubjectError {
    /// BR-J1: 과목 이름은 기관 전체에서 겹치지 않는다.
    #[error("이미 있는 과목입니다")]
    DuplicateName,
    #[error("과목을 저장하지 못했습니다: {0}")]
    Db(sqlx::Error),
}

impl From<sqlx::Error> for SaveSubjectError {
    fn from(error: sqlx::Error) -> Self {
        // subjects에서 UNIQUE는 name_key 하나뿐이다
        if is_unique_violation(&error) {
            SaveSubjectError::DuplicateName
        } else {
            SaveSubjectError::Db(error)
        }
    }
}
