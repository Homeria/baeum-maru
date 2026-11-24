//! 접속 모드(ADR 0021)에서 기억하는 것: 호스트 주소(ADR 0004)와 기기 증표(BR-K16).
//!
//! 데이터 폴더의 파일 하나에 둔다. 화면은 앱 명령으로 읽고 바꾼다. 기기 증표는 비밀이라
//! 로그에 남기지 않는다.

use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tauri::State;
use ts_rs::TS;

use crate::run_mode::{RunMode, RunModeState};

/// 기억하는 파일. 데이터 폴더 안에 둔다.
const FILE: &str = "remote.json";

/// 주소의 가장 긴 글자 수 (DNS 이름의 한도).
const MAX_ADDRESS_CHARS: usize = 253;

/// 파일에 적는 모양.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Stored {
    host_address: Option<String>,
    /// 승인을 요청하며 받은 기기 증표. 허락받기 전에도 둔다(다시 켜도 기다리던 요청을 이어 본다).
    device_token: Option<String>,
    /// 그 요청의 승인 번호. 기다리는 동안 다시 보여 주려고 둔다.
    approval_code: Option<String>,
    /// 한 번이라도 승인받아 호스트에 이어졌는지. 그 전이면 PIN 없이 호스트로 되돌릴 수 있다(BR-K17).
    #[serde(default)]
    approved_once: bool,
}

fn read(data_dir: &Path) -> Stored {
    match std::fs::read(data_dir.join(FILE)) {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_else(|error| {
            tracing::warn!(%error, "접속 설정 파일을 읽지 못했습니다");
            Stored::default()
        }),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Stored::default(),
        Err(error) => {
            tracing::warn!(%error, "접속 설정 파일을 열지 못했습니다");
            Stored::default()
        }
    }
}

fn write(data_dir: &Path, stored: &Stored) -> io::Result<()> {
    std::fs::create_dir_all(data_dir)?;
    let text = serde_json::to_vec(stored).map_err(io::Error::other)?;
    std::fs::write(data_dir.join(FILE), text)
}

/// 직원이 넣은 호스트 주소를 다듬는다. 포트는 47800 고정이라 받지 않는다(ADR 0004).
///
/// `http://`, 끝의 `/`, `:47800`은 떼어 준다. 화면에 보이는 주소를 그대로 옮겨 적어도 되게.
///
/// # Errors
///
/// 비었거나, 주소에 쓸 수 없는 글자가 있으면 화면에 보여 줄 문장.
pub fn normalize_address(input: &str) -> Result<String, String> {
    let address = input.trim();
    let address = address.strip_prefix("http://").unwrap_or(address);
    let address = address.trim_end_matches('/');
    let address = address.strip_suffix(":47800").unwrap_or(address);
    if address.is_empty() {
        return Err("호스트 주소를 넣어 주세요".to_owned());
    }
    let usable = address.len() <= MAX_ADDRESS_CHARS
        && address
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-');
    if usable {
        Ok(address.to_owned())
    } else {
        Err("주소는 192.168.0.12처럼 숫자와 점으로 넣어 주세요".to_owned())
    }
}

/// 이 PC의 이름(예: 컴퓨터 이름). 승인 요청 때 호스트 화면에 보인다. 직원이 고칠 수 있다.
fn computer_name() -> Option<String> {
    // Windows는 COMPUTERNAME, 리눅스는 /etc/hostname
    std::env::var("COMPUTERNAME")
        .ok()
        .or_else(|| std::fs::read_to_string("/etc/hostname").ok())
        .map(|name| name.trim().to_owned())
        .filter(|name| !name.is_empty())
}

/// `remote_settings` 명령의 응답.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RemoteSettingsResponse {
    /// 기억한 호스트 주소. 아직 넣지 않았으면 `null`.
    pub host_address: Option<String>,
    /// 승인을 요청하며 받은 기기 증표. 없으면 `null`.
    pub device_token: Option<String>,
    /// 기다리는 요청의 승인 번호. 없으면 `null`.
    pub approval_code: Option<String>,
    /// 승인 요청에 미리 채울 이 PC의 이름. 찾지 못하면 `null`.
    pub computer_name: Option<String>,
    /// 한 번이라도 승인받아 호스트에 이어졌는지. 그 전이면 PIN 없이 호스트로 되돌린다(BR-K17).
    pub approved_once: bool,
}

/// 접속 모드일 때만 데이터 폴더를 준다.
fn remote_data_dir(state: &RunModeState) -> Result<&Path, String> {
    if state.mode() == Some(RunMode::Remote) {
        Ok(state.data_dir())
    } else {
        Err("다른 PC에 접속하는 모드가 아닙니다".to_owned())
    }
}

fn save_failed(error: io::Error) -> String {
    tracing::error!(%error, "접속 설정을 저장하지 못했습니다");
    "접속 설정을 저장하지 못했습니다. 데이터 폴더를 쓸 수 있는지 확인해 주세요".to_owned()
}

/// 접속 모드에서 기억한 것.
///
/// # Errors
///
/// 접속 모드가 아니면 화면에 보여 줄 문장.
#[tauri::command]
pub fn remote_settings(state: State<'_, RunModeState>) -> Result<RemoteSettingsResponse, String> {
    let stored = read(remote_data_dir(&state)?);
    Ok(RemoteSettingsResponse {
        host_address: stored.host_address,
        device_token: stored.device_token,
        approval_code: stored.approval_code,
        computer_name: computer_name(),
        approved_once: stored.approved_once,
    })
}

/// 호스트 주소를 다듬어 기억하고, 기억한 주소를 돌려준다. 기기 증표는 그대로 둔다.
///
/// 호스트 PC의 주소만 바뀐 경우가 많아서다. 다른 호스트라면 그 호스트가 증표를 받지 않고,
/// 화면이 다시 승인을 요청한다.
///
/// # Errors
///
/// 주소 모양이 틀렸거나 저장하지 못하면 화면에 보여 줄 문장.
#[tauri::command]
pub fn save_host_address(state: State<'_, RunModeState>, address: &str) -> Result<String, String> {
    let data_dir = remote_data_dir(&state)?;
    let address = normalize_address(address)?;
    let stored = Stored {
        host_address: Some(address.clone()),
        ..read(data_dir)
    };
    write(data_dir, &stored).map_err(save_failed)?;
    Ok(address)
}

/// 승인을 요청하며 받은 기기 증표와 승인 번호를 기억한다.
///
/// # Errors
///
/// 저장하지 못하면 화면에 보여 줄 문장.
#[tauri::command]
pub fn save_device_request(
    state: State<'_, RunModeState>,
    token: String,
    code: String,
) -> Result<(), String> {
    let data_dir = remote_data_dir(&state)?;
    let stored = Stored {
        device_token: Some(token),
        approval_code: Some(code),
        ..read(data_dir)
    };
    write(data_dir, &stored).map_err(save_failed)
}

/// 기기 증표와 승인 번호를 버린다. 거절·만료되었거나 승인을 취소당했을 때.
///
/// # Errors
///
/// 저장하지 못하면 화면에 보여 줄 문장.
#[tauri::command]
pub fn clear_device_request(state: State<'_, RunModeState>) -> Result<(), String> {
    let data_dir = remote_data_dir(&state)?;
    let stored = Stored {
        device_token: None,
        approval_code: None,
        ..read(data_dir)
    };
    write(data_dir, &stored).map_err(save_failed)
}

/// 승인받아 호스트에 이어졌다고 기억한다. 이 뒤로는 호스트로 바꿀 때 관리자 PIN이 필요하다.
///
/// # Errors
///
/// 저장하지 못하면 화면에 보여 줄 문장.
#[tauri::command]
pub fn remember_approved(state: State<'_, RunModeState>) -> Result<(), String> {
    let data_dir = remote_data_dir(&state)?;
    let stored = read(data_dir);
    if stored.approved_once {
        return Ok(());
    }
    let stored = Stored {
        approved_once: true,
        ..stored
    };
    write(data_dir, &stored).map_err(save_failed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_shown_on_host_can_be_copied_as_is() {
        for input in [
            "192.168.0.12",
            " 192.168.0.12 ",
            "http://192.168.0.12",
            "http://192.168.0.12:47800/",
            "192.168.0.12:47800",
        ] {
            assert_eq!(
                normalize_address(input).as_deref(),
                Ok("192.168.0.12"),
                "{input}"
            );
        }
        assert_eq!(
            normalize_address("host-pc.local").as_deref(),
            Ok("host-pc.local")
        );
    }

    #[test]
    fn unusable_address_is_refused() {
        for input in [
            "",
            "  ",
            "192.168.0.12:8080",
            "호스트",
            "192.168.0.12/api",
            "a b",
        ] {
            assert!(normalize_address(input).is_err(), "{input}");
        }
    }

    #[test]
    fn saving_one_thing_keeps_the_other() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            &Stored {
                device_token: Some("증표".to_owned()),
                approval_code: Some("123456".to_owned()),
                ..Stored::default()
            },
        )
        .unwrap();

        let stored = Stored {
            host_address: Some("192.168.0.12".to_owned()),
            ..read(dir.path())
        };
        write(dir.path(), &stored).unwrap();

        let read_back = read(dir.path());
        assert_eq!(read_back.host_address.as_deref(), Some("192.168.0.12"));
        assert_eq!(read_back.device_token.as_deref(), Some("증표"));
    }

    #[test]
    fn older_file_without_approved_once_reads_as_not_yet() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(FILE), r#"{"hostAddress":"192.168.0.12"}"#).unwrap();

        let stored = read(dir.path());

        assert_eq!(stored.host_address.as_deref(), Some("192.168.0.12"));
        assert!(!stored.approved_once);
    }

    #[test]
    fn missing_or_broken_file_means_nothing_saved() {
        let dir = tempfile::tempdir().unwrap();
        assert!(read(dir.path()).host_address.is_none());

        std::fs::write(dir.path().join(FILE), "망가진 내용").unwrap();
        assert!(read(dir.path()).host_address.is_none());
    }
}
