//! 배움마루 앱.
//!
//! 창(Tauri)을 띄우고, 같은 프로세스에서 HTTP 서버를 돌린다.

mod server;
mod version;

use tracing_subscriber::EnvFilter;

/// 앱을 시작한다. 창을 닫으면 돌아온다.
///
/// # Panics
///
/// 창을 띄우지 못하면 멈춘다.
pub fn run() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    tauri::Builder::default()
        .setup(|_app| {
            let host: version::Version = version::CURRENT.parse()?;
            // 호스트·접속 선택이 생길 때까지는 늘 띄움
            tauri::async_runtime::spawn(server::serve(host));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("창을 띄우지 못했습니다");
}
