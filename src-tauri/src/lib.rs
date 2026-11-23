//! 배움마루 앱.
//!
//! 창(Tauri)을 띄우고, 같은 프로세스에서 HTTP 서버를 돌린다.

mod backup;
mod clock;
mod data_dir;
mod db;
mod pick;
mod restart;
mod roster_xlsx;
mod save_file;
mod secret;
#[cfg(feature = "dev-seed")]
pub mod seed;
mod server;
mod startup;
mod version;

use tauri::Manager;
use tracing_subscriber::EnvFilter;

use time::OffsetDateTime;

use crate::backup::Backups;
use crate::backup::name::BackupKind;
use crate::data_dir::DataDir;

/// 앱을 시작한다. 창을 닫으면 돌아온다.
///
/// # Panics
///
/// 창을 띄우지 못하면 멈춘다.
pub fn run() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    tauri::Builder::default()
        // ADR 0018: 저장 창과 폴더 고르기 창은 앱(Rust)에서만 띄운다. 화면에 권한을 열지 않는다
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            save_file::save_file,
            pick::pick_folder,
            pick::pick_backup_file,
            restart::restart_app
        ])
        .setup(|app| {
            let host: version::Version = version::CURRENT.parse()?;
            let exe = std::env::current_exe()?;
            let exe_dir = exe
                .parent()
                .ok_or("실행 파일이 있는 폴더를 찾지 못했습니다")?;
            let data_dir = DataDir::resolve(exe_dir, app.path().app_data_dir()?);
            // 복원을 준비해 두고 다시 켠 것이면 DB를 열기 전에 바꾼다 (ADR 0011)
            let restored = tauri::async_runtime::block_on(backup::restore::apply_pending(
                data_dir.path(),
                OffsetDateTime::now_utc(),
            ));
            // DB를 연 뒤에 서버와 백업을 시작한다. 쓸 수 없으면 그 까닭을 맡겨 두고 화면이 묻는다
            match tauri::async_runtime::block_on(startup::open_checked(data_dir.path())) {
                Ok(db) => {
                    tracing::info!(portable = data_dir.is_portable(), "DB 열림");
                    if let Some(outcome) = restored {
                        tauri::async_runtime::block_on(backup::restore::record(
                            &db,
                            outcome,
                            OffsetDateTime::now_utc(),
                        ));
                    }
                    let backups =
                        Backups::new(db.clone(), data_dir.path()).with_restore_outcome(restored);
                    tauri::async_runtime::spawn(backups.clone().run_daily());
                    // 호스트·접속 선택이 생길 때까지는 늘 띄움
                    tauri::async_runtime::spawn(server::serve(host, db, backups.clone()));
                    // 앱을 끌 때 백업에 쓴다
                    app.manage(backups);
                }
                Err(problem) => {
                    app.manage(problem);
                }
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("창을 띄우지 못했습니다")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event
                && let Some(backups) = app.try_state::<Backups>()
            {
                // 창이 다 닫힌 뒤라 기다려도 직원은 기다리지 않는다
                tauri::async_runtime::block_on(
                    backups.make(BackupKind::Exit, OffsetDateTime::now_utc()),
                );
            }
        });
}
