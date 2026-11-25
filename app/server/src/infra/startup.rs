//! 켤 때 DB 상태 가리기(ADR 0011 DB 손상 시 복원).
//!
//! DB가 깨졌으면 서버를 띄우지 않고 복원 화면을 보여 준다. 더 새 버전 앱의 DB나 데이터 폴더
//! 문제는 복원해도 소용없어서 따로 가린다.

use std::path::Path;

use sqlx::SqlitePool;
use sqlx::migrate::MigrateError;

use crate::infra::db::{self, DbError};

/// DB를 쓸 수 없는 까닭. 화면이 무엇을 보여 줄지 정한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbProblem {
    /// 열지 못했거나, 구조 변경을 적용하지 못했거나, 빠른 검사가 정상이 아니다. 복원 화면을 띄운다.
    Broken,
    /// 이 앱이 모르는 구조 변경이 이미 적용된 DB다. 복원하면 최근 데이터를 잃으니 업데이트만 안내한다.
    NewerVersion,
    /// 데이터 폴더를 만들거나 읽지 못했다. 복원해도 소용없다.
    Unavailable,
}

/// DB를 열고 구조 변경을 적용한 뒤 빠른 검사까지 한다. 쓸 수 없으면 그 까닭을 돌려준다.
pub async fn open_checked(data_dir: &Path) -> Result<SqlitePool, DbProblem> {
    let pool = db::open(data_dir).await.map_err(|error| {
        let problem = classify(&error);
        // 경로는 담지 않는다. DbError 문장에도 경로가 없다
        tracing::error!(%error, ?problem, "DB를 열지 못했습니다");
        problem
    })?;
    match quick_check(&pool).await {
        Ok(()) => Ok(pool),
        Err(reason) => {
            tracing::error!(%reason, "DB 빠른 검사에서 문제를 찾았습니다");
            pool.close().await;
            Err(DbProblem::Broken)
        }
    }
}

fn classify(error: &DbError) -> DbProblem {
    match error {
        DbError::CreateDir(_) => DbProblem::Unavailable,
        DbError::Migrate(MigrateError::VersionMissing(_)) => DbProblem::NewerVersion,
        DbError::Open(_) | DbError::Migrate(_) | DbError::BackupBeforeMigration(_) => {
            DbProblem::Broken
        }
    }
}

/// SQLite 빠른 검사. 정상이면 `Ok`, 아니면 첫 줄. 회원 정보가 아니라 쪽(page) 번호 같은 것만 나온다.
async fn quick_check(pool: &SqlitePool) -> Result<(), String> {
    let first: String = sqlx::query_scalar("PRAGMA quick_check")
        .fetch_one(pool)
        .await
        .map_err(|error| error.to_string())?;
    if first == "ok" { Ok(()) } else { Err(first) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn healthy_db_opens() {
        let dir = tempfile::tempdir().unwrap();

        assert!(open_checked(dir.path()).await.is_ok());
    }

    #[tokio::test]
    async fn garbage_file_is_broken() {
        let dir = tempfile::tempdir().unwrap();
        tokio::fs::write(db::file(dir.path()), "망가진 파일 ".repeat(1000))
            .await
            .unwrap();

        assert_eq!(
            open_checked(dir.path()).await.err(),
            Some(DbProblem::Broken)
        );
    }

    #[tokio::test]
    async fn damaged_pages_are_broken() {
        let dir = tempfile::tempdir().unwrap();
        let pool = db::open(dir.path()).await.unwrap();
        // 쪽이 여러 개가 되도록 채운다
        sqlx::query("CREATE TABLE filler (body TEXT)")
            .execute(&pool)
            .await
            .unwrap();
        for _ in 0..200 {
            sqlx::query("INSERT INTO filler VALUES (?)")
                .bind("더미 ".repeat(200))
                .execute(&pool)
                .await
                .unwrap();
        }
        pool.close().await;
        // 머리(첫 쪽)는 두고 뒤쪽 쪽들을 덮어쓴다
        let file = db::file(dir.path());
        let mut bytes = tokio::fs::read(&file).await.unwrap();
        let len = bytes.len();
        for byte in &mut bytes[len / 2..] {
            *byte = 0xA5;
        }
        tokio::fs::write(&file, bytes).await.unwrap();

        assert_eq!(
            open_checked(dir.path()).await.err(),
            Some(DbProblem::Broken)
        );
    }

    #[tokio::test]
    async fn db_from_newer_app_is_told_apart() {
        let dir = tempfile::tempdir().unwrap();
        let pool = db::open(dir.path()).await.unwrap();
        sqlx::query(
            "INSERT INTO _sqlx_migrations (version, description, installed_on, success, checksum, execution_time)
             VALUES (99991231000000, 'from newer app', '2099-01-01', 1, x'00', 0)",
        )
        .execute(&pool)
        .await
        .unwrap();
        pool.close().await;

        assert_eq!(
            open_checked(dir.path()).await.err(),
            Some(DbProblem::NewerVersion)
        );
    }

    #[tokio::test]
    async fn data_folder_that_is_a_file_is_unavailable() {
        let dir = tempfile::tempdir().unwrap();
        let not_a_folder = dir.path().join("data");
        tokio::fs::write(&not_a_folder, b"file").await.unwrap();

        assert_eq!(
            open_checked(&not_a_folder).await.err(),
            Some(DbProblem::Unavailable)
        );
    }
}
