//! 실행 모드 명령(ADR 0021): 지금 모드 보기, 처음 고르기, 바꾸기.

use serde::Serialize;
use tauri::{AppHandle, State};
use ts_rs::TS;

use baeum_server::run_mode::{self, RunMode, RunModeState};

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
        mode: state.mode(),
        leftover_data: state.has_leftover_data(),
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
    if !mode.is_choosable() {
        return Err("고를 수 없는 실행 모드입니다".to_owned());
    }
    if state.mode().is_some() {
        // 바꾸기는 관리자 PIN을 확인하는 다른 길로만 한다 (BR-K17)
        return Err("실행 모드가 이미 정해져 있습니다. 앱을 다시 켜 주세요".to_owned());
    }
    save(&state, mode)?;
    tracing::info!(?mode, "실행 모드를 정했습니다");
    app.restart();
}

/// 실행 모드를 바꾸고 앱을 다시 켠다. 성공하면 돌아오지 않는다(BR-K17).
///
/// 관리자 PIN 확인은 화면이 먼저 호스트에 받아 둔다(`/api/run-mode/permit`). 직원의 실수를
/// 막는 장치이고, 이 PC를 마음대로 다루는 사람까지 막지는 않는다. 이 PC의 DB는 지우지 않는다.
///
/// # Errors
///
/// 아직 고르지 않았거나, 같은 모드거나, 기억하지 못하면 화면에 보여 줄 문장.
#[tauri::command]
pub fn change_run_mode(
    app: AppHandle,
    state: State<'_, RunModeState>,
    mode: RunMode,
) -> Result<(), String> {
    match state.mode() {
        None => return Err("아직 실행 모드를 고르지 않았습니다".to_owned()),
        Some(current) if !run_mode::can_change(current, mode) => {
            return Err("그 모드로 바꿀 수 없습니다".to_owned());
        }
        Some(_) => {}
    }
    save(&state, mode)?;
    tracing::info!(?mode, "실행 모드를 바꿨습니다");
    app.restart();
}

/// 실행 모드를 기억한다. 실패하면 화면에 보여 줄 문장.
fn save(state: &RunModeState, mode: RunMode) -> Result<(), String> {
    run_mode::save(state.data_dir(), mode).map_err(|error| {
        tracing::error!(%error, "실행 모드를 기억하지 못했습니다");
        "실행 모드를 기억하지 못했습니다. 데이터 폴더를 쓸 수 있는지 확인해 주세요".to_owned()
    })
}
