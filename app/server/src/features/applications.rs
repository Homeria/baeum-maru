//! 신청 (BR-A1~A7, BR-S3, BR-S13, BR-X1~X2). 우선 배정 사유 붙이기는 `service/priorities.rs`와 `repository/priorities.rs`. 트랜잭션은 `service`가 연다 (ADR 0023).

mod dto;
pub(crate) mod error;
pub(crate) mod handler;
pub(crate) mod repository;
pub(crate) mod service;
#[cfg(test)]
mod tests;
