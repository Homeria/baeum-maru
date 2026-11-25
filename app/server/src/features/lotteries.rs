//! 추첨 (BR-L1~L6, BR-S3). 트랜잭션은 아직 repository가 연다. service는 따로 만든다 (ADR 0023).

mod dto;
pub(crate) mod handler;
pub(crate) mod repository;
#[cfg(test)]
mod tests;
