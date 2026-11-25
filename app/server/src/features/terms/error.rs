//! 학기 기능의 오류. 같은 기능의 모든 층이 쓴다 (ADR 0023).

use crate::infra::db::is_unique_violation;

/// 학기를 저장하지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum SaveTermError {
    /// BR-T1: 학기 이름은 겹치지 않는다.
    #[error("같은 이름의 학기가 있습니다")]
    DuplicateName,
    #[error("학기를 저장하지 못했습니다: {0}")]
    Db(sqlx::Error),
}

impl From<sqlx::Error> for SaveTermError {
    fn from(error: sqlx::Error) -> Self {
        // terms에서 UNIQUE는 name 하나뿐이다
        if is_unique_violation(&error) {
            SaveTermError::DuplicateName
        } else {
            SaveTermError::Db(error)
        }
    }
}
