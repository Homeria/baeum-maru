//! 배움마루 앱.
//!
//! 창(Tauri)을 띄우고, 같은 프로세스에서 HTTP 서버(`baeum-server`)를 돌린다.

mod commands;
mod data_dir;

use tauri::Manager;
use tracing_subscriber::EnvFilter;

use time::OffsetDateTime;

use baeum_server::backup::Backups;
use baeum_server::backup_file::name::BackupKind;
use baeum_server::{backup, broken_db, handover, run_mode, server, startup, version};

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
            commands::save_file::save_file,
            commands::pick::pick_folder,
            commands::pick::pick_backup_file,
            commands::pick::pick_handover_file,
            commands::restart::restart_app,
            commands::broken_db::startup_state,
            commands::broken_db::preview_broken_db_backup,
            commands::broken_db::restore_broken_db,
            commands::run_mode::run_mode_state,
            commands::host_screen::host_address,
            commands::host_screen::open_device_request_count,
            commands::remote::remote_settings,
            commands::remote::save_host_address,
            commands::remote::save_device_request,
            commands::remote::clear_device_request,
            commands::remote::remember_approved,
            commands::remote::check_host_identity,
            commands::remote::remember_host_identity,
            commands::remote::forget_host,
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
