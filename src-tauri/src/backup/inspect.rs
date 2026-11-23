//! 백업 파일 살펴보기(ADR 0011). 복원하기 전에 쓸 수 있는 파일인지 보고 미리보기 값을 센다.
//!
//! 파일은 읽기 전용으로 열고 고치지 않는다. 미리보기에는 회원 이름 같은 개인정보를 담지 않는다.

use std::path::Path;

use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{ConnectOptions, Connection, SqliteConnection};

use crate::db;

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

#[cfg(test)]
mod tests {
    use time::macros::datetime;

    use super::*;
    use crate::backup::name::BackupKind;
    use crate::backup::store;
    use crate::db::open_temp;
    use crate::db::test_support::{insert_member, insert_term};

    async fn backup_of(pool: &sqlx::SqlitePool, dir: &Path) -> std::path::PathBuf {
        let name = store::create(pool, dir, BackupKind::Manual, datetime!(2026-11-23 09:00))
            .await
            .unwrap();
        dir.join(name.to_string())
    }

    #[tokio::test]
    async fn counts_terms_members_and_applications() {
        let (dir, pool) = open_temp().await;
        insert_term(&pool, "2027년 1학기", "2027-03-01", "2027-06-30")
            .await
            .unwrap();
        insert_term(&pool, "2026년 2학기", "2026-09-01", "2026-12-31")
            .await
            .unwrap();
        insert_member(&pool, "00001").await.unwrap();
        insert_member(&pool, "00002").await.unwrap();
        let file = backup_of(&pool, &dir.path().join("backups")).await;

        let preview = inspect(&file).await.unwrap();

        assert_eq!(
            preview,
            BackupPreview {
                term_names: vec!["2026년 2학기".to_owned(), "2027년 1학기".to_owned()],
                member_count: 2,
                application_count: 0,
            }
        );
        // 살펴보기만 하고 보조 파일을 남기지 않는다
        let mut entries = tokio::fs::read_dir(dir.path().join("backups"))
            .await
            .unwrap();
        let mut count = 0;
        while entries.next_entry().await.unwrap().is_some() {
            count += 1;
        }
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn refuses_files_that_are_not_baeum_maru_backups() {
        let dir = tempfile::tempdir().unwrap();
        let text = dir.path().join("메모.db");
        tokio::fs::write(&text, "SQLite가 아닌 글자 파일입니다. ".repeat(100))
            .await
            .unwrap();
        let other = dir.path().join("다른 프로그램.db");
        let mut conn = SqliteConnectOptions::new()
            .filename(&other)
            .create_if_missing(true)
            .connect()
            .await
            .unwrap();
        sqlx::query("CREATE TABLE notes (body TEXT)")
            .execute(&mut conn)
            .await
            .unwrap();
        conn.close().await.unwrap();

        assert!(matches!(inspect(&text).await, Err(InspectError::Broken(_))));
        assert!(matches!(
            inspect(&other).await,
            Err(InspectError::NotBaeumMaru)
        ));
        assert!(matches!(
            inspect(&dir.path().join("없는 파일.db")).await,
            Err(InspectError::Broken(_))
        ));
    }

    #[tokio::test]
    async fn refuses_backups_from_newer_app() {
        let (dir, pool) = open_temp().await;
        // 이 앱이 모르는 구조 변경이 적용된 DB
        sqlx::query(
            "INSERT INTO _sqlx_migrations (version, description, installed_on, success, checksum, execution_time)
             VALUES (99991231000000, 'from newer app', '2099-01-01', 1, x'00', 0)",
        )
        .execute(&pool)
        .await
        .unwrap();
        let file = backup_of(&pool, &dir.path().join("backups")).await;

        assert!(matches!(
            inspect(&file).await,
            Err(InspectError::NewerVersion)
        ));
    }
}
