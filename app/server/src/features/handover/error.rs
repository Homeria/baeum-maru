//! 넘기기 기능의 오류. 같은 기능의 모든 층이 쓴다 (ADR 0023).

use crate::features::backups::error::InspectError;
use crate::infra::db;

/// 넘겨받지 못했다. 이 PC는 그대로다.
#[derive(Debug, thiserror::Error)]
pub enum TakeOverError {
    #[error(transparent)]
    Inspect(#[from] InspectError),
    #[error("이 PC는 이미 호스트이거나 호스트를 넘기는 중입니다")]
    NotAllowedHere,
    #[error("파일에서 그 관리자를 찾지 못했습니다")]
    AdminNotFound,
    #[error("PIN이 맞지 않습니다")]
    WrongPin,
    #[error("이 PC에 남은 예전 데이터를 보관하지 못해 넘겨받지 않았습니다")]
    KeepLeftover(#[source] std::io::Error),
    #[error("파일을 이 PC로 옮기지 못했습니다")]
    Install(#[source] std::io::Error),
    #[error("넘겨받은 DB를 열지 못했습니다")]
    Open(#[source] db::DbError),
    #[error("넘겨받은 DB에 세대 번호를 쓰지 못했습니다")]
    Record(#[source] sqlx::Error),
}
