//! 신청 기능의 오류. 같은 기능의 모든 층이 쓴다 (ADR 0023).

use baeum_domain::RuleViolation;

/// 신청을 저장하지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum SubmitError {
    #[error("회원을 찾지 못했습니다")]
    MemberNotFound,
    /// 고른 강좌가 그 학기에 없다. 그 사이 지워졌거나 다른 학기의 강좌다.
    #[error("이 학기에 없는 강좌를 골랐습니다")]
    CourseNotFound(Vec<i64>),
    /// BR-L3: 추첨한 강좌는 선발 마감 전까지 새 신청을 받지 않는다. 고른 강좌 가운데 그런 강좌들.
    #[error("추첨한 강좌는 선발 마감 전까지 새 신청을 받지 않습니다")]
    CourseDrawn(Vec<i64>),
    /// BR-X1: 학기 종료일이 지나 추가 신청을 받지 않는다. 고른 강좌 가운데 선발 마감한 강좌들.
    #[error("학기가 끝나 추가 신청을 받지 않습니다")]
    TermEnded(Vec<i64>),
    /// 우선 배정 사유를 고르지 않은 강좌에 붙였다.
    #[error("고르지 않은 강좌에 우선 배정을 붙였습니다")]
    PriorityForUnchosenCourse(Vec<i64>),
    /// BR-S13: 추가 신청(선발 마감한 강좌)에는 우선 배정 사유를 붙이지 않는다.
    #[error("추가 신청에는 우선 배정을 붙일 수 없습니다")]
    PriorityForAdditional(Vec<i64>),
    /// 목록에 없는 우선 배정 사유. 그 사이 지워졌을 수 있다.
    #[error("목록에 없는 우선 배정 사유를 골랐습니다")]
    PriorityReasonNotFound(Vec<i64>),
    /// BR-A5: 규칙을 어겼다. 사유가 있어도 예외를 줄 수 없는 규칙(BR-A7)이면 여기로 온다.
    #[error("신청 규칙을 어겼습니다")]
    RulesBroken(Vec<RuleViolation>),
    #[error("신청을 저장하지 못했습니다: {0}")]
    Db(#[from] sqlx::Error),
}

/// 신청을 지우지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum DeleteApplicationError {
    #[error("신청을 찾지 못했습니다")]
    NotFound,
    /// BR-D1: 접수됨(선발 전) 신청만 지운다.
    #[error("선발 뒤의 신청은 지울 수 없습니다")]
    NotSubmitted,
    #[error("신청을 지우지 못했습니다: {0}")]
    Db(#[from] sqlx::Error),
}

/// 사유를 바꾸지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum SetPrioritiesError {
    #[error("신청을 찾지 못했습니다")]
    NotFound,
    /// 유효한 신청(접수됨, 당첨, 대기)이 아니다. 확정은 마감 뒤라 여기 오지 않는다.
    #[error("우선 배정을 바꿀 수 없는 신청입니다")]
    NotChangeable,
    /// BR-S13: 선발 마감 뒤에는 바꿀 수 없다.
    #[error("선발을 마감한 강좌입니다")]
    Closed,
    /// 목록에 없는 사유. 그 사이 지워졌을 수 있다.
    #[error("목록에 없는 사유를 골랐습니다")]
    ReasonNotFound(Vec<i64>),
    #[error("우선 배정을 바꾸지 못했습니다: {0}")]
    Db(#[from] sqlx::Error),
}
