//! 호스트 넘기기(BR-H1~H4, ADR 0002)와 실행 모드(ADR 0021).
//!
//! 넘기기·넘겨받기·실행 모드 파일은 HTTP를 모르고 앱 명령이 부른다 (ADR 0023 구현 메모).

mod dto;
pub mod hand_over;
pub(crate) mod handler;
pub mod run_mode;
pub mod take_over;
#[cfg(test)]
mod tests;
