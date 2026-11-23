//! 백업(ADR 0011): DB를 통째로 복사한 파일을 쌓고 오래된 것을 정리한다.
//!
//! 백업마다 파일 하나다. 언제, 왜 만들었는지는 파일 이름에 적어 두고 다시 읽는다.
//! 그래서 목록이나 마지막 복사 시각을 따로 저장하지 않는다.
//!
//! 자동 백업이 실패해도 하던 일(추첨, 마감, 종료)은 막지 않는다. 로그만 남긴다.

pub mod name;
pub mod retention;
pub mod store;

use std::path::{Path, PathBuf};
use std::time::Duration;

use sqlx::SqlitePool;
use time::OffsetDateTime;

use crate::clock;
use name::{BackupKind, BackupName};

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
    dir: PathBuf,
}

impl Backups {
    /// `dir`(첫 번째 위치)에 `pool`의 DB를 백업한다.
    pub fn new(pool: SqlitePool, dir: PathBuf) -> Self {
        Backups { pool, dir }
    }

    /// 백업을 하나 만든다. 실패하면 로그를 남기고 `None`.
    pub async fn make(&self, kind: BackupKind, now: OffsetDateTime) -> Option<BackupName> {
        match store::create(&self.pool, &self.dir, kind, clock::local_datetime(now)).await {
            Ok(name) => {
                tracing::info!(kind = ?name.kind, "백업");
                Some(name)
            }
            Err(error) => {
                tracing::warn!(%error, ?kind, "백업하지 못했습니다");
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
            None
        } else {
            self.make(BackupKind::Daily, now).await
        }
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
        let backups = Backups::new(pool, super::dir(dir.path()));
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
}
