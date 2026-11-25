//! 추첨 (BR-L1~L6, BR-S3). 트랜잭션은 `service`가 연다 (ADR 0023).

mod dto;
pub(crate) mod error;
pub(crate) mod handler;
pub(crate) mod repository;
pub(crate) mod service;
#[cfg(test)]
mod tests;
