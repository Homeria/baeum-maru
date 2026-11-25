//! 배움마루 서버: DB, HTTP 서버, 백업, 실행 모드 파일, 호스트 넘기기 (ADR 0023).
//!
//! Tauri를 모른다. 창, 대화상자, 다시 켜기는 앱(`baeum-maru`)이 하고 여기 공개된 것만 부른다.
//! DB 함수와 HTTP 요청 처리는 공개하지 않는다. 앱이 쓰는 것만 아래에서 다시 내보낸다.

pub mod backup;
pub mod broken_db;
mod db;
pub mod handover;
mod infra;
mod pin_check;
mod roster_xlsx;
pub mod run_mode;
#[cfg(feature = "dev-seed")]
pub mod seed;
mod server;
pub mod take_over;

pub use db::devices::open_request_count;
pub use infra::backup_file::name::BackupKind;
pub use infra::{startup, version};
pub use server::serve;
