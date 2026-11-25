//! 강의실 기능의 오류. 같은 기능의 모든 층이 쓴다 (ADR 0023).

use crate::infra::db::is_unique_violation;

/// 강의실을 저장하지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum SaveRoomError {
    /// BR-R1: 같은 건물 안에서 강의실 이름은 겹치지 않는다.
    #[error("같은 건물에 같은 이름의 강의실이 있습니다")]
    DuplicateName,
    #[error("강의실을 저장하지 못했습니다: {0}")]
    Db(sqlx::Error),
}

impl From<sqlx::Error> for SaveRoomError {
    fn from(error: sqlx::Error) -> Self {
        // rooms에서 UNIQUE는 (building, name) 하나뿐이다
        if is_unique_violation(&error) {
            SaveRoomError::DuplicateName
        } else {
            SaveRoomError::Db(error)
        }
    }
}
