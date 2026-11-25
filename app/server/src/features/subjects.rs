//! 과목 (BR-J1, BR-J2). 단순한 기능이라 service 없이 handler와 repository만 둔다 (ADR 0023).

mod dto;
pub(crate) mod error;
pub(crate) mod handler;
pub(crate) mod repository;
#[cfg(test)]
mod tests;
