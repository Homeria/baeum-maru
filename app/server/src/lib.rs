//! 배움마루 서버: DB, HTTP 서버, 백업, 실행 모드 파일, 호스트 넘기기 (ADR 0023).
//!
//! Tauri를 모른다. 창, 대화상자, 다시 켜기는 앱(`baeum-maru`)이 하고 여기 공개된 것만 부른다.
//! DB 함수와 HTTP 요청 처리는 공개하지 않는다. 앱이 쓰는 것만 아래에서 다시 내보낸다.

mod db;
mod features;
mod http;
mod infra;
mod router;
#[cfg(feature = "dev-seed")]
pub mod seed;
mod server;

pub use features::backups::service::{self as backup, broken_db};
pub use features::devices::repository::open_request_count;
pub use features::handover::service::{hand_over as handover, run_mode, take_over};
pub use infra::backup_file::name::BackupKind;
pub use infra::{startup, version};
pub use router::serve;
