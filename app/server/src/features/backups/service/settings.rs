//! 두 번째 위치 설정(ADR 0011 구현 메모). 데이터 폴더의 `backup-settings.json`에 둔다.
//!
//! 경로는 그 PC에만 뜻이 있어서 DB에 두지 않는다. 데이터를 넘기거나 복원해도 따라가거나
//! 되돌아가지 않는다.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// 설정 파일 이름.
const FILE_NAME: &str = "backup-settings.json";

/// 백업 설정.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupSettings {
    /// 하루 한 번 백업을 한 부 더 복사할 폴더. 없으면 꺼짐(기본).
    pub second_location: Option<PathBuf>,
}

/// 데이터 폴더 `data_dir` 안의 설정 파일.
pub fn file(data_dir: &Path) -> PathBuf {
    data_dir.join(FILE_NAME)
}

/// 설정을 읽는다. 파일이 없거나 읽을 수 없으면 기본값(두 번째 위치 꺼짐).
pub async fn load(file: &Path) -> BackupSettings {
    match tokio::fs::read(file).await {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_else(|error| {
            tracing::warn!(%error, "백업 설정을 읽지 못해 기본값으로 봄");
            BackupSettings::default()
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => BackupSettings::default(),
        Err(error) => {
            tracing::warn!(%error, "백업 설정을 읽지 못해 기본값으로 봄");
            BackupSettings::default()
        }
    }
}

/// 설정을 저장한다. 다른 이름으로 다 쓴 뒤 이름을 바꾼다.
pub async fn save(file: &Path, settings: &BackupSettings) -> std::io::Result<()> {
    let bytes = serde_json::to_vec_pretty(settings).map_err(std::io::Error::other)?;
    let mut part = file.as_os_str().to_owned();
    part.push(".part");
    tokio::fs::write(&part, bytes).await?;
    tokio::fs::rename(&part, file).await
}
