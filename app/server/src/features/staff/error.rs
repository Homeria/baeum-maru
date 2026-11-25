//! 직원 기능의 오류. 같은 기능의 모든 층이 쓴다 (ADR 0023).

use crate::infra::db::is_unique_violation;

/// 직원 계정을 저장하지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum SaveStaffError {
    /// BR-K2: 직원 이름은 겹치지 않는다.
    #[error("같은 이름의 직원이 있습니다")]
    DuplicateName,
    /// BR-K9: 사용 중인 관리자가 하나도 남지 않게 된다.
    #[error("사용 중인 관리자가 한 명은 있어야 합니다")]
    LastAdmin,
    #[error("직원을 저장하지 못했습니다: {0}")]
    Db(sqlx::Error),
}

impl From<sqlx::Error> for SaveStaffError {
    fn from(error: sqlx::Error) -> Self {
        // staff에서 UNIQUE는 name_key 하나뿐이다
        if is_unique_violation(&error) {
            SaveStaffError::DuplicateName
        } else {
            SaveStaffError::Db(error)
        }
    }
}
