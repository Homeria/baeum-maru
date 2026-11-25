//! 선발 결과 (BR-C1~C5, BR-X1~X3). 포기·연락 기록은 `repository/changes.rs`. 트랜잭션은 아직 repository가 연다 (ADR 0023).

pub(crate) mod dto;
pub(crate) mod handler;
pub(crate) mod repository;
#[cfg(test)]
mod tests;
