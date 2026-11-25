//! 백업 요청 처리 `/api/backups`: 백업 목록, 지금 백업, 두 번째 위치 바꾸기(ADR 0011). 관리자만 한다.
//!
//! 두 번째 위치는 그 PC의 폴더라서 호스트 PC에서만 바꾼다. 다른 PC에서는 보기만 한다.
//! 복원도 호스트 PC에서만 한다. 호스트 앱을 다시 켜야 하고, 고른 파일도 호스트의 경로다.

use std::path::PathBuf;

use axum::extract::FromRef;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use baeum_domain::Action;
use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::dto::{
    BackupFailureResponse, BackupKindCode, BackupListResponse, BackupResponse,
    RestoreOutcomeResponse, RestorePreviewResponse, RestoreRequest, SecondLocationRequest,
    SecondLocationResponse, local_text,
};
use crate::features::auth::middleware::{Client, CurrentStaff};
use crate::features::backups::error::{
    InspectError, PrepareError, SecondLocationError, SourceError,
};
use crate::features::backups::service::inspect;
use crate::features::backups::service::restore::RestoreOutcome;
use crate::features::backups::service::source::RestoreSource;
use crate::features::backups::service::{Backups, SecondLocationStatus};
use crate::features::courses::repository as courses;
use crate::http::changes::{ChangeKind, ChangeNotices, notifies};
use crate::http::error::{ApiError, FieldError, field_error};

/// 백업 요청 주소. 로그인한 요청만 받는 묶음에 들어간다.
pub(crate) fn routes<S>(notices: &ChangeNotices) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    SqlitePool: FromRef<S>,
    Backups: FromRef<S>,
{
    Router::new()
        .route(
            "/api/backups",
            notifies(
                get(list_backups).post(create_backup),
                notices,
                ChangeKind::Backup,
            ),
        )
        .route(
            "/api/backups/second-location",
            notifies(put(set_second_location), notices, ChangeKind::Backup),
        )
        .route("/api/backups/restore-preview", post(preview_restore))
        .route("/api/backups/restore", post(restore))
}

/// 백업 목록, 마지막 자동 백업 실패, 두 번째 위치.
async fn list_backups(
    State(db): State<SqlitePool>,
    State(backups): State<Backups>,
    staff: CurrentStaff,
    client: Client,
) -> Result<Json<BackupListResponse>, ApiError> {
    staff.require(Action::ManageBackups)?;
    let stored = backups.list().await.map_err(ApiError::internal)?;
    let names = courses::names(&db).await.map_err(ApiError::internal)?;
    let items = stored
        .into_iter()
        .map(|backup| {
            let (kind, course_id) = BackupKindCode::split(backup.name.kind);
            BackupResponse {
                file_name: backup.name.to_string(),
                taken_at: local_text(backup.name.taken_at),
                kind,
                course_id,
                course_name: course_id.and_then(|id| names.get(&id).cloned()),
                size: backup.size,
            }
        })
        .collect();
    let last_failure = backups.last_failure().map(|failure| BackupFailureResponse {
        at: local_text(failure.at),
        kind: BackupKindCode::split(failure.kind).0,
    });
    let last_restore = backups.restore_outcome().map(|outcome| match outcome {
        RestoreOutcome::Restored { at, .. } => RestoreOutcomeResponse {
            at: local_text(at),
            restored: true,
        },
        RestoreOutcome::Failed { at } => RestoreOutcomeResponse {
            at: local_text(at),
            restored: false,
        },
    });
    let second = backups.second_location().await;
    Ok(Json(BackupListResponse {
        items,
        last_failure,
        last_restore,
        second_location: second_location_response(second, client),
    }))
}

/// 지금 백업. 성공하면 201과 만든 백업.
async fn create_backup(
    State(backups): State<Backups>,
    staff: CurrentStaff,
) -> Result<(StatusCode, Json<BackupResponse>), ApiError> {
    staff.require(Action::ManageBackups)?;
    let name = backups
        .make_now(OffsetDateTime::now_utc())
        .await
        .map_err(|error| {
            tracing::warn!(%error, staff_id = staff.id, "지금 백업 실패");
            ApiError::BackupFailed
        })?;
    let size = backups
        .list()
        .await
        .map_err(ApiError::internal)?
        .into_iter()
        .find(|backup| backup.name == name)
        .map_or(0, |backup| backup.size);
    tracing::info!(staff_id = staff.id, "지금 백업");
    Ok((
        StatusCode::CREATED,
        Json(BackupResponse {
            file_name: name.to_string(),
            taken_at: local_text(name.taken_at),
            kind: BackupKindCode::Manual,
            course_id: None,
            course_name: None,
            size,
        }),
    ))
}

/// 두 번째 위치를 바꾸거나 끈다. 호스트 PC에서만. 바뀐 두 번째 위치를 돌려준다.
async fn set_second_location(
    State(backups): State<Backups>,
    staff: CurrentStaff,
    client: Client,
    body: Result<Json<SecondLocationRequest>, JsonRejection>,
) -> Result<Json<SecondLocationResponse>, ApiError> {
    staff.require(Action::ManageBackups)?;
    client.require_host()?;
    let Json(request) = body?;
    let path = request
        .path
        .map(|path| path.trim().to_owned())
        .filter(|path| !path.is_empty())
        .map(PathBuf::from);
    let turned_on = path.is_some();
    backups
        .set_second_location(path)
        .await
        .map_err(|error| match error {
            SecondLocationError::Save(error) => ApiError::internal(error),
            invalid => ApiError::InvalidInput(vec![FieldError {
                field: "path".to_owned(),
                message: invalid.to_string(),
            }]),
        })?;
    // 경로는 직원 이름 같은 폴더 이름이 있을 수 있어 로그에 남기지 않는다
    tracing::info!(staff_id = staff.id, turned_on, "두 번째 위치 바꿈");
    let second = backups.second_location().await;
    Ok(Json(second_location_response(second, client)))
}

/// 복원할 백업을 살펴보고 미리보기를 돌려준다. 호스트 PC에서만.
async fn preview_restore(
    State(backups): State<Backups>,
    staff: CurrentStaff,
    client: Client,
    body: Result<Json<RestoreRequest>, JsonRejection>,
) -> Result<Json<RestorePreviewResponse>, ApiError> {
    staff.require(Action::ManageBackups)?;
    client.require_host()?;
    let Json(request) = body?;
    let source = RestoreSource::resolve(backups.dir(), request.file_name, request.path)
        .await
        .map_err(source_error)?;
    let preview = inspect::inspect(&source.file)
        .await
        .map_err(|error| unusable(source.field, &error))?;
    Ok(Json(RestorePreviewResponse {
        taken_at: source.name.map(|name| local_text(name.taken_at)),
        kind: source.name.map(|name| BackupKindCode::split(name.kind).0),
        term_names: preview.term_names,
        member_count: preview.member_count,
        application_count: preview.application_count,
    }))
}

/// 복원을 준비한다. 성공하면 204이고, 화면이 앱을 다시 켜면 바뀐다. 호스트 PC에서만.
async fn restore(
    State(backups): State<Backups>,
    staff: CurrentStaff,
    client: Client,
    body: Result<Json<RestoreRequest>, JsonRejection>,
) -> Result<StatusCode, ApiError> {
    staff.require(Action::ManageBackups)?;
    client.require_host()?;
    let Json(request) = body?;
    let source = RestoreSource::resolve(backups.dir(), request.file_name, request.path)
        .await
        .map_err(source_error)?;
    backups
        .prepare_restore(&source.file, staff.id, OffsetDateTime::now_utc())
        .await
        .map_err(|error| match error {
            PrepareError::Inspect(error) => unusable(source.field, &error),
            PrepareError::Backup(error) => {
                tracing::warn!(%error, staff_id = staff.id, "복원 직전 백업 실패");
                ApiError::BackupFailed
            }
            PrepareError::Io(error) => ApiError::internal(error),
        })?;
    Ok(StatusCode::NO_CONTENT)
}

/// 고르지 못한 복원 파일을 API 오류로.
fn source_error(error: SourceError) -> ApiError {
    match error.field() {
        Some(field) => field_error(field, error.to_string()),
        None => ApiError::MalformedRequest,
    }
}

/// 복원할 수 없는 파일을 API 오류로. `field`는 요청 칸.
fn unusable(field: &str, error: &InspectError) -> ApiError {
    if let InspectError::Broken(source) = error {
        // 없는 파일도 여기로 온다. 경로는 남기지 않는다
        tracing::info!(error = %source, "복원할 수 없는 파일");
    }
    field_error(field, error.to_string())
}

fn second_location_response(
    status: SecondLocationStatus,
    client: Client,
) -> SecondLocationResponse {
    SecondLocationResponse {
        path: status.path.map(|path| path.display().to_string()),
        reachable: status.reachable,
        last_copied_at: status.last_copied_at.map(local_text),
        changeable: client.on_host,
    }
}
