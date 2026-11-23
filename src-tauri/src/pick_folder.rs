//! 두 번째 위치로 쓸 폴더를 고른다(ADR 0011, ADR 0018과 같은 방식).
//!
//! 폴더 고르기 창은 앱이 띄운다. 화면에는 파일 권한을 열지 않는다.

use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

/// 폴더 고르기 창을 띄워 고른 폴더 경로를 돌려준다. 창을 닫으면 `None`이다.
///
/// # Errors
///
/// 창이 응답 없이 사라졌거나 경로를 알 수 없으면 화면에 보여 줄 문장.
#[tauri::command]
pub async fn pick_folder(app: AppHandle) -> Result<Option<String>, String> {
    let (picked, chosen) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .set_title("백업을 한 부 더 둘 폴더")
        .pick_folder(move |path| {
            // 받는 쪽이 사라졌으면(명령이 끝남) 할 일이 없다
            let _ = picked.send(path);
        });
    let Some(path) = chosen
        .await
        .map_err(|_| "폴더 고르기 창이 닫혔습니다".to_owned())?
    else {
        return Ok(None);
    };
    let path = path
        .into_path()
        .map_err(|_| "고른 폴더를 알 수 없습니다".to_owned())?;
    Ok(Some(path.display().to_string()))
}
