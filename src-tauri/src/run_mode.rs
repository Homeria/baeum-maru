//! 실행 모드(ADR 0021): 이 PC의 앱이 호스트로 켜는지, 다른 PC의 호스트에 접속하는지.
//!
//! 처음 켤 때 한 번 고르고 데이터 폴더에 기억한다. 이미 DB가 있는 PC는 묻지 않고 호스트로 켠다.

use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use ts_rs::TS;

use crate::db;

/// 실행 모드를 기억하는 파일. 데이터 폴더 안에 둔다.
const FILE: &str = "run-mode.json";

/// 이 PC의 앱이 켜는 방식.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum RunMode {
    /// 이 PC가 DB를 갖고 서버를 띄운다.
    Host,
    /// 다른 PC의 호스트에 접속한다. 이 PC의 DB는 열지 않는다.
    Remote,
}

/// 파일에 적는 모양. 나중에 칸이 늘어도 읽을 수 있게 객체로 둔다.
#[derive(Debug, Serialize, Deserialize)]
struct Stored {
    mode: RunMode,
}

/// 기억한 실행 모드를 읽는다. 아직 고르지 않았으면 `None`.
///
/// 파일이 없어도 DB가 있으면 호스트다. 이 기능 전부터 쓰던 PC다.
/// 파일을 읽지 못하면 없는 것처럼 다룬다. 접속 모드 PC라도 DB가 없으면 다시 묻기만 한다.
pub fn resolve(data_dir: &Path) -> Option<RunMode> {
    match std::fs::read(data_dir.join(FILE)) {
        Ok(bytes) => match serde_json::from_slice::<Stored>(&bytes) {
            Ok(stored) => return Some(stored.mode),
            Err(error) => tracing::warn!(%error, "실행 모드 파일을 읽지 못했습니다"),
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => tracing::warn!(%error, "실행 모드 파일을 열지 못했습니다"),
    }
    db::file(data_dir).exists().then_some(RunMode::Host)
}

/// 실행 모드를 기억한다. 데이터 폴더가 없으면 만든다.
pub fn save(data_dir: &Path, mode: RunMode) -> io::Result<()> {
    std::fs::create_dir_all(data_dir)?;
    let text = serde_json::to_vec(&Stored { mode }).map_err(io::Error::other)?;
    std::fs::write(data_dir.join(FILE), text)
}

/// 켤 때 정한 실행 모드. 앱에 맡겨 두고 명령이 꺼내 쓴다.
#[derive(Debug, Clone)]
pub struct RunModeState {
    data_dir: PathBuf,
    mode: Option<RunMode>,
}

impl RunModeState {
    pub fn new(data_dir: &Path, mode: Option<RunMode>) -> Self {
        RunModeState {
            data_dir: data_dir.to_owned(),
            mode,
        }
    }
}

/// `run_mode_state` 명령의 응답.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RunModeResponse {
    /// 아직 고르지 않았으면 `null`.
    pub mode: Option<RunMode>,
    /// 접속 모드인데 이 PC에 예전 DB가 남아 있다. 지우지 않고 쓰지도 않는다.
    pub leftover_data: bool,
}

/// 켤 때 정한 실행 모드.
#[tauri::command]
pub fn run_mode_state(state: State<'_, RunModeState>) -> RunModeResponse {
    RunModeResponse {
        mode: state.mode,
        leftover_data: state.mode == Some(RunMode::Remote) && db::file(&state.data_dir).exists(),
    }
}

/// 처음 켤 때 고른 실행 모드를 기억하고 앱을 다시 켠다. 성공하면 돌아오지 않는다.
///
/// # Errors
///
/// 이미 정해졌거나 기억하지 못하면 화면에 보여 줄 문장.
#[tauri::command]
pub fn choose_run_mode(
    app: AppHandle,
    state: State<'_, RunModeState>,
    mode: RunMode,
) -> Result<(), String> {
    if state.mode.is_some() {
        // 바꾸기는 관리자 PIN을 확인하는 다른 길로만 한다 (BR-K17)
        return Err("실행 모드가 이미 정해져 있습니다. 앱을 다시 켜 주세요".to_owned());
    }
    save(&state.data_dir, mode).map_err(|error| {
        tracing::error!(%error, "실행 모드를 기억하지 못했습니다");
        "실행 모드를 기억하지 못했습니다. 데이터 폴더를 쓸 수 있는지 확인해 주세요".to_owned()
    })?;
    tracing::info!(?mode, "실행 모드를 정했습니다");
    app.restart();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undecided_on_fresh_pc() {
        let dir = tempfile::tempdir().unwrap();

        assert_eq!(resolve(dir.path()), None);
    }

    #[test]
    fn pc_with_db_is_host_without_asking() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(db::file(dir.path()), "").unwrap();

        assert_eq!(resolve(dir.path()), Some(RunMode::Host));
    }

    #[test]
    fn remembers_chosen_mode() {
        let dir = tempfile::tempdir().unwrap();

        save(dir.path(), RunMode::Remote).unwrap();

        assert_eq!(resolve(dir.path()), Some(RunMode::Remote));
    }

    #[test]
    fn remote_stays_remote_with_leftover_db() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(db::file(dir.path()), "").unwrap();

        save(dir.path(), RunMode::Remote).unwrap();

        assert_eq!(resolve(dir.path()), Some(RunMode::Remote));
    }

    #[test]
    fn creates_missing_data_folder() {
        let dir = tempfile::tempdir().unwrap();
        let data_dir = dir.path().join("새 폴더");

        save(&data_dir, RunMode::Host).unwrap();

        assert_eq!(resolve(&data_dir), Some(RunMode::Host));
    }

    #[test]
    fn unreadable_file_falls_back_to_db() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(FILE), "망가진 내용").unwrap();

        assert_eq!(resolve(dir.path()), None);

        std::fs::write(db::file(dir.path()), "").unwrap();
        assert_eq!(resolve(dir.path()), Some(RunMode::Host));
    }
}
