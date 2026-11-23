//! `/api/backups`: 백업 목록, 지금 백업, 두 번째 위치 바꾸기(ADR 0011). 관리자만 한다.
//!
//! 두 번째 위치는 그 PC의 폴더라서 호스트 PC에서만 바꾼다. 다른 PC에서는 보기만 한다.

mod json;

use std::path::PathBuf;

use axum::Json;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use baeum_domain::Action;
use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::auth::{Client, CurrentStaff};
use super::error::{ApiError, FieldError};
use crate::backup::{Backups, SecondLocationError, SecondLocationStatus};
use crate::db::courses;

use json::{
    BackupFailureResponse, BackupKindCode, BackupListResponse, BackupResponse,
    SecondLocationRequest, SecondLocationResponse, local_text,
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
    let second = backups.second_location().await;
    Ok(Json(BackupListResponse {
        items,
        last_failure,
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

    use super::super::test_support::{app, app_as, from_other_pc, get, json_body, post, put};
    use crate::backup::Backups;
    use crate::backup::name::BackupKind;
    use crate::db::test_support::{insert_course, insert_subject, insert_term};

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
        let (_dir, _db, router) = app().await;
        let router = from_other_pc(router);
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
}
