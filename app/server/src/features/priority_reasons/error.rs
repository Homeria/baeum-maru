//! 우선 배정 사유 기능의 오류. 같은 기능의 모든 층이 쓴다 (ADR 0023).

use crate::infra::db::is_unique_violation;

/// 사유를 저장하지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum SavePriorityReasonError {
    /// BR-S10: 사유 이름은 겹치지 않는다.
    #[error("같은 이름의 사유가 있습니다")]
    DuplicateName,
    #[error("사유를 저장하지 못했습니다: {0}")]
    Db(sqlx::Error),
}

impl From<sqlx::Error> for SavePriorityReasonError {
    fn from(error: sqlx::Error) -> Self {
        // priority_reasons에서 UNIQUE는 name_key 하나뿐이다
        if is_unique_violation(&error) {
            SavePriorityReasonError::DuplicateName
        } else {
            SavePriorityReasonError::Db(error)
        }
    }
}
