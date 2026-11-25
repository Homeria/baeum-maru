//! 선발 결과 기능의 오류. 같은 기능의 모든 층이 쓴다 (ADR 0023).

/// 선발 결과를 읽지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum CourseResultError {
    #[error("강좌를 찾지 못했습니다")]
    CourseNotFound,
    /// 포기와 승격은 선발 마감 뒤에만 한다(BR-C3).
    #[error("선발을 마감하지 않은 강좌입니다")]
    NotClosed,
    #[error("선발 결과를 읽지 못했습니다: {0}")]
    Db(#[from] sqlx::Error),
}

/// 선발 결과를 바꾸지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum ChangeSelectionError {
    #[error("신청을 찾지 못했습니다")]
    ApplicationNotFound,
    /// 포기와 승격은 선발 마감 뒤에만 한다(BR-C3).
    #[error("선발을 마감하지 않은 강좌입니다")]
    NotClosed,
    /// 이 일을 할 수 없는 상태다. 예: 대기자가 아닌 신청에 연락 결과
    #[error("지금 상태에서는 할 수 없습니다")]
    WrongStatus,
    /// 빈자리가 없어 당첨으로 바꿀 수 없다(BR-W3, BR-W8).
    #[error("빈자리가 없습니다")]
    NoOpenSeat,
    #[error("선발 결과를 바꾸지 못했습니다: {0}")]
    Db(#[from] sqlx::Error),
}
