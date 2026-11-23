//! 백업 파일을 만들고, 정리하고, 두 번째 위치로 복사한다(ADR 0011).
//!
//! 파일은 늘 `이름.part`로 다 쓴 뒤 이름을 바꾼다. 중간에 꺼지거나 동기화 폴더(구글 드라이브
//! 등)가 덜 쓴 파일을 올리는 일을 막는다. `.part` 파일은 백업으로 치지 않는다.

use std::path::{Path, PathBuf};

use sqlx::SqlitePool;
use time::PrimitiveDateTime;

use super::name::{BackupKind, BackupName};
use super::retention;

/// 쓰는 중인 파일에 붙이는 꼬리.
const PART: &str = ".part";

/// 백업하지 못했다. 경로는 직원 이름 같은 폴더 이름이 있을 수 있어 담지 않는다.
#[derive(Debug, thiserror::Error)]
pub enum BackupError {
    #[error("백업 파일을 쓰거나 읽지 못했습니다: {0}")]
    Io(#[from] std::io::Error),
    #[error("DB를 복사하지 못했습니다: {0}")]
    Db(#[from] sqlx::Error),
    #[error("백업 폴더 경로에 쓸 수 없는 글자가 있습니다")]
    NotUtf8Path,
}

/// 폴더 안의 백업 하나.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredBackup {
    pub name: BackupName,
    /// 바이트.
    pub size: u64,
}

/// `dir`에 지금 DB의 백업을 만들고 보관 개수를 넘은 백업을 지운다. 만든 백업 이름을 돌려준다.
///
/// 같은 초에 같은 종류를 또 만들면 새로 만들지 않고 이미 있는 것을 돌려준다.
pub async fn create(
    pool: &SqlitePool,
    dir: &Path,
    kind: BackupKind,
    taken_at: PrimitiveDateTime,
) -> Result<BackupName, BackupError> {
    tokio::fs::create_dir_all(dir).await?;
    let name = BackupName { taken_at, kind };
    let target = dir.join(name.to_string());
    if !tokio::fs::try_exists(&target).await? {
        let part = part_path(&target);
        remove_if_exists(&part).await?;
        let part_text = part.to_str().ok_or(BackupError::NotUtf8Path)?;
        // 앱이 켜진 채 쓰는 중인 내용까지 깨끗한 사본 하나로 만든다
        sqlx::query("VACUUM INTO ?")
            .bind(part_text)
            .execute(pool)
            .await?;
        tokio::fs::rename(&part, &target).await?;
    }
    prune(dir).await?;
    Ok(name)
}

/// `file`(백업 파일)을 `dir`(두 번째 위치)에 복사하고 그쪽도 보관 개수를 넘은 백업을 지운다.
#[cfg_attr(not(test), expect(dead_code, reason = "두 번째 위치 설정을 붙이기 전"))]
pub async fn copy_to(file: &Path, name: BackupName, dir: &Path) -> Result<(), BackupError> {
    tokio::fs::create_dir_all(dir).await?;
    let target = dir.join(name.to_string());
    let part = part_path(&target);
    tokio::fs::copy(file, &part).await?;
    tokio::fs::rename(&part, &target).await?;
    prune(dir).await
}

/// `dir` 안의 백업들. 새것부터. 폴더가 없으면 빈 목록.
pub async fn list(dir: &Path) -> Result<Vec<StoredBackup>, BackupError> {
    let mut entries = match tokio::fs::read_dir(dir).await {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.into()),
    };
    let mut backups = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        let Some(name) = entry.file_name().to_str().and_then(BackupName::parse) else {
            continue;
        };
        let metadata = entry.metadata().await?;
        if metadata.is_file() {
            backups.push(StoredBackup {
                name,
                size: metadata.len(),
            });
        }
    }
    backups.sort_by_key(|backup| std::cmp::Reverse(backup.name));
    Ok(backups)
}

/// `dir`에서 보관 개수를 넘은 백업을 지운다.
async fn prune(dir: &Path) -> Result<(), BackupError> {
    let names: Vec<BackupName> = list(dir)
        .await?
        .into_iter()
        .map(|backup| backup.name)
        .collect();
    for name in retention::to_remove(&names) {
        remove_if_exists(&dir.join(name.to_string())).await?;
    }
    Ok(())
}

fn part_path(target: &Path) -> PathBuf {
    let mut part = target.as_os_str().to_owned();
    part.push(PART);
    PathBuf::from(part)
}

async fn remove_if_exists(path: &Path) -> Result<(), BackupError> {
    match tokio::fs::remove_file(path).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

#[cfg(test)]
mod tests {
    use sqlx::sqlite::SqliteConnectOptions;
    use time::Duration;
    use time::macros::datetime;

    use super::*;
    use crate::db::open_temp;
    use crate::db::test_support::insert_member;

    const AT: PrimitiveDateTime = datetime!(2026-11-23 09:00);

    #[tokio::test]
    async fn creates_a_readable_copy_of_the_db() {
        let (dir, pool) = open_temp().await;
        insert_member(&pool, "00001").await.unwrap();
        let backups = dir.path().join("backups");

        let name = create(&pool, &backups, BackupKind::Manual, AT)
            .await
            .unwrap();

        let copy = SqlitePool::connect_with(
            SqliteConnectOptions::new().filename(backups.join(name.to_string())),
        )
        .await
        .unwrap();
        let members: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM members")
            .fetch_one(&copy)
            .await
            .unwrap();
        assert_eq!(members, 1);
        let listed = list(&backups).await.unwrap();
        assert_eq!(listed.len(), 1);
        assert!(listed[0].size > 0);
    }

    #[tokio::test]
    async fn same_second_same_kind_is_made_once() {
        let (dir, pool) = open_temp().await;
        let backups = dir.path().join("backups");

        create(&pool, &backups, BackupKind::Exit, AT).await.unwrap();
        create(&pool, &backups, BackupKind::Exit, AT).await.unwrap();

        assert_eq!(list(&backups).await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn removes_backups_beyond_the_limit_and_ignores_other_files() {
        let (dir, pool) = open_temp().await;
        let backups = dir.path().join("backups");
        tokio::fs::create_dir_all(&backups).await.unwrap();
        // 그때그때 백업 20개가 이미 있다
        for day in 0..20 {
            let old = BackupName {
                taken_at: datetime!(2026-01-01 09:00) + Duration::days(day),
                kind: BackupKind::Exit,
            };
            tokio::fs::write(backups.join(old.to_string()), b"old")
                .await
                .unwrap();
        }
        tokio::fs::write(backups.join("메모.txt"), b"memo")
            .await
            .unwrap();

        create(&pool, &backups, BackupKind::Manual, AT)
            .await
            .unwrap();

        let listed = list(&backups).await.unwrap();
        assert_eq!(listed.len(), 20);
        assert_eq!(listed[0].name.kind, BackupKind::Manual);
        assert!(
            listed
                .iter()
                .all(|backup| backup.name.taken_at != datetime!(2026-01-01 09:00))
        );
        assert!(
            tokio::fs::try_exists(backups.join("메모.txt"))
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn copies_to_second_location_without_leaving_part_files() {
        let (dir, pool) = open_temp().await;
        let backups = dir.path().join("backups");
        let second = dir.path().join("외장 디스크").join("배움마루");
        let name = create(&pool, &backups, BackupKind::Daily, AT)
            .await
            .unwrap();

        copy_to(&backups.join(name.to_string()), name, &second)
            .await
            .unwrap();

        let mut files = Vec::new();
        let mut entries = tokio::fs::read_dir(&second).await.unwrap();
        while let Some(entry) = entries.next_entry().await.unwrap() {
            files.push(entry.file_name().into_string().unwrap());
        }
        assert_eq!(files, [name.to_string()]);
    }

    #[tokio::test]
    async fn missing_folder_lists_nothing() {
        let dir = tempfile::tempdir().unwrap();

        assert!(list(&dir.path().join("없음")).await.unwrap().is_empty());
    }
}
