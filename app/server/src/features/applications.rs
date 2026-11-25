//! 신청 (BR-A1~A7, BR-S3, BR-S13, BR-X1~X2). 우선 배정 사유 붙이기는 `repository/priorities.rs`. 트랜잭션은 아직 repository가 연다 (ADR 0023).

mod dto;
pub(crate) mod handler;
pub(crate) mod repository;
#[cfg(test)]
mod tests;
