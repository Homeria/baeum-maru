//! SQLite DB 열기와 구조 변경(마이그레이션).
//!
//! ADR 0005: SQLite + sqlx. 구조 변경은 시작할 때 자동으로 적용한다.

pub mod members;
pub mod rooms;
pub mod subjects;

use std::path::Path;
use std::time::Duration;

use chrono::{DateTime, SecondsFormat, Utc};

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

/// DB에 시각을 쓰는 모양. [docs/conventions.md]: UTC, 밀리초까지, 끝에 Z
pub fn timestamp_text(time: DateTime<Utc>) -> String {
    time.to_rfc3339_opts(SecondsFormat::Millis, true)
}

/// UNIQUE 제약을 어겨서 난 오류인지. 테이블마다 어떤 UNIQUE인지는 부르는 쪽이 안다.
fn is_unique_violation(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .is_some_and(|db_error| db_error.is_unique_violation())
}

/// 테스트용: 임시 폴더에 새 DB를 연다. 폴더는 돌려준 `TempDir`가 사라질 때 지워진다.
#[cfg(test)]
pub(crate) async fn open_temp() -> (tempfile::TempDir, SqlitePool) {
    let dir = tempfile::tempdir().unwrap();
    let pool = open(dir.path()).await.unwrap();
    (dir, pool)
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
        let (_dir, pool) = open_temp().await;

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

    async fn insert_member(pool: &SqlitePool, member_no: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO members (member_no, name, phone_number, created_at, updated_at)
             VALUES (?, '회원1', '01000000001', '2026-11-15T00:00:00.000Z', '2026-11-15T00:00:00.000Z')",
        )
        .bind(member_no)
        .execute(pool)
        .await
        .map(|_| ())
    }

    #[tokio::test]
    async fn br_m2_rejects_duplicate_member_no() {
        let (_dir, pool) = open_temp().await;

        insert_member(&pool, "00001").await.unwrap();
        let duplicate = insert_member(&pool, "00001").await.unwrap_err();

        let is_unique_violation = duplicate
            .as_database_error()
            .is_some_and(|error| error.is_unique_violation());
        assert!(is_unique_violation, "{duplicate}");
    }

    #[tokio::test]
    async fn br_m2_allows_different_member_no() {
        let (_dir, pool) = open_temp().await;

        insert_member(&pool, "00001").await.unwrap();
        insert_member(&pool, "00002").await.unwrap();
    }

    #[tokio::test]
    async fn members_reject_unknown_gender() {
        let (_dir, pool) = open_temp().await;

        let result = sqlx::query(
            "INSERT INTO members (member_no, name, phone_number, gender, created_at, updated_at)
             VALUES ('00001', '회원1', '01000000001', 'other', '2026-11-15T00:00:00.000Z', '2026-11-15T00:00:00.000Z')",
        )
        .execute(&pool)
        .await;

        assert!(result.is_err());
    }

    async fn insert_room(
        pool: &SqlitePool,
        building: Option<&str>,
        floor: Option<i64>,
        name: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO rooms (building, floor, name, created_at, updated_at)
             VALUES (?, ?, ?, '2026-11-15T00:00:00.000Z', '2026-11-15T00:00:00.000Z')",
        )
        .bind(building)
        .bind(floor)
        .bind(name)
        .execute(pool)
        .await
        .map(|_| ())
    }

    #[tokio::test]
    async fn br_r1_rejects_same_name_in_same_building() {
        let (_dir, pool) = open_temp().await;

        insert_room(&pool, Some("본관"), Some(3), "다목적실")
            .await
            .unwrap();
        // 층이 달라도 겹친 것으로 봄
        let duplicate = insert_room(&pool, Some("본관"), Some(2), "다목적실")
            .await
            .unwrap_err();

        let is_unique_violation = duplicate
            .as_database_error()
            .is_some_and(|error| error.is_unique_violation());
        assert!(is_unique_violation, "{duplicate}");
    }

    #[tokio::test]
    async fn br_r1_allows_same_name_in_other_building() {
        let (_dir, pool) = open_temp().await;

        insert_room(&pool, Some("본관"), None, "다목적실")
            .await
            .unwrap();
        insert_room(&pool, Some("별관"), None, "다목적실")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn br_r1_rooms_require_building() {
        let (_dir, pool) = open_temp().await;

        assert!(insert_room(&pool, None, None, "다목적실").await.is_err());
    }

    #[tokio::test]
    async fn rooms_accept_floors_from_basement_2_to_5() {
        let (_dir, pool) = open_temp().await;

        for (i, floor) in [None, Some(-2), Some(-1), Some(1), Some(5)]
            .into_iter()
            .enumerate()
        {
            let name = format!("강의실{i}");
            insert_room(&pool, Some("본관"), floor, &name)
                .await
                .unwrap();
        }
    }

    #[tokio::test]
    async fn rooms_reject_floor_zero_and_out_of_range() {
        let (_dir, pool) = open_temp().await;

        for floor in [0, -3, 6] {
            let result = insert_room(&pool, Some("본관"), Some(floor), "다목적실").await;
            assert!(result.is_err(), "{floor}");
        }
    }

    async fn insert_subject(
        pool: &SqlitePool,
        name: &str,
        name_key: &str,
        category: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO subjects (name, name_key, category, created_at, updated_at)
             VALUES (?, ?, ?, '2026-11-15T00:00:00.000Z', '2026-11-15T00:00:00.000Z')",
        )
        .bind(name)
        .bind(name_key)
        .bind(category)
        .execute(pool)
        .await
        .map(|_| ())
    }

    #[tokio::test]
    async fn br_j1_rejects_same_name_key() {
        let (_dir, pool) = open_temp().await;

        insert_subject(&pool, "한글교실", "한글교실", None)
            .await
            .unwrap();
        // 분류가 달라도 겹친 것으로 봄
        let duplicate = insert_subject(&pool, "한글 교실", "한글교실", Some("평생교육"))
            .await
            .unwrap_err();

        assert!(is_unique_violation(&duplicate), "{duplicate}");
    }

    #[tokio::test]
    async fn br_j1_allows_different_names() {
        let (_dir, pool) = open_temp().await;

        insert_subject(&pool, "스마트폰", "스마트폰", None)
            .await
            .unwrap();
        insert_subject(&pool, "스마트폰 활용", "스마트폰활용", None)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn subjects_keep_no_category_as_null_not_empty_text() {
        let (_dir, pool) = open_temp().await;

        insert_subject(&pool, "요가", "요가", None).await.unwrap();
        assert!(
            insert_subject(&pool, "댄스", "댄스", Some(""))
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn subjects_reject_empty_name_key() {
        let (_dir, pool) = open_temp().await;

        assert!(insert_subject(&pool, " ", "", None).await.is_err());
    }
}
