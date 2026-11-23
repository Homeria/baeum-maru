//! 오래된 백업 정리(ADR 0011).
//!
//! - 하루 한 번 백업: 최근 30개(하루 하나씩이니 30일치)
//! - 지금 백업, 앱 종료, 추첨·선발 마감 직전, 복원 직전: 합쳐서 최근 20개
//! - 구조 변경 직전: 지우지 않는다

use super::name::{BackupKind, BackupName};

/// 하루 한 번 백업을 남기는 개수.
const DAILY_KEEP: usize = 30;
/// 그때그때 만든 백업(지금 백업, 종료, 작업 직전)을 합쳐 남기는 개수.
const EVENT_KEEP: usize = 20;

/// 보관 개수를 넘은 백업들. 가장 오래된 것부터 지운다.
pub fn to_remove(backups: &[BackupName]) -> Vec<BackupName> {
    let mut daily: Vec<BackupName> = Vec::new();
    let mut event: Vec<BackupName> = Vec::new();
    for backup in backups {
        match backup.kind {
            BackupKind::Daily => daily.push(*backup),
            BackupKind::Manual
            | BackupKind::Exit
            | BackupKind::BeforeLottery { .. }
            | BackupKind::BeforeClose { .. }
            | BackupKind::BeforeRestore => event.push(*backup),
            BackupKind::BeforeMigration => {}
        }
    }
    let mut removed = oldest_beyond(daily, DAILY_KEEP);
    removed.extend(oldest_beyond(event, EVENT_KEEP));
    removed.sort();
    removed
}

/// 새것부터 `keep`개를 남기고 나머지.
fn oldest_beyond(mut backups: Vec<BackupName>, keep: usize) -> Vec<BackupName> {
    backups.sort_by(|a, b| b.cmp(a));
    backups.split_off(keep.min(backups.len()))
}

#[cfg(test)]
mod tests {
    use time::Duration;
    use time::macros::datetime;

    use super::*;

    /// 같은 종류 백업 `count`개. 하루씩 차이 난다.
    fn days(kind: BackupKind, count: i64) -> Vec<BackupName> {
        (0..count)
            .map(|day| BackupName {
                taken_at: datetime!(2026-01-01 09:00) + Duration::days(day),
                kind,
            })
            .collect()
    }

    #[test]
    fn keeps_newest_30_daily_backups() {
        let daily = days(BackupKind::Daily, 33);

        let removed = to_remove(&daily);

        assert_eq!(removed, daily[..3]);
    }

    #[test]
    fn event_backups_share_20_places() {
        let mut backups = days(BackupKind::Exit, 15);
        backups.extend(days(BackupKind::BeforeLottery { course_id: 1 }, 4));
        backups.extend(days(BackupKind::Manual, 2));
        backups.extend(days(BackupKind::BeforeRestore, 1));

        let removed = to_remove(&backups);

        // 22개 중 가장 오래된 2개. 같은 날이면 종류 순서로 앞선 종료 백업이 먼저다
        assert_eq!(removed.len(), 2);
        assert_eq!(removed[0].taken_at, datetime!(2026-01-01 09:00));
        assert_eq!(removed[1].taken_at, datetime!(2026-01-01 09:00));
    }

    #[test]
    fn migration_backups_are_never_removed_and_do_not_take_places() {
        let mut backups = days(BackupKind::BeforeMigration, 50);
        backups.extend(days(BackupKind::Daily, 30));

        assert!(to_remove(&backups).is_empty());
    }
}
