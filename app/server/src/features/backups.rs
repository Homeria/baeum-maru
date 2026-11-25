//! 백업(ADR 0011): 백업 만들기, 백업 검사, 복원, 두 번째 위치, 깨진 DB 복원, 백업 요청 처리.
//!
//! 백업 만들기·검사·복원과 깨진 DB 복원은 HTTP를 모르고 앱 명령도 부른다 (ADR 0023 구현 메모).

pub mod backup;
pub mod broken_db;
mod dto;
pub(crate) mod handler;
#[cfg(test)]
mod tests;
