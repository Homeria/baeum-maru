//! 선발 결과 (BR-C1~C5, BR-X1~X3). 포기·연락 기록은 `service/changes.rs`와 `repository/changes.rs`. 트랜잭션은 `service`가 연다 (ADR 0023).

pub(crate) mod dto;
pub(crate) mod error;
pub(crate) mod handler;
pub(crate) mod repository;
pub(crate) mod service;
#[cfg(test)]
mod tests;
