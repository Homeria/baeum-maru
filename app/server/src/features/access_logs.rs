//! 접속 기록 (BR-K13). 모든 기능이 기록을 남기므로 repository를 crate 안에 공개한다. 다른 기능은 부르지 않는다 (ADR 0023).

mod dto;
pub(crate) mod handler;
pub(crate) mod repository;
#[cfg(test)]
mod tests;
