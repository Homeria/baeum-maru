//! 추첨 기능의 오류. 같은 기능의 모든 층이 쓴다 (ADR 0023).

/// 추첨하지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum RunLotteryError {
    #[error("강좌를 찾지 못했습니다")]
    CourseNotFound,
    /// BR-L5: 마감 뒤에는 다시 추첨할 수 없다.
    #[error("선발을 마감한 강좌입니다")]
    Closed,
    /// 접수됨 신청이 없다. 빈 추첨으로 신청만 막히는 일(BR-L3)을 피한다.
    #[error("추첨할 신청이 없습니다")]
    NoApplicants,
    /// BR-S5: 자리 보장 신청이 정원보다 많다. 이전 추첨도 그대로 둔다.
    #[error("자리 보장 신청 {guaranteed}건이 정원 {capacity}명보다 많습니다")]
    OverCapacity { guaranteed: usize, capacity: u16 },
    #[error("추첨하지 못했습니다: {0}")]
    Db(#[from] sqlx::Error),
}

/// 선발을 마감하지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum CloseSelectionError {
    #[error("강좌를 찾지 못했습니다")]
    CourseNotFound,
    /// 추첨 전에는 마감할 결과가 없다.
    #[error("추첨하지 않은 강좌입니다")]
    NotDrawn,
    #[error("이미 선발을 마감한 강좌입니다")]
    AlreadyClosed,
    #[error("선발을 마감하지 못했습니다: {0}")]
    Db(#[from] sqlx::Error),
}
