//! 직원 (BR-K2~K6, BR-K9, BR-K10). 저장과 마지막 관리자 확인은 `service`가 한다 (ADR 0023).

mod dto;
pub(crate) mod handler;
pub(crate) mod repository;
pub(crate) mod service;
#[cfg(test)]
mod tests;
