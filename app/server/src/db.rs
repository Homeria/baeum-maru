//! 아직 기능 폴더(`features/`)로 옮기지 않은 기능의 SQL (ADR 0023).
//!
//! DB 열기와 공통 도구는 [`crate::infra::db`]에 있다.

pub mod application_priorities;
pub mod applications;
pub mod rosters;
pub mod selection_changes;
pub mod selection_results;
#[cfg(test)]
pub(crate) mod test_support;
