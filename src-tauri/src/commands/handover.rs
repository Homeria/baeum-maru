//! 호스트 넘기기 명령(BR-H1, BR-H2): 저장 창과 다시 켜기. 본체는 [`crate::handover`].

use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;
use time::OffsetDateTime;

use crate::handover::{self, HandedOverAdmin, HandingOver, HandoverFileResponse};
use crate::run_mode::RunModeState;

/// 켤 때 만든 넘기기 파일의 미리보기.
///
/// # Errors
///
/// 넘기는 중이 아니거나 파일을 만들지 못했으면 화면에 보여 줄 문장.
#[tauri::command]
pub async fn handover_file(
    state: State<'_, Option<HandingOver>>,
) -> Result<HandoverFileResponse, String> {
    handover::preview(&state).await
}

/// 저장 창에서 고른 곳에 넘기기 파일을 저장하고, 이 PC를 "넘김"으로 바꿔 다시 켠다(BR-H1).
/// 저장 창을 닫으면 `false`를 돌려주고 아무것도 바꾸지 않는다.
///
/// # Errors
///
/// 넘기는 중이 아니거나, 저장하지 못했거나, 저장한 파일을 읽을 수 없으면 화면에 보여 줄 문장.
#[tauri::command]
pub async fn save_handover_file(
    app: AppHandle,
    state: State<'_, Option<HandingOver>>,
    run_mode: State<'_, RunModeState>,
) -> Result<bool, String> {
    handover::handing_over_file(&state)?;
    let (picked, chosen) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .set_file_name(handover::file_name(OffsetDateTime::now_utc()))
        .add_filter("배움마루 데이터", &["db"])
        .save_file(move |path| {
            let _ = picked.send(path);
        });
    let Some(path) = chosen
        .await
        .map_err(|_| "저장 창이 닫혔습니다".to_owned())?
    else {
        return Ok(false);
    };
    let target = path
        .into_path()
        .map_err(|_| "저장할 곳을 알 수 없습니다".to_owned())?;
    handover::save_file(&state, &run_mode, &target).await?;
    app.restart();
}

/// 넘기지 않고 다시 호스트로 켠다. 넘기는 중일 때만.
///
/// # Errors
///
/// 넘기는 중이 아니거나 기억하지 못하면 화면에 보여 줄 문장.
#[tauri::command]
pub async fn abort_handover(
    app: AppHandle,
    run_mode: State<'_, RunModeState>,
) -> Result<(), String> {
    handover::abort(&run_mode).await?;
    app.restart();
}

/// 넘기기를 취소할 수 있는 관리자들. 넘긴 PC에서만.
///
/// # Errors
///
/// 넘긴 PC가 아니거나 DB를 열지 못하면 화면에 보여 줄 문장.
#[tauri::command]
pub async fn handed_over_admins(
    run_mode: State<'_, RunModeState>,
) -> Result<Vec<HandedOverAdmin>, String> {
    handover::admins(&run_mode).await
}

/// 관리자 PIN을 확인하고 넘기기를 취소해 다시 호스트로 켠다(BR-H2).
///
/// # Errors
///
/// 넘긴 PC가 아니거나, 관리자가 아니거나, PIN이 틀렸거나, 기억하지 못하면 화면에 보여 줄 문장.
#[tauri::command]
pub async fn cancel_handover(
    app: AppHandle,
    run_mode: State<'_, RunModeState>,
    staff_id: i64,
    pin: String,
) -> Result<(), String> {
    handover::cancel(&run_mode, staff_id, pin).await?;
    app.restart();
}
