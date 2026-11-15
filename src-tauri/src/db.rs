//! SQLite DB 열기와 구조 변경(마이그레이션).
//!
//! ADR 0005: SQLite + sqlx. 구조 변경은 시작할 때 자동으로 적용한다.

use std::path::Path;
use std::time::Duration;

use sqlx::SqlitePool;
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};

/// 데이터 폴더 안의 DB 파일 이름.
const FILE_NAME: &str = "baeum-maru.db";

/// `src-tauri/migrations/`의 구조 변경 파일. 빌드할 때 실행 파일에 들어간다.
static MIGRATOR: Migrator = sqlx::migrate!();

/// DB를 열지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("데이터 폴더를 만들지 못했습니다: {0}")]
    CreateDir(#[source] std::io::Error),
    #[error("DB를 열지 못했습니다: {0}")]
    Open(#[from] sqlx::Error),
    #[error("DB 구조 변경을 적용하지 못했습니다: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
}

/// `dir` 안의 DB를 열고(없으면 만들고) 구조 변경을 적용한다.
pub async fn open(dir: &Path) -> Result<SqlitePool, DbError> {
    tokio::fs::create_dir_all(dir)
        .await
        .map_err(DbError::CreateDir)?;

    // [ADR 0005]: 실험 01의 설정. 정전 때 마지막 저장을 잃지 않도록 synchronous는 FULL
    let options = SqliteConnectOptions::new()
        .filename(dir.join(FILE_NAME))
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Full)
        .busy_timeout(Duration::from_secs(5))
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(options)
        .await?;
    MIGRATOR.run(&pool).await?;
    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn text(pool: &SqlitePool, sql: &'static str) -> String {
        sqlx::query_scalar(sql).fetch_one(pool).await.unwrap()
    }

    #[tokio::test]
    async fn creates_db_file_in_new_folder() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("data");

        open(&dir).await.unwrap();

        assert!(dir.join(FILE_NAME).is_file());
    }

    #[tokio::test]
    async fn applies_settings() {
        let dir = tempfile::tempdir().unwrap();
        let pool = open(dir.path()).await.unwrap();

        let journal_mode = "SELECT journal_mode FROM pragma_journal_mode";
        assert_eq!(text(&pool, journal_mode).await, "wal");
        // 2 = FULL
        let synchronous = "SELECT CAST(synchronous AS TEXT) FROM pragma_synchronous";
        assert_eq!(text(&pool, synchronous).await, "2");
        let foreign_keys = "SELECT CAST(foreign_keys AS TEXT) FROM pragma_foreign_keys";
        assert_eq!(text(&pool, foreign_keys).await, "1");
        let busy_timeout = "SELECT CAST(timeout AS TEXT) FROM pragma_busy_timeout";
        assert_eq!(text(&pool, busy_timeout).await, "5000");
    }

    #[tokio::test]
    async fn opens_existing_db_again() {
        let dir = tempfile::tempdir().unwrap();
        open(dir.path()).await.unwrap().close().await;

        open(dir.path()).await.unwrap();
    }
}
