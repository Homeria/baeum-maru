//! 받는 PC에 파일을 저장한다(ADR 0018).
//!
//! 화면이 호스트에서 받은 파일 내용을 이 명령에 넘기면, 앱이 저장 창을 띄우고 직원이 고른
//! 곳에 쓴다. 화면에는 파일 쓰기 권한을 열지 않는다.

use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

/// 저장 창을 띄워 `contents`를 저장한다. 저장한 파일 경로를 돌려준다.
/// 직원이 저장 창을 닫으면 `None`이다.
///
/// `file_name`은 저장 창에 미리 채울 이름이다. 예: `2027년 1학기 게시용 강좌별.xlsx`
///
/// # Errors
///
/// 고른 곳에 쓰지 못하면 화면에 보여 줄 문장.
#[tauri::command]
pub async fn save_file(
    app: AppHandle,
    file_name: String,
    contents: Vec<u8>,
) -> Result<Option<String>, String> {
    let (picked, chosen) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .set_file_name(suggested_name(&file_name))
        .add_filter("Excel 파일", &["xlsx"])
        .save_file(move |path| {
            // 받는 쪽이 사라졌으면(명령이 끝남) 할 일이 없다
            let _ = picked.send(path);
        });
    let Some(path) = chosen
        .await
        .map_err(|_| "저장 창이 닫혔습니다".to_owned())?
    else {
        return Ok(None);
    };
    let path = path
        .into_path()
        .map_err(|_| "저장할 곳을 알 수 없습니다".to_owned())?;
    tokio::fs::write(&path, contents).await.map_err(|error| {
        // 경로에는 직원 이름 같은 폴더 이름이 있을 수 있어 로그에 남기지 않는다
        tracing::warn!(kind = ?error.kind(), "파일을 저장하지 못했습니다");
        "파일을 저장하지 못했습니다. 다른 곳을 골라 주세요".to_owned()
    })?;
    Ok(Some(path.display().to_string()))
}

/// 저장 창에 미리 채울 이름. 폴더를 가리키는 글자(`/`, `\`)와 Windows가 쓰지 못하게 하는
/// 글자를 지운다. 비면 `결과 명단.xlsx`.
fn suggested_name(file_name: &str) -> String {
    let name: String = file_name
        .chars()
        .filter(|c| !matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'))
        .filter(|c| !c.is_control())
        .collect();
    let name = name.trim().trim_start_matches('.');
    if name.is_empty() {
        "결과 명단.xlsx".to_owned()
    } else {
        name.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suggested_name_cannot_point_to_another_folder() {
        assert_eq!(
            suggested_name("2027년 1학기 게시용 강좌별.xlsx"),
            "2027년 1학기 게시용 강좌별.xlsx"
        );
        assert_eq!(suggested_name("../../a/b:c.xlsx"), "abc.xlsx");
        assert_eq!(suggested_name("  "), "결과 명단.xlsx");
    }
}
