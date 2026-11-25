//! 배움마루 서버: DB, HTTP 서버, 백업, 실행 모드 파일, 호스트 넘기기 (ADR 0023).
//!
//! Tauri를 모른다. 창, 대화상자, 다시 켜기는 앱(`baeum-maru`)이 하고 여기 공개된 것만 부른다.

pub mod backup;
pub mod backup_file;
pub mod broken_db;
mod clock;
pub mod db;
pub mod handover;
mod pin_check;
mod roster_xlsx;
pub mod run_mode;
mod secret;
#[cfg(feature = "dev-seed")]
pub mod seed;
pub mod server;
pub mod startup;
pub mod take_over;
pub mod version;
