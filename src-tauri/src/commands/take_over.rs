//! 넘겨받기 명령(BR-H3): 다시 켜기. 본체는 [`crate::take_over`].

use std::path::Path;

use tauri::{AppHandle, State};

use crate::run_mode::RunModeState;
use crate::take_over::{self, TakeOverPreviewResponse};

/// 고른 넘기기 파일을 살펴보고 미리보기와 관리자 목록을 돌려준다.
///
/// # Errors
///
/// 넘겨받을 수 없는 PC거나 파일이면 화면에 보여 줄 문장.
#[tauri::command]
pub async fn preview_take_over(
    run_mode: State<'_, RunModeState>,
    path: String,
) -> Result<TakeOverPreviewResponse, String> {
    take_over::preview_for(&run_mode, Path::new(&path)).await
}

/// 파일 안 관리자의 PIN으로 확인하고 이 PC를 새 호스트로 바꿔 다시 켠다(BR-H3).
///
/// # Errors
///
/// 넘겨받을 수 없는 PC나 파일이거나, PIN이 틀렸거나, 옮기지 못하면 화면에 보여 줄 문장.
/// 이 PC는 그대로다.
#[tauri::command]
pub async fn take_over(
    app: AppHandle,
    run_mode: State<'_, RunModeState>,
    path: String,
    staff_id: i64,
    pin: String,
) -> Result<(), String> {
    take_over::take_over(&run_mode, Path::new(&path), staff_id, pin).await?;
    app.restart();
}
