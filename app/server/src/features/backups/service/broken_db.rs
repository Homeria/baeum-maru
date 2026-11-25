//! DB가 깨졌을 때 로그인 없이 복원하기(ADR 0011 DB 손상 시 복원). Tauri를 모르고, 앱 명령은
//! `commands::broken_db`가 한다.
//!
//! 서버는 띄우지 않는다. 앱 명령은 그 PC의 창에서만 부를 수 있다. 복원 명령은 켤 때 DB가
//! 깨졌다고 가린 경우에만 동작한다. 화면에는 데이터를 보여 주지 않고 개수와 학기 이름만 준다.

use std::path::{Path, PathBuf};

use time::OffsetDateTime;

use super::inspect::BackupPreview;
use super::inspect::inspect;
use super::restore::{self, BrokenRestoreError};
use super::source::RestoreSource;
use crate::features::backups::dto::{BackupKindCode, local_text};
/// 화면이 받는 모양. launcher가 이 모듈 이름으로 찾는다.
pub use crate::features::backups::dto::{
    BrokenDbBackupResponse, BrokenDbSuggestion, StartupStateCode, StartupStateResponse,
};
/// 미리보기와 복원 요청은 로그인한 뒤 백업 탭의 복원과 같은 모양이다.
pub use crate::features::backups::dto::{RestorePreviewResponse, RestoreRequest};
use crate::infra::backup_file::{self, name::BackupName};
use crate::infra::startup::DbProblem;

/// 켤 때 DB를 쓸 수 없었다. 앱에 맡겨 두고 명령이 꺼내 쓴다.
#[derive(Debug, Clone)]
pub struct BrokenDb {
    data_dir: PathBuf,
    problem: DbProblem,
}

impl BrokenDb {
    pub fn new(data_dir: &Path, problem: DbProblem) -> Self {
        BrokenDb {
            data_dir: data_dir.to_owned(),
            problem,
        }
    }
}

/// 켤 때 DB 상태. 깨졌으면 백업 목록과 권하는 백업도 준다. `broken`은 켤 때 맡겨 둔 상태다.
pub async fn startup_state(broken: Option<&BrokenDb>) -> StartupStateResponse {
    let Some(broken) = broken else {
        return StartupStateResponse {
            state: StartupStateCode::Ready,
            backups: Vec::new(),
            suggestion: None,
        };
    };
    let state = match broken.problem {
        DbProblem::Broken => StartupStateCode::Broken,
        DbProblem::NewerVersion => StartupStateCode::NewerVersion,
        DbProblem::Unavailable => StartupStateCode::Unavailable,
    };
    if state != StartupStateCode::Broken {
        return StartupStateResponse {
            state,
            backups: Vec::new(),
            suggestion: None,
        };
    }
    let (backups, suggestion) = restore::suggest(&backup_file::dir(&broken.data_dir)).await;
    StartupStateResponse {
        state,
        backups: backups
            .into_iter()
            .map(|backup| BrokenDbBackupResponse {
                file_name: backup.name.to_string(),
                taken_at: local_text(backup.name.taken_at),
                kind: BackupKindCode::split(backup.name.kind).0,
            })
            .collect(),
        suggestion: suggestion.map(|(name, preview)| BrokenDbSuggestion {
            file_name: name.to_string(),
            preview: preview_response(Some(name), preview),
        }),
    }
}

/// 깨졌을 때 고른 백업의 미리보기.
///
/// # Errors
///
/// DB가 깨지지 않았거나 쓸 수 없는 파일이면 화면에 보여 줄 문장.
pub async fn preview(
    broken: Option<&BrokenDb>,
    request: RestoreRequest,
) -> Result<RestorePreviewResponse, String> {
    let data_dir = broken_data_dir(broken)?;
    let source = resolve(&data_dir, request).await?;
    let preview = inspect(&source.file)
        .await
        .map_err(|error| error.to_string())?;
    Ok(preview_response(source.name, preview))
}

/// 깨진 DB를 고른 백업으로 되돌린다. 다시 켜기는 부르는 쪽이 한다.
///
/// # Errors
///
/// DB가 깨지지 않았거나 되돌리지 못하면 화면에 보여 줄 문장. 깨진 DB는 그대로다.
pub async fn restore(broken: Option<&BrokenDb>, request: RestoreRequest) -> Result<(), String> {
    let data_dir = broken_data_dir(broken)?;
    let source = resolve(&data_dir, request).await?;
    restore::restore_broken(&data_dir, &source.file, OffsetDateTime::now_utc())
        .await
        .map_err(|error| {
            tracing::error!(%error, "깨진 DB를 되돌리지 못했습니다");
            match error {
                BrokenRestoreError::Inspect(error) => error.to_string(),
                BrokenRestoreError::Io(_) | BrokenRestoreError::Open(_) => {
                    "되돌리지 못했습니다. 다른 백업을 골라 주세요".to_owned()
                }
            }
        })
}

/// DB가 깨졌을 때만 데이터 폴더를 준다. 멀쩡하면 로그인 없이 복원할 수 없다.
fn broken_data_dir(broken: Option<&BrokenDb>) -> Result<PathBuf, String> {
    match broken {
        Some(broken) if broken.problem == DbProblem::Broken => Ok(broken.data_dir.clone()),
        _ => Err(
            "DB가 깨졌을 때만 여기서 복원할 수 있습니다. 로그인한 뒤 백업 탭에서 복원해 주세요"
                .to_owned(),
        ),
    }
}

async fn resolve(data_dir: &Path, request: RestoreRequest) -> Result<RestoreSource, String> {
    RestoreSource::resolve(&backup_file::dir(data_dir), request.file_name, request.path)
        .await
        .map_err(|error| error.to_string())
}

fn preview_response(name: Option<BackupName>, preview: BackupPreview) -> RestorePreviewResponse {
    RestorePreviewResponse {
        taken_at: name.map(|name| local_text(name.taken_at)),
        kind: name.map(|name| BackupKindCode::split(name.kind).0),
        term_names: preview.term_names,
        member_count: preview.member_count,
        application_count: preview.application_count,
    }
}
