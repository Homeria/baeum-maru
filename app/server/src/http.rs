//! 요청 공통 처리(ADR 0023): 오류 응답, 인증, 쪽 나누기, 변경 알림(SSE), 버전 확인.
//!
//! 기능(`features`)을 모른다. 기능의 요청 처리 함수가 이것을 쓴다.

pub mod changes;
pub mod error;
pub mod paging;
pub mod version;
