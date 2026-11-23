//! 앱 다시 켜기(ADR 0011). 복원을 준비한 뒤 화면이 부른다.
//!
//! 서버(axum)는 창을 다시 켤 손잡이가 없어서, 복원 준비가 끝나면 화면이 이 명령을 부른다.
//! 다시 켜지지 않아도 받아 둔 파일은 남아 다음에 켤 때 복원된다.

use tauri::AppHandle;

/// 앱을 다시 켠다. 돌아오지 않는다.
#[tauri::command]
pub fn restart_app(app: AppHandle) {
    tracing::info!("복원하려고 앱을 다시 켬");
    app.restart();
}
