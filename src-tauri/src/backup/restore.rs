//! 백업으로 되돌리기(ADR 0011 평소 복원).
//!
//! 앱이 켜진 채 DB 파일을 바꾸지 않는다. 서버와 하루 백업 작업이 DB를 쥐고 있기 때문이다.
//!
//! 1. 준비([`prepare`]): 고른 파일을 살펴보고, 지금 상태를 복원 직전 백업으로 남기고, 고른 파일을
//!    데이터 폴더에 `restore-pending.db`로 받아 둔다. 그 뒤 앱을 다시 켠다.
//! 2. 바꾸기([`apply_pending`]): 다음 시작 때 DB를 열기 전에 받아 둔 파일로 바꾼다.
//!    실패하면 지금 DB를 그대로 둔다.
//! 3. 기록([`record`]): DB를 연 뒤 되돌린 DB의 접속 기록에 남긴다.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use time::{OffsetDateTime, PrimitiveDateTime};

use super::inspect::{InspectError, inspect};
use super::name::{BackupKind, BackupName};
use super::store::{self, BackupError};
use crate::clock;
use crate::db;
use crate::db::access_logs::{self, AccessEvent};

/// 받아 둔 백업 파일. 이 파일이 있으면 다음 시작 때 바꾼다.
const PENDING_DB: &str = "restore-pending.db";
/// 누가 언제 복원했는지.
const PENDING_INFO: &str = "restore-pending.json";
/// 바꾸는 동안 지금 DB 파일들을 잠깐 옮겨 두는 꼬리.
const REPLACED: &str = ".replaced";
/// DB와 함께 다니는 보조 파일(WAL). 남으면 옛 내용이 새 파일에 덮일 수 있다.
const SIDE_FILES: [&str; 2] = ["-wal", "-shm"];

/// 복원을 준비하지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum PrepareError {
    #[error(transparent)]
    Inspect(#[from] InspectError),
    /// 복원 직전 백업을 만들지 못하면 복원하지 않는다.
    #[error("지금 상태를 백업하지 못해 복원하지 않았습니다: {0}")]
    Backup(#[source] BackupError),
    #[error("백업 파일을 받아 두지 못했습니다: {0}")]
    Io(#[from] std::io::Error),
}

/// 시작할 때 복원한 결과. 백업 탭에 보여 준다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreOutcome {
    /// 되돌렸다. `staff_id`는 복원한 관리자.
    Restored {
        staff_id: Option<i64>,
        /// 이 PC의 지역 시각.
        at: PrimitiveDateTime,
    },
    /// 바꾸지 못해 지금 DB를 그대로 쓴다.
    Failed { at: PrimitiveDateTime },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PendingInfo {
    staff_id: i64,
}

/// 복원을 준비한다. 만든 복원 직전 백업 이름을 돌려준다. 그 뒤 앱을 다시 켜면 바뀐다.
///
/// `source`는 백업 폴더의 파일이거나 관리자가 다른 곳에서 고른 파일이다.
pub async fn prepare(
    pool: &SqlitePool,
    data_dir: &Path,
    source: &Path,
    staff_id: i64,
    now: OffsetDateTime,
) -> Result<BackupName, PrepareError> {
    inspect(source).await?;
    let before = store::create(
        pool,
        &super::dir(data_dir),
        BackupKind::BeforeRestore,
        clock::local_datetime(now),
    )
    .await
    .map_err(PrepareError::Backup)?;
    let info = serde_json::to_vec(&PendingInfo { staff_id }).map_err(std::io::Error::other)?;
    write_then_rename(&data_dir.join(PENDING_INFO), Content::Bytes(&info)).await?;
    // 받아 둔 파일이 마지막에 생겨야 반쯤 준비된 복원이 일어나지 않는다
    write_then_rename(&data_dir.join(PENDING_DB), Content::CopyOf(source)).await?;
    Ok(before)
}

/// 받아 둔 백업이 있으면 지금 DB와 바꾼다. DB를 열기 전에 부른다. 받아 둔 것이 없으면 `None`.
pub async fn apply_pending(data_dir: &Path, now: OffsetDateTime) -> Option<RestoreOutcome> {
    let pending = data_dir.join(PENDING_DB);
    if !tokio::fs::try_exists(&pending).await.unwrap_or(false) {
        return None;
    }
    let info_file = data_dir.join(PENDING_INFO);
    let staff_id = match tokio::fs::read(&info_file).await {
        Ok(bytes) => serde_json::from_slice::<PendingInfo>(&bytes)
            .ok()
            .map(|info| info.staff_id),
        Err(_) => None,
    };
    let at = clock::local_datetime(now);
    let outcome = match swap(data_dir, &pending).await {
        Ok(()) => {
            tracing::info!(?staff_id, "백업으로 되돌림");
            RestoreOutcome::Restored { staff_id, at }
        }
        Err(error) => {
            tracing::error!(%error, "백업으로 되돌리지 못해 지금 DB를 그대로 씀");
            RestoreOutcome::Failed { at }
        }
    };
    // 실패했어도 다시 시도하지 않는다. 관리자가 백업 탭에서 다시 고른다
    for file in [&pending, &info_file] {
        if let Err(error) = remove_if_exists(file).await {
            tracing::warn!(%error, "복원 준비 파일을 지우지 못했습니다");
        }
    }
    Some(outcome)
}

/// 되돌렸으면 접속 기록에 남긴다. 복원한 관리자가 되돌린 DB에 없으면 직원 칸을 비운다.
pub async fn record(pool: &SqlitePool, outcome: RestoreOutcome, now: OffsetDateTime) {
    let RestoreOutcome::Restored { staff_id, .. } = outcome else {
        return;
    };
    let result = async {
        let known = match staff_id {
            Some(id) => {
                sqlx::query_scalar::<_, i64>("SELECT id FROM staff WHERE id = ?")
                    .bind(id)
                    .fetch_optional(pool)
                    .await?
            }
            None => None,
        };
        access_logs::record(pool, known, AccessEvent::Restore, now).await
    }
    .await;
    if let Err(error) = result {
        tracing::warn!(%error, "복원을 접속 기록에 남기지 못했습니다");
    }
}

/// 지금 DB 파일들을 옆으로 옮기고 받아 둔 파일을 그 자리에 둔다. 실패하면 되돌려 놓는다.
async fn swap(data_dir: &Path, pending: &Path) -> std::io::Result<()> {
    // 받아 둔 사이에 파일이 상했을 수 있다
    inspect(pending).await.map_err(std::io::Error::other)?;
    let db_file = db::file(data_dir);
    let current: Vec<PathBuf> = std::iter::once(db_file.clone())
        .chain(SIDE_FILES.iter().map(|side| with_suffix(&db_file, side)))
        .collect();
    let mut moved = Vec::new();
    for file in &current {
        if tokio::fs::try_exists(file).await? {
            let aside = with_suffix(file, REPLACED);
            if let Err(error) = tokio::fs::rename(file, &aside).await {
                put_back(&moved).await;
                return Err(error);
            }
            moved.push((file.clone(), aside));
        }
    }
    if let Err(error) = tokio::fs::rename(pending, &db_file).await {
        put_back(&moved).await;
        return Err(error);
    }
    for (_, aside) in &moved {
        // 남아도 다음 복원 때 덮이니 실패는 넘어간다
        let _ = tokio::fs::remove_file(aside).await;
    }
    Ok(())
}

async fn put_back(moved: &[(PathBuf, PathBuf)]) {
    for (original, aside) in moved {
        if let Err(error) = tokio::fs::rename(aside, original).await {
            tracing::error!(%error, "옮겨 둔 DB 파일을 되돌려 놓지 못했습니다");
        }
    }
}

enum Content<'a> {
    Bytes(&'a [u8]),
    CopyOf(&'a Path),
}

async fn write_then_rename(target: &Path, content: Content<'_>) -> std::io::Result<()> {
    let part = with_suffix(target, ".part");
    match content {
        Content::Bytes(bytes) => tokio::fs::write(&part, bytes).await?,
        Content::CopyOf(source) => {
            tokio::fs::copy(source, &part).await?;
        }
    }
    tokio::fs::rename(&part, target).await
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut text = path.as_os_str().to_owned();
    text.push(suffix);
    PathBuf::from(text)
}

async fn remove_if_exists(path: &Path) -> std::io::Result<()> {
    match tokio::fs::remove_file(path).await {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(error),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use time::macros::datetime;

    use super::*;
    use crate::db::open_temp;
    use crate::db::test_support::{insert_member, insert_staff};

    const NOW: OffsetDateTime = datetime!(2026-11-23 01:00 UTC);

    async fn member_count(pool: &SqlitePool) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM members")
            .fetch_one(pool)
            .await
            .unwrap()
    }

    async fn files_in(dir: &Path) -> Vec<String> {
        let mut names = Vec::new();
        let mut entries = tokio::fs::read_dir(dir).await.unwrap();
        while let Some(entry) = entries.next_entry().await.unwrap() {
            names.push(entry.file_name().into_string().unwrap());
        }
        names.sort();
        names
    }

    #[tokio::test]
    async fn restores_backup_on_next_start() {
        let (dir, pool) = open_temp().await;
        let admin = insert_staff(&pool, "관리자", "admin", None).await.unwrap();
        insert_member(&pool, "00001").await.unwrap();
        let backups = super::super::dir(dir.path());
        let one_member = store::create(
            &pool,
            &backups,
            BackupKind::Daily,
            datetime!(2026-11-22 09:00),
        )
        .await
        .unwrap();
        insert_member(&pool, "00002").await.unwrap();

        let before = prepare(
            &pool,
            dir.path(),
            &backups.join(one_member.to_string()),
            admin,
            NOW,
        )
        .await
        .unwrap();
        // 앱을 다시 켠다
        pool.close().await;
        let outcome = apply_pending(dir.path(), NOW).await;
        let pool = db::open(dir.path()).await.unwrap();
        record(&pool, outcome.unwrap(), NOW).await;

        assert!(matches!(
            outcome,
            Some(RestoreOutcome::Restored { staff_id: Some(id), .. }) if id == admin
        ));
        assert_eq!(member_count(&pool).await, 1);
        assert_eq!(before.kind, BackupKind::BeforeRestore);
        // 받아 둔 파일과 옮겨 둔 파일이 남지 않는다
        let left: Vec<String> = files_in(dir.path())
            .await
            .into_iter()
            .filter(|name| name.contains("restore") || name.contains(REPLACED))
            .collect();
        assert!(left.is_empty(), "{left:?}");
        let logged: Vec<(Option<i64>, String)> =
            sqlx::query_as("SELECT staff_id, event FROM access_logs WHERE event = 'restore'")
                .fetch_all(&pool)
                .await
                .unwrap();
        assert_eq!(logged, [(Some(admin), "restore".to_owned())]);
    }

    #[tokio::test]
    async fn before_restore_backup_keeps_current_state() {
        let (dir, pool) = open_temp().await;
        insert_member(&pool, "00001").await.unwrap();
        let backups = super::super::dir(dir.path());
        let empty = store::create(
            &pool,
            &backups,
            BackupKind::Daily,
            datetime!(2026-11-22 09:00),
        )
        .await
        .unwrap();
        insert_member(&pool, "00002").await.unwrap();

        let before = prepare(&pool, dir.path(), &backups.join(empty.to_string()), 1, NOW)
            .await
            .unwrap();

        let preview = inspect(&backups.join(before.to_string())).await.unwrap();
        assert_eq!(preview.member_count, 2);
    }

    #[tokio::test]
    async fn admin_missing_in_restored_db_is_logged_without_staff() {
        let (dir, pool) = open_temp().await;
        let backups = super::super::dir(dir.path());
        let no_staff = store::create(
            &pool,
            &backups,
            BackupKind::Daily,
            datetime!(2026-11-22 09:00),
        )
        .await
        .unwrap();
        // 백업 뒤에 만든 관리자가 복원한다
        let admin = insert_staff(&pool, "관리자", "admin", None).await.unwrap();

        prepare(
            &pool,
            dir.path(),
            &backups.join(no_staff.to_string()),
            admin,
            NOW,
        )
        .await
        .unwrap();
        pool.close().await;
        let outcome = apply_pending(dir.path(), NOW).await.unwrap();
        let pool = db::open(dir.path()).await.unwrap();
        record(&pool, outcome, NOW).await;

        let logged: Vec<Option<i64>> =
            sqlx::query_scalar("SELECT staff_id FROM access_logs WHERE event = 'restore'")
                .fetch_all(&pool)
                .await
                .unwrap();
        assert_eq!(logged, [None]);
    }

    #[tokio::test]
    async fn refuses_bad_file_without_backing_up_or_staging() {
        let (dir, pool) = open_temp().await;
        let bad = dir.path().join("메모.db");
        tokio::fs::write(&bad, b"not a database").await.unwrap();

        let result = prepare(&pool, dir.path(), &bad, 1, NOW).await;

        assert!(matches!(result, Err(PrepareError::Inspect(_))));
        assert!(
            store::list(&super::super::dir(dir.path()))
                .await
                .unwrap()
                .is_empty()
        );
        assert!(apply_pending(dir.path(), NOW).await.is_none());
    }

    #[tokio::test]
    async fn damaged_pending_file_keeps_current_db() {
        let (dir, pool) = open_temp().await;
        insert_member(&pool, "00001").await.unwrap();
        pool.close().await;
        tokio::fs::write(dir.path().join(PENDING_DB), b"broken")
            .await
            .unwrap();

        let outcome = apply_pending(dir.path(), NOW).await;

        assert!(matches!(outcome, Some(RestoreOutcome::Failed { .. })));
        let pool = db::open(dir.path()).await.unwrap();
        assert_eq!(member_count(&pool).await, 1);
        assert!(
            !tokio::fs::try_exists(dir.path().join(PENDING_DB))
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn nothing_pending_means_nothing_happens() {
        let dir = tempfile::tempdir().unwrap();

        assert_eq!(apply_pending(dir.path(), NOW).await, None);
    }
}
