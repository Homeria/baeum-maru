//! 화면 확인용 더미 데이터. 개발용 `dev-seed` 기능을 켰을 때만 빌드된다.
//!
//! 실제 데이터와 섞이지 않도록 빈 폴더에만 넣는다. 모두 지어낸 데이터다.

mod basics;

use std::fmt::Debug;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use time::OffsetDateTime;

use crate::clock::local_date;
use crate::db::{self, DbError};

/// 더미 데이터를 넣지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum SeedError {
    #[error(
        "{0} 폴더가 비어 있지 않습니다. 더미 데이터는 빈 폴더에만 넣습니다 (just seed-clear로 지울 수 있음)"
    )]
    NotEmpty(PathBuf),
    #[error("폴더를 확인하지 못했습니다: {0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Open(#[from] DbError),
    /// 더미 데이터가 확인을 통과하지 못했거나 저장하지 못했다. 이 파일의 데이터를 고친다.
    #[error("{what} 더미 데이터를 넣지 못했습니다: {detail}")]
    Dummy { what: &'static str, detail: String },
}

/// 다 넣은 뒤 알려 줄 것.
pub struct SeedReport {
    /// 관리자 복구 코드. 처음 설정처럼 이때만 알 수 있다.
    pub recovery_code: String,
}

/// 어떤 오류든 `what`을 붙인 [`SeedError::Dummy`]로 바꾸는 함수를 만든다. `map_err`에 넘긴다.
fn fail<E: Debug>(what: &'static str) -> impl FnOnce(E) -> SeedError {
    move |error| SeedError::Dummy {
        what,
        detail: format!("{error:?}"),
    }
}

/// `dir`에 새 DB를 만들고 더미 데이터를 넣는다.
///
/// # Errors
///
/// `dir`에 파일이 하나라도 있으면 아무것도 하지 않고 [`SeedError::NotEmpty`]를 돌려준다.
pub async fn run(dir: &Path) -> Result<SeedReport, SeedError> {
    ensure_empty(dir)?;
    let pool = db::open(dir).await?;
    let now = OffsetDateTime::now_utc();
    let basics = basics::insert(&pool, local_date(now), now).await?;
    pool.close().await;
    Ok(SeedReport {
        recovery_code: basics.recovery_code,
    })
}

/// 폴더가 없거나 비어 있어야 통과한다.
fn ensure_empty(dir: &Path) -> Result<(), SeedError> {
    match std::fs::read_dir(dir) {
        Ok(mut entries) => match entries.next() {
            None => Ok(()),
            Some(_) => Err(SeedError::NotEmpty(dir.to_path_buf())),
        },
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fills_folder_that_does_not_exist_yet() {
        let parent = tempfile::tempdir().unwrap();
        let dir = parent.path().join("data");

        run(&dir).await.unwrap();

        assert!(dir.join("baeum-maru.db").is_file());
    }

    #[tokio::test]
    async fn fills_empty_folder() {
        let dir = tempfile::tempdir().unwrap();

        run(dir.path()).await.unwrap();

        assert!(dir.path().join("baeum-maru.db").is_file());
    }

    /// sqlx는 SQL을 고정된 글자로만 받는다 (값을 끼워 넣다 생기는 실수를 막음)
    async fn count(pool: &sqlx::SqlitePool, sql: &'static str) -> i64 {
        sqlx::query_scalar(sql).fetch_one(pool).await.unwrap()
    }

    #[tokio::test]
    async fn fills_staff_term_rooms_subjects_and_courses() {
        let dir = tempfile::tempdir().unwrap();

        let report = run(dir.path()).await.unwrap();

        let pool = db::open(dir.path()).await.unwrap();
        assert_eq!(count(&pool, "SELECT COUNT(*) FROM staff").await, 4);
        assert_eq!(count(&pool, "SELECT COUNT(*) FROM terms").await, 1);
        assert_eq!(count(&pool, "SELECT COUNT(*) FROM rooms").await, 9);
        assert_eq!(count(&pool, "SELECT COUNT(*) FROM subjects").await, 9);
        assert_eq!(count(&pool, "SELECT COUNT(*) FROM courses").await, 12);
        assert_eq!(
            count(&pool, "SELECT COUNT(*) FROM course_schedules").await,
            15
        );
        assert!(!report.recovery_code.is_empty());
    }

    #[tokio::test]
    async fn leaves_folder_with_files_untouched() {
        let dir = tempfile::tempdir().unwrap();
        let existing = dir.path().join("baeum-maru.db");
        std::fs::write(&existing, b"real data").unwrap();

        let result = run(dir.path()).await;

        assert!(matches!(result, Err(SeedError::NotEmpty(_))));
        assert_eq!(std::fs::read(&existing).unwrap(), b"real data");
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }
}
