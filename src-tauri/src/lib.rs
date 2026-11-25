//! 배움마루 앱.
//!
//! 창(Tauri)을 띄우고, 같은 프로세스에서 HTTP 서버를 돌린다.

mod backup;
mod backup_file;
mod broken_db;
mod clock;
mod commands;
mod data_dir;
mod db;
mod handover;
mod host_screen;
mod pick;
mod pin_check;
mod remote;
mod restart;
mod roster_xlsx;
mod run_mode;
mod save_file;
mod secret;
#[cfg(feature = "dev-seed")]
pub mod seed;
mod server;
mod startup;
mod take_over;
mod version;

use tauri::Manager;
use tracing_subscriber::EnvFilter;

use time::OffsetDateTime;

use crate::backup::Backups;
use crate::backup_file::name::BackupKind;
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
            pick::pick_handover_file,
            restart::restart_app,
            broken_db::startup_state,
            broken_db::preview_broken_db_backup,
            broken_db::restore_broken_db,
            commands::run_mode::run_mode_state,
            host_screen::host_address,
            host_screen::open_device_request_count,
            remote::remote_settings,
            remote::save_host_address,
            remote::save_device_request,
            remote::clear_device_request,
            remote::remember_approved,
            remote::check_host_identity,
            remote::remember_host_identity,
            remote::forget_host,
            commands::run_mode::choose_run_mode,
            commands::run_mode::change_run_mode,
            commands::handover::handover_file,
            commands::handover::save_handover_file,
            commands::handover::abort_handover,
            commands::handover::handed_over_admins,
            commands::handover::cancel_handover,
            commands::take_over::preview_take_over,
            commands::take_over::take_over
        ])
        .setup(|app| {
            let host: version::Version = version::CURRENT.parse()?;
            let exe = std::env::current_exe()?;
            let exe_dir = exe
                .parent()
                .ok_or("실행 파일이 있는 폴더를 찾지 못했습니다")?;
            let data_dir = DataDir::resolve(exe_dir, app.path().app_data_dir()?);
            // ADR 0021: 호스트일 때만 DB를 열고 서버와 백업을 켠다. 아직 고르지 않았으면 화면이 묻는다
            let mode = run_mode::resolve(data_dir.path());
            app.manage(run_mode::RunModeState::new(data_dir.path(), mode));
            // BR-H1: 넘기는 중이면 서버 없이 DB만 열어 넘기기 파일을 만든다. 그 밖에는 None을 맡긴다
            let handing_over = (mode == Some(run_mode::RunMode::HandingOver)).then(|| {
                tauri::async_runtime::block_on(handover::prepare(
                    data_dir.path(),
                    OffsetDateTime::now_utc(),
                ))
            });
            app.manage(handing_over);
            if mode != Some(run_mode::RunMode::Host) {
                tracing::info!(?mode, "호스트가 아니라서 DB를 열지 않습니다");
                return Ok(());
            }
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
                    // 호스트 화면의 앱 명령이 쓴다 (host_screen)
                    app.manage(db.clone());
                    tauri::async_runtime::spawn(server::serve(host, db, backups.clone()));
                    // 앱을 끌 때 백업에 쓴다
                    app.manage(backups);
                }
                Err(problem) => {
                    app.manage(broken_db::BrokenDb::new(data_dir.path(), problem));
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
