//! 백업 파일 살펴보기(ADR 0011). 복원하기 전에 쓸 수 있는 파일인지 보고 미리보기 값을 센다.
//!
//! 파일은 읽기 전용으로 열고 고치지 않는다. 미리보기에는 회원 이름 같은 개인정보를 담지 않는다.

use std::path::Path;

use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{ConnectOptions, Connection, SqliteConnection};

use crate::infra::db;

/// 복원 확인 창에 보여 줄 것.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupPreview {
    /// 시작일 순서.
    pub term_names: Vec<String>,
    pub member_count: i64,
    pub application_count: i64,
}

/// 복원할 수 없는 파일이다. 경로는 담지 않는다.
#[derive(Debug, thiserror::Error)]
pub enum InspectError {
    #[error("백업 파일을 열지 못했습니다. 파일이 깨졌거나 DB 파일이 아닙니다")]
    Broken(#[source] sqlx::Error),
    #[error("백업 파일이 깨졌습니다. 다른 백업을 골라 주세요")]
    Damaged,
    #[error("배움마루 백업 파일이 아닙니다")]
    NotBaeumMaru,
    #[error("더 새 버전의 배움마루에서 만든 백업입니다. 앱을 새 버전으로 바꾼 뒤 복원해 주세요")]
    NewerVersion,
}

/// `file`이 복원할 수 있는 백업인지 보고 미리보기 값을 센다.
pub async fn inspect(file: &Path) -> Result<BackupPreview, InspectError> {
    // immutable: 다른 프로그램이 쓰지 않는 파일로 보고 잠금이나 보조 파일(-wal, -shm)을 만들지 않는다
    let mut conn = SqliteConnectOptions::new()
        .filename(file)
        .read_only(true)
        .immutable(true)
        .connect()
        .await
        .map_err(InspectError::Broken)?;
    let result = read(&mut conn).await;
    // 닫다 실패해도 읽은 결과가 중요하다
    let _ = conn.close().await;
    result
}

async fn read(conn: &mut SqliteConnection) -> Result<BackupPreview, InspectError> {
    let check: String = sqlx::query_scalar("PRAGMA quick_check")
        .fetch_one(&mut *conn)
        .await
        .map_err(InspectError::Broken)?;
    if check != "ok" {
        return Err(InspectError::Damaged);
    }
    let versions: Vec<i64> =
        sqlx::query_scalar("SELECT version FROM _sqlx_migrations WHERE success = 1")
            .fetch_all(&mut *conn)
            .await
            .map_err(|_| InspectError::NotBaeumMaru)?;
    if versions.is_empty() {
        return Err(InspectError::NotBaeumMaru);
    }
    if !versions.into_iter().all(db::is_known_migration) {
        return Err(InspectError::NewerVersion);
    }
    let term_names: Vec<String> =
        sqlx::query_scalar("SELECT name FROM terms ORDER BY starts_on, id")
            .fetch_all(&mut *conn)
            .await
            .map_err(|_| InspectError::NotBaeumMaru)?;
    let member_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM members")
        .fetch_one(&mut *conn)
        .await
        .map_err(|_| InspectError::NotBaeumMaru)?;
    let application_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM applications")
        .fetch_one(&mut *conn)
        .await
        .map_err(|_| InspectError::NotBaeumMaru)?;
    Ok(BackupPreview {
        term_names,
        member_count,
        application_count,
    })
}
