//! 화면 확인용 더미 데이터. 개발용 `dev-seed` 기능을 켰을 때만 빌드된다.
//!
//! 실제 데이터와 섞이지 않도록 빈 폴더에만 넣는다. 모두 지어낸 데이터다.

use std::io::ErrorKind;
use std::path::{Path, PathBuf};

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
}

/// `dir`에 새 DB를 만들고 더미 데이터를 넣는다.
///
/// # Errors
///
/// `dir`에 파일이 하나라도 있으면 아무것도 하지 않고 [`SeedError::NotEmpty`]를 돌려준다.
pub async fn run(dir: &Path) -> Result<(), SeedError> {
    ensure_empty(dir)?;
    let pool = db::open(dir).await?;
    pool.close().await;
    Ok(())
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
