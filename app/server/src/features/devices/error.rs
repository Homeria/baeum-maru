//! 기기 승인 기능의 오류. 같은 기능의 모든 층이 쓴다 (ADR 0023).

/// 승인 요청을 받지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum RequestError {
    /// 기다리는 요청이 이미 [`MAX_OPEN_REQUESTS`]개다.
    #[error("기다리는 승인 요청이 너무 많습니다")]
    TooMany,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

/// 승인 요청을 허락하거나 거절하지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum DecideError {
    /// 그런 요청이 없거나, 이미 허락·거절했거나, 5분이 지났다.
    #[error("기다리는 승인 요청이 없습니다")]
    NotOpen,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}
