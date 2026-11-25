//! 깨진 DB 복원 명령(ADR 0011 DB 손상 시 복원): 켤 때 맡겨 둔 상태 꺼내기와 다시 켜기.
//! 본체는 [`baeum_server::broken_db`].

use tauri::{AppHandle, Manager, State};

use baeum_server::broken_db::{
    self, BrokenDb, RestorePreviewResponse, RestoreRequest, StartupStateResponse,
};

/// 켤 때 DB 상태. 깨졌으면 백업 목록과 권하는 백업도 준다.
#[tauri::command]
pub async fn startup_state(app: AppHandle) -> StartupStateResponse {
    broken_db::startup_state(broken(&app).as_deref()).await
}

/// 깨졌을 때 고른 백업의 미리보기.
///
/// # Errors
///
/// DB가 깨지지 않았거나 쓸 수 없는 파일이면 화면에 보여 줄 문장.
#[tauri::command]
pub async fn preview_broken_db_backup(
    app: AppHandle,
    request: RestoreRequest,
) -> Result<RestorePreviewResponse, String> {
    broken_db::preview(broken(&app).as_deref(), request).await
}

/// 깨진 DB를 고른 백업으로 되돌리고 앱을 다시 켠다. 성공하면 돌아오지 않는다.
///
/// # Errors
///
/// DB가 깨지지 않았거나 되돌리지 못하면 화면에 보여 줄 문장. 깨진 DB는 그대로다.
#[tauri::command]
pub async fn restore_broken_db(app: AppHandle, request: RestoreRequest) -> Result<(), String> {
    broken_db::restore(broken(&app).as_deref(), request).await?;
    app.restart();
}

/// 켤 때 DB를 쓸 수 없었으면 맡겨 둔 상태. 멀쩡하면 맡기지 않았다.
fn broken(app: &AppHandle) -> Option<State<'_, BrokenDb>> {
    app.try_state::<BrokenDb>()
}
