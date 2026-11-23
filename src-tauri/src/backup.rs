//! 백업(ADR 0011): DB를 통째로 복사한 파일을 쌓고 오래된 것을 정리한다.
//!
//! 백업마다 파일 하나다. 언제, 왜 만들었는지는 파일 이름에 적어 두고 다시 읽는다.
//! 그래서 목록이나 마지막 복사 시각을 따로 저장하지 않는다.
//!
//! 자동 백업이 실패해도 하던 일(추첨, 마감, 종료)은 막지 않는다. 로그를 남기고, 마지막 실패를
//! 기억해 백업 탭에 보여 준다.

pub mod inspect;
pub mod name;
pub mod restore;
pub mod retention;
pub mod settings;
pub mod store;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use sqlx::SqlitePool;
use time::{OffsetDateTime, PrimitiveDateTime};

use crate::clock;
use name::{BackupKind, BackupName};
use restore::{PrepareError, RestoreOutcome};
use settings::BackupSettings;
use store::{BackupError, StoredBackup};

/// 두 번째 위치를 바꾸지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum SecondLocationError {
    #[error("폴더 경로는 드라이브부터 적어야 합니다")]
    NotAbsolute,
    #[error("없는 폴더입니다. 외장 디스크나 USB가 꽂혀 있는지 확인해 주세요")]
    NotFolder,
    #[error("첫 번째 백업 폴더와 같은 곳은 고를 수 없습니다. 다른 디스크의 폴더를 골라 주세요")]
    SameAsFirst,
    #[error("이 폴더에 파일을 쓸 수 없습니다. 다른 폴더를 골라 주세요")]
    NotWritable(#[source] std::io::Error),
    #[error("백업 설정을 저장하지 못했습니다: {0}")]
    Save(#[source] std::io::Error),
}

/// 마지막으로 실패한 자동 백업. 다음 자동 백업이 되면 지운다. 백업 탭에 보여 준다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Failure {
    /// 이 PC의 지역 시각.
    pub at: PrimitiveDateTime,
    pub kind: BackupKind,
}

/// 두 번째 위치의 상태. 관리 화면에 보여 준다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecondLocationStatus {
    /// 고른 폴더. 없으면 꺼짐.
    pub path: Option<PathBuf>,
    /// 지금 그 폴더가 있는지(외장 디스크가 꽂혀 있는지).
    pub reachable: bool,
    /// 그 폴더에 있는 가장 최근 하루 백업의 시각.
    pub last_copied_at: Option<PrimitiveDateTime>,
}

/// 데이터 폴더 안의 백업 폴더 이름(첫 번째 위치).
const DIR_NAME: &str = "backups";
/// 켜 둔 채 날이 바뀌었는지 보는 간격.
const DAILY_CHECK: Duration = Duration::from_secs(10 * 60);

/// 데이터 폴더 `data_dir` 안의 백업 폴더.
pub fn dir(data_dir: &Path) -> PathBuf {
    data_dir.join(DIR_NAME)
}

/// 이 앱의 백업. 요청 처리 함수와 앱 종료 때 함께 쓴다.
#[derive(Debug, Clone)]
pub struct Backups {
    pool: SqlitePool,
    data_dir: PathBuf,
    /// 첫 번째 위치.
    dir: PathBuf,
    settings_file: PathBuf,
    /// 이번에 켤 때 복원한 결과. 켤 때 한 번 정하고 바꾸지 않는다.
    restore_outcome: Option<RestoreOutcome>,
    /// 서버, 하루 백업 작업, 앱 종료가 함께 쓰고 고친다. 잠깐 쓰고 바로 놓는다.
    last_failure: Arc<Mutex<Option<Failure>>>,
}

impl Backups {
    /// 데이터 폴더 `data_dir`의 백업 폴더(첫 번째 위치)에 `pool`의 DB를 백업한다.
    pub fn new(pool: SqlitePool, data_dir: &Path) -> Self {
        Backups {
            pool,
            data_dir: data_dir.to_owned(),
            dir: dir(data_dir),
            settings_file: settings::file(data_dir),
            restore_outcome: None,
            last_failure: Arc::new(Mutex::new(None)),
        }
    }

    /// 이번에 켤 때 복원한 결과를 붙인다.
    pub fn with_restore_outcome(self, restore_outcome: Option<RestoreOutcome>) -> Self {
        Backups {
            restore_outcome,
            ..self
        }
    }

    /// 이번에 켤 때 복원한 결과. 복원하지 않았으면 `None`.
    pub fn restore_outcome(&self) -> Option<RestoreOutcome> {
        self.restore_outcome
    }

    /// `source` 백업으로 복원을 준비한다. 앱을 다시 켜면 바뀐다. 만든 복원 직전 백업 이름을 돌려준다.
    pub async fn prepare_restore(
        &self,
        source: &Path,
        staff_id: i64,
        now: OffsetDateTime,
    ) -> Result<BackupName, PrepareError> {
        let before = restore::prepare(&self.pool, &self.data_dir, source, staff_id, now).await?;
        tracing::info!(staff_id, "복원 준비");
        Ok(before)
    }

    /// 마지막으로 실패한 자동 백업. 그 뒤로 자동 백업이 됐으면 `None`.
    pub fn last_failure(&self) -> Option<Failure> {
        // 다른 쪽이 쥔 채 멈췄어도 값은 그대로 읽는다
        *self
            .last_failure
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    fn set_last_failure(&self, failure: Option<Failure>) {
        *self
            .last_failure
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = failure;
    }

    /// 첫 번째 위치의 백업들. 새것부터.
    pub async fn list(&self) -> Result<Vec<StoredBackup>, BackupError> {
        store::list(&self.dir).await
    }

    /// 관리자가 누른 "지금 백업". 실패하면 오류를 돌려줘 화면에 알린다.
    pub async fn make_now(&self, now: OffsetDateTime) -> Result<BackupName, BackupError> {
        let name = store::create(
            &self.pool,
            &self.dir,
            BackupKind::Manual,
            clock::local_datetime(now),
        )
        .await?;
        tracing::info!("지금 백업");
        Ok(name)
    }

    /// 두 번째 위치의 상태.
    pub async fn second_location(&self) -> SecondLocationStatus {
        let Some(path) = settings::load(&self.settings_file).await.second_location else {
            return SecondLocationStatus {
                path: None,
                reachable: false,
                last_copied_at: None,
            };
        };
        let reachable = tokio::fs::metadata(&path)
            .await
            .is_ok_and(|metadata| metadata.is_dir());
        let last_copied_at = if reachable {
            store::list(&path).await.ok().and_then(|backups| {
                backups
                    .into_iter()
                    .find(|backup| backup.name.kind == BackupKind::Daily)
                    .map(|backup| backup.name.taken_at)
            })
        } else {
            None
        };
        SecondLocationStatus {
            path: Some(path),
            reachable,
            last_copied_at,
        }
    }

    /// 두 번째 위치를 바꾼다. `None`이면 끈다. 켜면 가장 최근 하루 백업을 바로 한 부 복사한다.
    pub async fn set_second_location(
        &self,
        path: Option<PathBuf>,
    ) -> Result<(), SecondLocationError> {
        if let Some(path) = &path {
            self.check_second_location(path).await?;
        }
        let settings = BackupSettings {
            second_location: path,
        };
        settings::save(&self.settings_file, &settings)
            .await
            .map_err(SecondLocationError::Save)?;
        let latest_daily = self.list().await.ok().and_then(|backups| {
            backups
                .into_iter()
                .find(|backup| backup.name.kind == BackupKind::Daily)
        });
        if let Some(backup) = latest_daily {
            self.copy_to_second(backup.name).await;
        }
        Ok(())
    }

    async fn check_second_location(&self, path: &Path) -> Result<(), SecondLocationError> {
        if !path.is_absolute() {
            return Err(SecondLocationError::NotAbsolute);
        }
        let is_dir = tokio::fs::metadata(path)
            .await
            .is_ok_and(|metadata| metadata.is_dir());
        if !is_dir {
            return Err(SecondLocationError::NotFolder);
        }
        let chosen = tokio::fs::canonicalize(path)
            .await
            .map_err(|_| SecondLocationError::NotFolder)?;
        if let Ok(first) = tokio::fs::canonicalize(&self.dir).await
            && (chosen.starts_with(&first) || first.starts_with(&chosen))
        {
            return Err(SecondLocationError::SameAsFirst);
        }
        // 쓸 수 있는지 작은 파일을 써 보고 지운다
        let probe = chosen.join(".baeum-maru-write-test");
        tokio::fs::write(&probe, b"ok")
            .await
            .map_err(SecondLocationError::NotWritable)?;
        tokio::fs::remove_file(&probe)
            .await
            .map_err(SecondLocationError::NotWritable)
    }

    /// 하루 백업을 두 번째 위치에 복사한다. 꺼져 있으면 하지 않고, 실패하면 로그만 남긴다.
    async fn copy_to_second(&self, name: BackupName) {
        let Some(second) = settings::load(&self.settings_file).await.second_location else {
            return;
        };
        match store::copy_to(&self.dir.join(name.to_string()), name, &second).await {
            Ok(()) => tracing::info!("두 번째 위치에 백업 복사"),
            // 외장 디스크가 빠져 있으면 조용히 넘어간다. 관리 화면이 마지막 복사 시각으로 알린다
            Err(error) => tracing::warn!(%error, "두 번째 위치에 복사하지 못했습니다"),
        }
    }

    /// 백업을 하나 만든다. 실패하면 로그를 남기고 `None`.
    pub async fn make(&self, kind: BackupKind, now: OffsetDateTime) -> Option<BackupName> {
        let at = clock::local_datetime(now);
        match store::create(&self.pool, &self.dir, kind, at).await {
            Ok(name) => {
                tracing::info!(kind = ?name.kind, "백업");
                self.set_last_failure(None);
                Some(name)
            }
            Err(error) => {
                tracing::warn!(%error, ?kind, "백업하지 못했습니다");
                self.set_last_failure(Some(Failure { at, kind }));
                None
            }
        }
    }

    /// 오늘(이 PC의 날짜) 하루 백업이 없으면 만든다.
    pub async fn daily_if_missing(&self, now: OffsetDateTime) -> Option<BackupName> {
        let today = clock::local_datetime(now).date();
        let made_today = match store::list(&self.dir).await {
            Ok(backups) => backups.iter().any(|backup| {
                backup.name.kind == BackupKind::Daily && backup.name.taken_at.date() == today
            }),
            Err(error) => {
                tracing::warn!(%error, "백업 목록을 읽지 못했습니다");
                false
            }
        };
        if made_today {
            return None;
        }
        let name = self.make(BackupKind::Daily, now).await?;
        self.copy_to_second(name).await;
        Some(name)
    }

    /// 켤 때, 그리고 켜 둔 채 날이 바뀌면 하루 백업을 만든다. 앱이 꺼질 때까지 돈다.
    pub async fn run_daily(self) {
        let mut ticks = tokio::time::interval(DAILY_CHECK);
        loop {
            // 첫 tick은 바로 온다: 켤 때 백업
            ticks.tick().await;
            self.daily_if_missing(OffsetDateTime::now_utc()).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use time::macros::datetime;

    use super::*;
    use crate::db::open_temp;

    #[tokio::test]
    async fn daily_backup_is_made_once_a_day() {
        let (dir, pool) = open_temp().await;
        let backups = Backups::new(pool, dir.path());
        let morning = datetime!(2026-11-23 01:00 UTC);

        let first = backups.daily_if_missing(morning).await;
        let again = backups
            .daily_if_missing(morning + time::Duration::minutes(30))
            .await;
        let next_day = backups
            .daily_if_missing(morning + time::Duration::days(1))
            .await;

        assert!(first.is_some());
        assert!(again.is_none());
        assert!(next_day.is_some());
    }

    #[tokio::test]
    async fn daily_backup_is_copied_to_second_location() {
        let (dir, pool) = open_temp().await;
        let backups = Backups::new(pool, dir.path());
        let second = tempfile::tempdir().unwrap();
        let morning = datetime!(2026-11-23 01:00 UTC);
        backups.daily_if_missing(morning).await.unwrap();

        // 켜는 순간 가장 최근 하루 백업이 복사된다
        backups
            .set_second_location(Some(second.path().to_owned()))
            .await
            .unwrap();
        let status = backups.second_location().await;
        assert!(status.reachable);
        let first_copy = status.last_copied_at.unwrap();

        backups
            .daily_if_missing(morning + time::Duration::days(1))
            .await;

        let later = backups.second_location().await.last_copied_at.unwrap();
        assert!(later > first_copy);
        // 지금 백업은 두 번째 위치로 가지 않는다
        backups
            .make_now(morning + time::Duration::days(1))
            .await
            .unwrap();
        assert_eq!(store::list(second.path()).await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn failed_backup_is_remembered_until_next_success() {
        let (dir, pool) = open_temp().await;
        let backups = Backups::new(pool, dir.path());
        // 백업 폴더 자리에 파일이 있어 폴더를 만들 수 없다
        tokio::fs::write(super::dir(dir.path()), b"not a folder")
            .await
            .unwrap();
        let at = datetime!(2026-11-23 01:00 UTC);

        assert!(backups.make(BackupKind::Exit, at).await.is_none());
        let failure = backups.last_failure().unwrap();
        assert_eq!(failure.kind, BackupKind::Exit);

        tokio::fs::remove_file(super::dir(dir.path()))
            .await
            .unwrap();
        backups.make(BackupKind::Exit, at).await.unwrap();
        assert_eq!(backups.last_failure(), None);
    }

    #[tokio::test]
    async fn refuses_unusable_second_locations() {
        let (dir, pool) = open_temp().await;
        let backups = Backups::new(pool, dir.path());

        let relative = backups
            .set_second_location(Some(PathBuf::from("백업")))
            .await;
        let missing = backups
            .set_second_location(Some(dir.path().join("없는 폴더")))
            .await;
        tokio::fs::create_dir_all(super::dir(dir.path()))
            .await
            .unwrap();
        let same = backups
            .set_second_location(Some(super::dir(dir.path())))
            .await;

        assert!(matches!(relative, Err(SecondLocationError::NotAbsolute)));
        assert!(matches!(missing, Err(SecondLocationError::NotFolder)));
        assert!(matches!(same, Err(SecondLocationError::SameAsFirst)));
        assert_eq!(backups.second_location().await.path, None);
    }

    #[tokio::test]
    async fn turning_off_second_location() {
        let (dir, pool) = open_temp().await;
        let backups = Backups::new(pool, dir.path());
        let second = tempfile::tempdir().unwrap();
        backups
            .set_second_location(Some(second.path().to_owned()))
            .await
            .unwrap();

        backups.set_second_location(None).await.unwrap();

        assert_eq!(
            backups.second_location().await,
            SecondLocationStatus {
                path: None,
                reachable: false,
                last_copied_at: None
            }
        );
    }
}
