//! 직원 (BR-K2~K6, BR-K9, BR-K10). 단순한 기능이라 service 없이 handler와 repository만 둔다 (ADR 0023).

mod dto;
pub(crate) mod handler;
pub(crate) mod repository;
#[cfg(test)]
mod tests;
