//! 복원 테스트.

use std::path::{Path, PathBuf};

use baeum_domain::HostIdentity;
use sqlx::SqlitePool;
use time::OffsetDateTime;

use crate::features::backups::service::inspect::inspect;
use crate::infra::backup_file::name::{BackupKind, BackupName};
use crate::infra::backup_file::store::{self};
use crate::infra::data_bundle;
use crate::infra::db::{self};

use time::macros::datetime;

use crate::db::test_support::{insert_member, insert_staff};
use crate::features::backups::error::{BrokenRestoreError, PrepareError};
use crate::features::backups::service::restore::*;
use crate::infra::db::open_temp;

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
    let backups = crate::infra::backup_file::dir(dir.path());
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
async fn br_h4_restore_keeps_bundle_id_and_generation() {
    let (dir, pool) = open_temp().await;
    let admin = insert_staff(&pool, "관리자", "admin", None).await.unwrap();
    let backups = crate::infra::backup_file::dir(dir.path());
    // 세대 1일 때 만든 백업
    let old = store::create(
        &pool,
        &backups,
        BackupKind::Daily,
        datetime!(2026-11-22 09:00),
    )
    .await
    .unwrap();
    let now = HostIdentity {
        bundle_id: data_bundle::read(&pool).await.unwrap().bundle_id,
        generation: 3,
    };
    data_bundle::write(&pool, now).await.unwrap();

    prepare(
        &pool,
        dir.path(),
        &backups.join(old.to_string()),
        admin,
        NOW,
    )
    .await
    .unwrap();
    pool.close().await;
    let outcome = apply_pending(dir.path(), NOW).await.unwrap();
    let pool = db::open(dir.path()).await.unwrap();
    record(&pool, outcome, NOW).await;

    assert_eq!(data_bundle::read(&pool).await.unwrap(), now);
}

#[tokio::test]
async fn pending_info_from_before_identity_still_restores() {
    let dir = tempfile::tempdir().unwrap();
    let pool = db::open(dir.path()).await.unwrap();
    let backup = store::create(
        &pool,
        &crate::infra::backup_file::dir(dir.path()),
        BackupKind::Daily,
        datetime!(2026-11-22 09:00),
    )
    .await
    .unwrap();
    pool.close().await;
    tokio::fs::write(dir.path().join(PENDING_INFO), br#"{"staffId":1}"#)
        .await
        .unwrap();
    tokio::fs::copy(
        crate::infra::backup_file::dir(dir.path()).join(backup.to_string()),
        dir.path().join(PENDING_DB),
    )
    .await
    .unwrap();

    let outcome = apply_pending(dir.path(), NOW).await;

    assert!(matches!(
        outcome,
        Some(RestoreOutcome::Restored {
            staff_id: Some(1),
            identity: None,
            ..
        })
    ));
}

#[tokio::test]
async fn before_restore_backup_keeps_current_state() {
    let (dir, pool) = open_temp().await;
    insert_member(&pool, "00001").await.unwrap();
    let backups = crate::infra::backup_file::dir(dir.path());
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
    let backups = crate::infra::backup_file::dir(dir.path());
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
        store::list(&crate::infra::backup_file::dir(dir.path()))
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

/// 데이터 폴더에 회원 한 명짜리 하루 백업을 만들고, DB를 깨뜨린다. 그 백업 파일을 돌려준다.
async fn broken_with_backup(dir: &Path) -> PathBuf {
    let pool = db::open(dir).await.unwrap();
    insert_member(&pool, "00001").await.unwrap();
    let name = store::create(
        &pool,
        &crate::infra::backup_file::dir(dir),
        BackupKind::Daily,
        datetime!(2026-11-22 09:00),
    )
    .await
    .unwrap();
    pool.close().await;
    tokio::fs::write(db::file(dir), b"broken db").await.unwrap();
    tokio::fs::write(with_suffix(&db::file(dir), "-wal"), b"old wal")
        .await
        .unwrap();
    crate::infra::backup_file::dir(dir).join(name.to_string())
}

#[tokio::test]
async fn broken_db_is_replaced_and_kept_aside_once() {
    let dir = tempfile::tempdir().unwrap();
    let backup = broken_with_backup(dir.path()).await;
    // 지난번에 남긴 깨진 DB
    tokio::fs::write(dir.path().join(BROKEN_DB), b"older broken")
        .await
        .unwrap();

    restore_broken(dir.path(), &backup, NOW).await.unwrap();

    let pool = db::open(dir.path()).await.unwrap();
    assert_eq!(member_count(&pool).await, 1);
    let logged: Vec<Option<i64>> =
        sqlx::query_scalar("SELECT staff_id FROM access_logs WHERE event = 'restore'")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(logged, [None]);
    // 깨진 DB는 하나만, 새것으로 남는다
    assert_eq!(
        tokio::fs::read(dir.path().join(BROKEN_DB)).await.unwrap(),
        b"broken db"
    );
    assert_eq!(
        tokio::fs::read(with_suffix(&dir.path().join(BROKEN_DB), "-wal"))
            .await
            .unwrap(),
        b"old wal"
    );
}

#[tokio::test]
async fn failed_broken_restore_leaves_broken_db_in_place() {
    let dir = tempfile::tempdir().unwrap();
    broken_with_backup(dir.path()).await;
    let bad = dir.path().join("메모.db");
    tokio::fs::write(&bad, b"not a database").await.unwrap();

    let result = restore_broken(dir.path(), &bad, NOW).await;

    assert!(matches!(result, Err(BrokenRestoreError::Inspect(_))));
    assert_eq!(
        tokio::fs::read(db::file(dir.path())).await.unwrap(),
        b"broken db"
    );
    assert!(
        !tokio::fs::try_exists(dir.path().join(BROKEN_DB))
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn suggests_newest_usable_backup() {
    let dir = tempfile::tempdir().unwrap();
    let usable = broken_with_backup(dir.path()).await;
    let backups = crate::infra::backup_file::dir(dir.path());
    // 더 새것이지만 깨진 백업
    let newer = BackupName {
        taken_at: datetime!(2026-11-23 09:00),
        kind: BackupKind::Exit,
    };
    tokio::fs::write(backups.join(newer.to_string()), b"broken backup")
        .await
        .unwrap();

    let (listed, suggestion) = suggest(&backups).await;

    assert_eq!(listed.len(), 2);
    let (name, preview) = suggestion.unwrap();
    assert_eq!(backups.join(name.to_string()), usable);
    assert_eq!(preview.member_count, 1);
}

#[tokio::test]
async fn no_backups_means_no_suggestion() {
    let dir = tempfile::tempdir().unwrap();

    let (listed, suggestion) = suggest(&crate::infra::backup_file::dir(dir.path())).await;

    assert!(listed.is_empty());
    assert!(suggestion.is_none());
}
