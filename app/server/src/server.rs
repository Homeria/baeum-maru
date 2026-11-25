//! 아직 기능 폴더(`features/`)로 옮기지 않은 기능의 요청 처리 (ADR 0023).
//!
//! 요청 주소를 묶고 서버를 띄우는 일은 [`crate::router`]가 한다.

pub(crate) mod access_logs;
pub(crate) mod applications;
pub(crate) mod backups;
pub(crate) mod courses;
pub(crate) mod devices;
pub(crate) mod handover;
pub(crate) mod login;
pub(crate) mod lotteries;
pub(crate) mod members;
pub(crate) mod priority_reasons;
pub(crate) mod recovery;
pub(crate) mod rosters;
pub(crate) mod run_mode;
pub(crate) mod selection_results;
pub(crate) mod setup;
pub(crate) mod staff;
pub(crate) mod subjects;
pub(crate) mod terms;
#[cfg(test)]
pub(crate) mod test_support;
