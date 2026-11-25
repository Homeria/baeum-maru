//! `/api/backups`: 백업 목록, 지금 백업, 두 번째 위치 바꾸기(ADR 0011). 관리자만 한다.
//!
//! 두 번째 위치는 그 PC의 폴더라서 호스트 PC에서만 바꾼다. 다른 PC에서는 보기만 한다.
//! 복원도 호스트 PC에서만 한다. 호스트 앱을 다시 켜야 하고, 고른 파일도 호스트의 경로다.

mod json;

/// 깨진 DB 복원(`broken_db`)도 같은 모양으로 답한다.
pub use json::{BackupKindCode, RestorePreviewResponse, RestoreRequest, local_text};

use std::path::PathBuf;

use axum::Json;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use baeum_domain::Action;
use sqlx::SqlitePool;
use time::OffsetDateTime;

use crate::backup::inspect::{self, InspectError};
use crate::backup::restore::{PrepareError, RestoreOutcome};
use crate::backup::source::{RestoreSource, SourceError};
use crate::backup::{Backups, SecondLocationError, SecondLocationStatus};
use crate::db::courses;
use crate::http::auth::{Client, CurrentStaff};
use crate::http::error::{ApiError, FieldError};

use json::{
    BackupFailureResponse, BackupListResponse, BackupResponse, RestoreOutcomeResponse,
    SecondLocationRequest, SecondLocationResponse,
};

/// 백업 목록, 마지막 자동 백업 실패, 두 번째 위치.
pub async fn list_backups(
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
pub async fn create_backup(
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
pub async fn set_second_location(
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
pub async fn preview_restore(
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
pub async fn restore(
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
        Some(field) => field_error(field, &error.to_string()),
        None => ApiError::MalformedRequest,
    }
}

/// 복원할 수 없는 파일을 API 오류로. `field`는 요청 칸.
fn unusable(field: &str, error: &InspectError) -> ApiError {
    if let InspectError::Broken(source) = error {
        // 없는 파일도 여기로 온다. 경로는 남기지 않는다
        tracing::info!(error = %source, "복원할 수 없는 파일");
    }
    field_error(field, &error.to_string())
}

fn field_error(field: &str, message: &str) -> ApiError {
    ApiError::InvalidInput(vec![FieldError {
        field: field.to_owned(),
        message: message.to_owned(),
    }])
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

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use serde_json::{Value, json};

    use time::macros::datetime;

    use super::super::test_support::{app, app_as, from_approved_pc, get, json_body, post, put};
    use crate::backup::Backups;
    use crate::db::test_support::{insert_course, insert_subject, insert_term};
    use crate::infra::backup_file::name::BackupKind;

    #[tokio::test]
    async fn only_admin_manages_backups() {
        for role in ["staff", "temp_staff", "read_only"] {
            let (_dir, _db, router) = app_as(role).await;

            let list = get(router.clone(), "/api/backups").await;
            let create = post(router, "/api/backups", "{}").await;

            assert_eq!(list.status(), StatusCode::FORBIDDEN, "{role}");
            assert_eq!(create.status(), StatusCode::FORBIDDEN, "{role}");
        }
    }

    #[tokio::test]
    async fn backup_now_shows_in_the_list() {
        let (_dir, _db, router) = app().await;

        let created = post(router.clone(), "/api/backups", "{}").await;
        assert_eq!(created.status(), StatusCode::CREATED);
        let created = json_body(created).await;
        let list = json_body(get(router, "/api/backups").await).await;

        assert_eq!(created["kind"], "manual");
        assert!(created["size"].as_u64().unwrap() > 0);
        assert_eq!(list["items"][0]["fileName"], created["fileName"]);
        assert_eq!(list["lastFailure"], Value::Null);
        assert_eq!(
            list["secondLocation"],
            json!({ "path": null, "reachable": false, "lastCopiedAt": null, "changeable": true })
        );
    }

    #[tokio::test]
    async fn lottery_backup_shows_course_name() {
        let (dir, db, router) = app().await;
        let term_id = insert_term(&db, "2027년 1학기", "2027-03-01", "2027-06-30")
            .await
            .unwrap();
        let subject_id = insert_subject(&db, "요가", "요가", None).await.unwrap();
        let course_id = insert_course(&db, term_id, subject_id, Some("초급"), None)
            .await
            .unwrap();
        Backups::new(db, dir.path())
            .make(
                BackupKind::BeforeLottery { course_id },
                datetime!(2026-11-23 01:00 UTC),
            )
            .await
            .unwrap();

        let list = json_body(get(router, "/api/backups").await).await;

        assert_eq!(list["items"][0]["kind"], "before_lottery");
        assert_eq!(list["items"][0]["courseId"], course_id);
        assert_eq!(list["items"][0]["courseName"], "요가 초급");
    }

    #[tokio::test]
    async fn sets_and_clears_second_location_on_host() {
        let (_dir, _db, router) = app().await;
        let second = tempfile::tempdir().unwrap();
        let path = second.path().display().to_string();

        let set = put(
            router.clone(),
            "/api/backups/second-location",
            &json!({ "path": path }).to_string(),
        )
        .await;
        assert_eq!(set.status(), StatusCode::OK);
        assert_eq!(json_body(set).await["path"], path.as_str());

        let cleared = put(
            router.clone(),
            "/api/backups/second-location",
            &json!({ "path": null }).to_string(),
        )
        .await;
        assert_eq!(json_body(cleared).await["path"], Value::Null);
    }

    #[tokio::test]
    async fn missing_folder_is_invalid_input() {
        let (dir, _db, router) = app().await;
        let missing = dir.path().join("없는 폴더").display().to_string();

        let response = put(
            router,
            "/api/backups/second-location",
            &json!({ "path": missing }).to_string(),
        )
        .await;

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = json_body(response).await;
        assert_eq!(body["fields"][0]["field"], "path");
    }

    #[tokio::test]
    async fn other_pc_sees_but_cannot_change_second_location() {
        let (_dir, db, router) = app().await;
        let router = from_approved_pc(&db, router).await;
        let second = tempfile::tempdir().unwrap();

        let list = json_body(get(router.clone(), "/api/backups").await).await;
        let response = put(
            router,
            "/api/backups/second-location",
            &json!({ "path": second.path().display().to_string() }).to_string(),
        )
        .await;

        assert_eq!(list["secondLocation"]["changeable"], false);
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        assert_eq!(json_body(response).await["code"], "HOST_ONLY");
    }

    async fn first_backup(router: axum::Router) -> String {
        let created = json_body(post(router, "/api/backups", "{}").await).await;
        created["fileName"].as_str().unwrap().to_owned()
    }

    #[tokio::test]
    async fn previews_backup_from_list_and_from_other_place() {
        let (dir, db, router) = app().await;
        insert_term(&db, "2027년 1학기", "2027-03-01", "2027-06-30")
            .await
            .unwrap();
        let file_name = first_backup(router.clone()).await;
        let elsewhere = tempfile::tempdir().unwrap();
        let copied = elsewhere.path().join("옛 백업.db");
        tokio::fs::copy(
            crate::infra::backup_file::dir(dir.path()).join(&file_name),
            &copied,
        )
        .await
        .unwrap();

        let from_list = post(
            router.clone(),
            "/api/backups/restore-preview",
            &json!({ "fileName": file_name }).to_string(),
        )
        .await;
        let from_path = post(
            router,
            "/api/backups/restore-preview",
            &json!({ "path": copied.display().to_string() }).to_string(),
        )
        .await;

        assert_eq!(from_list.status(), StatusCode::OK);
        let from_list = json_body(from_list).await;
        assert_eq!(from_list["kind"], "manual");
        assert_eq!(from_list["termNames"], json!(["2027년 1학기"]));
        assert_eq!(from_list["memberCount"], 0);
        assert_eq!(from_list["applicationCount"], 0);
        let from_path = json_body(from_path).await;
        // 이름을 바꾼 파일은 시각을 알 수 없다
        assert_eq!(from_path["takenAt"], Value::Null);
        assert_eq!(from_path["termNames"], json!(["2027년 1학기"]));
    }

    #[tokio::test]
    async fn restore_is_prepared_for_next_start() {
        let (dir, _db, router) = app().await;
        let file_name = first_backup(router.clone()).await;

        let response = post(
            router.clone(),
            "/api/backups/restore",
            &json!({ "fileName": file_name }).to_string(),
        )
        .await;

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert!(
            tokio::fs::try_exists(dir.path().join("restore-pending.db"))
                .await
                .unwrap()
        );
        let list = json_body(get(router, "/api/backups").await).await;
        assert_eq!(list["items"][0]["kind"], "before_restore");
        assert_eq!(list["lastRestore"], Value::Null);
    }

    #[tokio::test]
    async fn unusable_sources_are_invalid_input() {
        let (dir, _db, router) = app().await;
        let text = dir.path().join("메모.db");
        tokio::fs::write(&text, b"not a database").await.unwrap();
        let cases = [
            (
                json!({ "fileName": "../baeum-maru.db" }),
                "fileName",
                "백업 목록에 없는 파일",
            ),
            (
                json!({ "fileName": "20261123-090000_daily.db" }),
                "fileName",
                "백업 파일이 없습니다",
            ),
            (json!({ "path": "백업.db" }), "path", "드라이브부터"),
            (
                json!({ "path": text.display().to_string() }),
                "path",
                "DB 파일이 아닙니다",
            ),
        ];
        for (body, field, message) in cases {
            let response = post(router.clone(), "/api/backups/restore", &body.to_string()).await;

            assert_eq!(
                response.status(),
                StatusCode::UNPROCESSABLE_ENTITY,
                "{body}"
            );
            let error = json_body(response).await;
            assert_eq!(error["fields"][0]["field"], field, "{body}");
            assert!(
                error["fields"][0]["message"]
                    .as_str()
                    .unwrap()
                    .contains(message),
                "{body}: {error}"
            );
        }
        let both = post(
            router,
            "/api/backups/restore",
            &json!({ "fileName": "a", "path": "b" }).to_string(),
        )
        .await;
        assert_eq!(both.status(), StatusCode::BAD_REQUEST);
        assert!(
            !tokio::fs::try_exists(dir.path().join("restore-pending.db"))
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn only_admin_on_host_restores() {
        let (_dir, _db, router) = app_as("staff").await;
        let staff = post(router, "/api/backups/restore", r#"{"fileName":"x"}"#).await;
        assert_eq!(staff.status(), StatusCode::FORBIDDEN);

        let (_dir, db, router) = app().await;
        let file_name = first_backup(router.clone()).await;
        let router = from_approved_pc(&db, router).await;
        for uri in ["/api/backups/restore-preview", "/api/backups/restore"] {
            let response = post(
                router.clone(),
                uri,
                &json!({ "fileName": file_name }).to_string(),
            )
            .await;

            assert_eq!(response.status(), StatusCode::FORBIDDEN, "{uri}");
            assert_eq!(json_body(response).await["code"], "HOST_ONLY");
        }
    }
}
