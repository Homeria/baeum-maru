//! 백업 만들기·목록·정리·두 번째 위치 테스트.

use std::path::PathBuf;

use crate::infra::backup_file;
use crate::infra::backup_file::name::BackupKind;
use crate::infra::backup_file::store::{self};

use time::macros::datetime;

use crate::features::backups::error::SecondLocationError;
use crate::features::backups::service::*;
use crate::infra::db::open_temp;

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
    tokio::fs::write(backup_file::dir(dir.path()), b"not a folder")
        .await
        .unwrap();
    let at = datetime!(2026-11-23 01:00 UTC);

    assert!(backups.make(BackupKind::Exit, at).await.is_none());
    let failure = backups.last_failure().unwrap();
    assert_eq!(failure.kind, BackupKind::Exit);

    tokio::fs::remove_file(backup_file::dir(dir.path()))
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
    tokio::fs::create_dir_all(backup_file::dir(dir.path()))
        .await
        .unwrap();
    let same = backups
        .set_second_location(Some(backup_file::dir(dir.path())))
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
