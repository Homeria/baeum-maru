//! 백업(ADR 0011): 백업 만들기, 백업 검사, 복원, 두 번째 위치, 깨진 DB 복원, 백업 요청 처리.
//!
//! 본체(`service`)는 HTTP를 모르고 앱 명령도 부른다. 파일과 데이터 폴더를 다루는 일이 많아
//! repository로 나누지 않았다 (ADR 0023 구현 메모).

mod dto;
pub(crate) mod error;
pub(crate) mod handler;
pub mod service;
#[cfg(test)]
mod tests;
