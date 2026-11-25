//! 백업으로 되돌리기(ADR 0011 평소 복원).
//!
//! 앱이 켜진 채 DB 파일을 바꾸지 않는다. 서버와 하루 백업 작업이 DB를 쥐고 있기 때문이다.
//!
//! 1. 준비([`prepare`]): 고른 파일을 살펴보고, 지금 상태를 복원 직전 백업으로 남기고, 고른 파일을
//!    데이터 폴더에 `restore-pending.db`로 받아 둔다. 그 뒤 앱을 다시 켠다.
//! 2. 바꾸기([`apply_pending`]): 다음 시작 때 DB를 열기 전에 받아 둔 파일로 바꾼다.
//!    실패하면 지금 DB를 그대로 둔다.
//! 3. 기록([`record`]): DB를 연 뒤 되돌리기 전의 묶음 ID와 세대 번호를 다시 써 넣고(BR-H4),
//!    되돌린 DB의 접속 기록에 남긴다.
//!
//! DB가 깨졌을 때([`restore_broken`])는 DB를 쥔 곳이 없어서 그 자리에서 바꾼다. 깨진 DB는
//! 복원 직전 백업을 만들 수 없으니 데이터 폴더에 `baeum-maru.broken.db` 하나로 남긴다.

use std::path::{Path, PathBuf};

use baeum_domain::{DataBundleId, HostIdentity};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use time::{OffsetDateTime, PrimitiveDateTime};

use super::inspect::{BackupPreview, InspectError, inspect};
use crate::features::access_logs::repository::{self as access_logs, AccessEvent};
use crate::infra::backup_file::name::{BackupKind, BackupName};
use crate::infra::backup_file::store::{self, BackupError, StoredBackup};
use crate::infra::clock;
use crate::infra::data_bundle;
use crate::infra::db::{self, DbError};

/// 받아 둔 백업 파일. 이 파일이 있으면 다음 시작 때 바꾼다.
pub(in crate::features::backups) const PENDING_DB: &str = "restore-pending.db";
/// 넘겨받을 파일을 데이터 폴더에 받아 두는 이름. 바로 바꾸고 지운다.
const TAKE_OVER_DB: &str = "take-over-pending.db";
/// 누가 언제 복원했는지.
pub(in crate::features::backups) const PENDING_INFO: &str = "restore-pending.json";
/// 깨진 DB를 남겨 두는 이름. 하나만 둔다.
pub(in crate::features::backups) const BROKEN_DB: &str = "baeum-maru.broken.db";
/// 바꾸는 동안 지금 DB 파일들을 잠깐 옮겨 두는 꼬리.
pub(in crate::features::backups) const REPLACED: &str = ".replaced";
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

/// 깨진 DB를 되돌리지 못했다. 깨진 DB는 제자리에 그대로 있다.
#[derive(Debug, thiserror::Error)]
pub enum BrokenRestoreError {
    #[error(transparent)]
    Inspect(#[from] InspectError),
    #[error("백업 파일을 옮기지 못했습니다: {0}")]
    Io(#[from] std::io::Error),
    #[error("되돌린 DB를 열지 못했습니다: {0}")]
    Open(#[from] DbError),
}

/// 시작할 때 복원한 결과. 백업 탭에 보여 준다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreOutcome {
    /// 되돌렸다. `staff_id`는 복원한 관리자.
    Restored {
        staff_id: Option<i64>,
        /// 되돌리기 전의 묶음 ID와 세대 번호. 되돌린 DB에 다시 써 넣는다(BR-H4).
        /// 읽지 못했으면 `None`이고, 백업에 있던 값을 그대로 쓴다.
        identity: Option<HostIdentity>,
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
    /// 이 칸이 생기기 전에 받아 둔 복원이면 없다.
    #[serde(default)]
    identity: Option<StoredIdentity>,
}

/// 파일에 적는 묶음 ID와 세대 번호.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredIdentity {
    bundle_id: String,
    generation: u32,
}

impl From<HostIdentity> for StoredIdentity {
    fn from(identity: HostIdentity) -> Self {
        StoredIdentity {
            bundle_id: identity.bundle_id.to_string(),
            generation: identity.generation,
        }
    }
}

impl StoredIdentity {
    /// 묶음 ID 형식이 틀리면 `None`.
    fn parse(&self) -> Option<HostIdentity> {
        Some(HostIdentity {
            bundle_id: DataBundleId::parse(&self.bundle_id).ok()?,
            generation: self.generation,
        })
    }
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
        &crate::infra::backup_file::dir(data_dir),
        BackupKind::BeforeRestore,
        clock::local_datetime(now),
    )
    .await
    .map_err(PrepareError::Backup)?;
    // 묶음 ID와 세대 번호가 백업 때로 돌아가지 않게 지금 값을 적어 둔다(BR-H4)
    let identity = data_bundle::read(pool)
        .await
        .inspect_err(|error| tracing::warn!(%error, "데이터 묶음 정보를 읽지 못했습니다"))
        .ok()
        .map(StoredIdentity::from);
    let info =
        serde_json::to_vec(&PendingInfo { staff_id, identity }).map_err(std::io::Error::other)?;
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
    let info = match tokio::fs::read(&info_file).await {
        Ok(bytes) => serde_json::from_slice::<PendingInfo>(&bytes).ok(),
        Err(_) => None,
    };
    let staff_id = info.as_ref().map(|info| info.staff_id);
    let identity = info
        .as_ref()
        .and_then(|info| info.identity.as_ref())
        .and_then(StoredIdentity::parse);
    let at = clock::local_datetime(now);
    let outcome = match swap(data_dir, &pending).await {
        Ok(()) => {
            tracing::info!(?staff_id, "백업으로 되돌림");
            RestoreOutcome::Restored {
                staff_id,
                identity,
                at,
            }
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

/// `source`를 이 데이터 폴더의 DB로 들인다(BR-H3 넘겨받기). DB를 쥔 곳이 없을 때만 부른다.
///
/// 지금 DB 파일이 있으면 바꾸는 동안 옮겨 두었다가, 다 되면 지운다. 실패하면 제자리로 돌려놓는다.
/// 지우기 전에 남길 것은 부르는 쪽이 백업으로 남긴다.
pub async fn install_copy(data_dir: &Path, source: &Path) -> std::io::Result<()> {
    let pending = data_dir.join(TAKE_OVER_DB);
    write_then_rename(&pending, Content::CopyOf(source)).await?;
    let swapped = swap(data_dir, &pending).await;
    if swapped.is_err() {
        let _ = remove_if_exists(&pending).await;
    }
    swapped
}

/// 되돌렸으면 되돌리기 전의 묶음 ID와 세대 번호를 다시 써 넣고(BR-H4) 접속 기록에 남긴다.
/// 복원한 관리자가 되돌린 DB에 없으면 직원 칸을 비운다. DB를 열어 구조 변경을 적용한 뒤 부른다.
pub async fn record(pool: &SqlitePool, outcome: RestoreOutcome, now: OffsetDateTime) {
    let RestoreOutcome::Restored {
        staff_id, identity, ..
    } = outcome
    else {
        return;
    };
    if let Some(identity) = identity
        && let Err(error) = data_bundle::write(pool, identity).await
    {
        // 앱을 멈추지는 않는다. 접속 PC가 옛 세대로 보고 막으면 관리자가 알아챈다
        tracing::error!(%error, "되돌리기 전 데이터 묶음 정보를 다시 쓰지 못했습니다");
    }
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

/// DB가 깨졌을 때 권할 백업. 백업 폴더 `backups_dir`의 백업들(새것부터)과, 그중 새것부터
/// 살펴서 처음으로 쓸 수 있는 것과 그 미리보기.
pub async fn suggest(
    backups_dir: &Path,
) -> (Vec<StoredBackup>, Option<(BackupName, BackupPreview)>) {
    let backups = store::list(backups_dir).await.unwrap_or_else(|error| {
        tracing::warn!(%error, "백업 목록을 읽지 못했습니다");
        Vec::new()
    });
    let mut suggestion = None;
    for backup in &backups {
        let file = backups_dir.join(backup.name.to_string());
        match inspect(&file).await {
            Ok(preview) => {
                suggestion = Some((backup.name, preview));
                break;
            }
            Err(error) => tracing::info!(%error, kind = ?backup.name.kind, "권할 수 없는 백업"),
        }
    }
    (backups, suggestion)
}

/// 깨진 DB를 `source` 백업으로 바로 바꾼다. 앱이 DB를 쥐고 있지 않을 때만 부른다.
///
/// 깨진 DB는 `baeum-maru.broken.db`로 남긴다. 실패하면 깨진 DB를 제자리로 돌려놓는다.
/// 성공하면 되돌린 DB를 열어 구조 변경을 적용하고, 접속 기록에 직원 없이 복원을 남긴다.
pub async fn restore_broken(
    data_dir: &Path,
    source: &Path,
    now: OffsetDateTime,
) -> Result<(), BrokenRestoreError> {
    inspect(source).await?;
    let db_file = db::file(data_dir);
    // 깨진 DB라도 이 표는 읽힐 수 있다. 못 읽으면 백업에 있던 값을 쓴다(BR-H4)
    let identity = data_bundle::read_file(&db_file).await;
    let broken = data_dir.join(BROKEN_DB);
    // 지난번에 남긴 깨진 DB는 지운다. 회원 정보 사본이 쌓이지 않게 하나만 둔다
    for old in db_family(&broken) {
        remove_if_exists(&old).await?;
    }
    let moved = move_aside(&db_file, |suffix| with_suffix(&broken, suffix)).await?;
    if let Err(error) = write_then_rename(&db_file, Content::CopyOf(source)).await {
        put_back(&moved).await;
        return Err(error.into());
    }
    match db::open(data_dir).await {
        Ok(pool) => {
            record(
                &pool,
                RestoreOutcome::Restored {
                    staff_id: None,
                    identity,
                    at: clock::local_datetime(now),
                },
                now,
            )
            .await;
            pool.close().await;
            tracing::info!("깨진 DB를 백업으로 되돌림");
            Ok(())
        }
        Err(error) => {
            for file in db_family(&db_file) {
                let _ = remove_if_exists(&file).await;
            }
            put_back(&moved).await;
            Err(error.into())
        }
    }
}

/// DB 파일과 보조 파일들에 붙는 꼬리. DB 파일 자신은 빈 꼬리다.
const FAMILY: [&str; 3] = ["", SIDE_FILES[0], SIDE_FILES[1]];

/// DB 파일과 보조 파일들(`-wal`, `-shm`).
fn db_family(db_file: &Path) -> Vec<PathBuf> {
    FAMILY
        .iter()
        .map(|suffix| with_suffix(db_file, suffix))
        .collect()
}

/// `db_file`과 보조 파일들 중 있는 것을 옮긴다. `aside`는 꼬리(`""`, `-wal`, `-shm`)를 받아
/// 옮길 곳을 정한다. 옮긴 (원래, 옮긴 곳) 목록을 돌려준다. 중간에 실패하면 되돌려 놓는다.
async fn move_aside(
    db_file: &Path,
    aside: impl Fn(&str) -> PathBuf,
) -> std::io::Result<Vec<(PathBuf, PathBuf)>> {
    let mut moved = Vec::new();
    for suffix in FAMILY {
        let file = with_suffix(db_file, suffix);
        if tokio::fs::try_exists(&file).await? {
            let target = aside(suffix);
            if let Err(error) = tokio::fs::rename(&file, &target).await {
                put_back(&moved).await;
                return Err(error);
            }
            moved.push((file, target));
        }
    }
    Ok(moved)
}

/// 지금 DB 파일들을 옆으로 옮기고 받아 둔 파일을 그 자리에 둔다. 실패하면 되돌려 놓는다.
async fn swap(data_dir: &Path, pending: &Path) -> std::io::Result<()> {
    // 받아 둔 사이에 파일이 상했을 수 있다
    inspect(pending).await.map_err(std::io::Error::other)?;
    let db_file = db::file(data_dir);
    let moved = move_aside(&db_file, |suffix| {
        with_suffix(&with_suffix(&db_file, suffix), REPLACED)
    })
    .await?;
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

pub(in crate::features::backups) fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
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
