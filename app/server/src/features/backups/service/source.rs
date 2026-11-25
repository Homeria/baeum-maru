//! 복원할 백업 파일 고르기. 목록의 파일 이름이나 다른 곳의 경로 중 하나를 받는다.
//!
//! 평소 복원(서버)과 DB 손상 시 복원(앱 명령)이 함께 쓴다.

use std::path::{Path, PathBuf};

use crate::features::backups::error::SourceError;
use crate::infra::backup_file::name::BackupName;

/// 복원할 백업 파일.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreSource {
    pub file: PathBuf,
    /// 파일 이름이 백업 이름 모양이면 그 이름.
    pub name: Option<BackupName>,
    /// 오류를 붙일 요청 칸. `fileName` 또는 `path`
    pub field: &'static str,
}

impl RestoreSource {
    /// `file_name`(백업 폴더 `backups_dir`의 목록 이름)이나 `path`(다른 곳) 중 하나를 받는다.
    pub async fn resolve(
        backups_dir: &Path,
        file_name: Option<String>,
        path: Option<String>,
    ) -> Result<Self, SourceError> {
        let source = match (file_name, path) {
            (Some(file_name), None) => {
                // 백업 이름 모양만 받아서 백업 폴더 밖을 가리키지 못하게 한다
                let name = BackupName::parse(&file_name).ok_or(SourceError::NotInList)?;
                RestoreSource {
                    file: backups_dir.join(name.to_string()),
                    name: Some(name),
                    field: "fileName",
                }
            }
            (None, Some(path)) => {
                let file = PathBuf::from(path.trim());
                if !file.is_absolute() {
                    return Err(SourceError::NotAbsolute);
                }
                let name = file
                    .file_name()
                    .and_then(|name| name.to_str())
                    .and_then(BackupName::parse);
                RestoreSource {
                    file,
                    name,
                    field: "path",
                }
            }
            _ => return Err(SourceError::Malformed),
        };
        let is_file = tokio::fs::metadata(&source.file)
            .await
            .is_ok_and(|metadata| metadata.is_file());
        if is_file {
            Ok(source)
        } else {
            Err(SourceError::Missing {
                field: source.field,
            })
        }
    }
}
