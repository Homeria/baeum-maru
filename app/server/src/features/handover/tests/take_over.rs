//! 넘겨받기 테스트.

use std::path::{Path, PathBuf};

use time::OffsetDateTime;
use time::macros::datetime;

use crate::features::backups::service::inspect::inspect;
use crate::features::handover::service::run_mode::RunMode;
use crate::infra::backup_file::name::BackupKind;
use crate::infra::backup_file::{self, store};
use crate::infra::secret;
use crate::infra::{data_bundle, db};

use crate::db::test_support::{insert_member, insert_staff};
use crate::features::access_logs::repository::list_for_test;
use crate::features::handover::service::take_over::*;

const NOW: OffsetDateTime = datetime!(2026-11-25 01:00 UTC);

/// 옛 호스트의 넘기기 파일을 만든다. PIN 123456인 관리자와 회원 한 명. 관리자 id를 함께 준다.
async fn handover_file(dir: &Path) -> (PathBuf, i64) {
    let old_host = dir.join("옛 호스트");
    let pool = db::open(&old_host).await.unwrap();
    let admin = insert_staff(&pool, "관리자", "admin", None).await.unwrap();
    insert_staff(&pool, "직원1", "staff", None).await.unwrap();
    sqlx::query("UPDATE staff SET pin_hash = ?")
        .bind(secret::hash("123456").unwrap())
        .execute(&pool)
        .await
        .unwrap();
    insert_member(&pool, "00001").await.unwrap();
    let name = store::create(
        &pool,
        &backup_file::dir(&old_host),
        BackupKind::BeforeHandover,
        datetime!(2026-11-25 09:00),
    )
    .await
    .unwrap();
    pool.close().await;
    (backup_file::dir(&old_host).join(name.to_string()), admin)
}

#[tokio::test]
async fn preview_lists_only_admins() {
    let dir = tempfile::tempdir().unwrap();
    let (file, admin) = handover_file(dir.path()).await;

    let preview = preview(&file).await.unwrap();

    assert_eq!(preview.member_count, 1);
    assert_eq!(
        preview.admins,
        [TakeOverAdmin {
            id: admin,
            name: "관리자".to_owned()
        }]
    );
}

#[tokio::test]
async fn br_h3_take_over_raises_generation_and_keeps_bundle_id() {
    let dir = tempfile::tempdir().unwrap();
    let (file, admin) = handover_file(dir.path()).await;
    let source = data_bundle::read_file(&file).await.unwrap();
    let new_pc = dir.path().join("새 PC");

    let generation = take_over_file(&new_pc, &file, admin, "123456".to_owned(), NOW)
        .await
        .unwrap();

    assert_eq!(generation, 2);
    let pool = db::open(&new_pc).await.unwrap();
    let now = data_bundle::read(&pool).await.unwrap();
    assert_eq!(now.bundle_id, source.bundle_id);
    assert_eq!(now.generation, 2);
    assert_eq!(
        list_for_test(&pool).await.last().unwrap(),
        &(Some(admin), "take_over".to_owned())
    );
    // 넘기기 파일은 그대로다
    assert_eq!(data_bundle::read_file(&file).await.unwrap(), source);
}

#[tokio::test]
async fn br_h3_wrong_pin_or_non_admin_changes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let (file, admin) = handover_file(dir.path()).await;
    let new_pc = dir.path().join("새 PC");

    let wrong = take_over_file(&new_pc, &file, admin, "000000".to_owned(), NOW).await;
    let worker = take_over_file(&new_pc, &file, admin + 1, "123456".to_owned(), NOW).await;

    assert!(matches!(wrong, Err(TakeOverError::WrongPin)));
    assert!(matches!(worker, Err(TakeOverError::AdminNotFound)));
    assert!(!db::file(&new_pc).exists());
}

#[tokio::test]
async fn br_h3_leftover_data_is_kept_as_backup() {
    let dir = tempfile::tempdir().unwrap();
    let (file, admin) = handover_file(dir.path()).await;
    let new_pc = dir.path().join("새 PC");
    let leftover = db::open(&new_pc).await.unwrap();
    insert_member(&leftover, "00001").await.unwrap();
    insert_member(&leftover, "00002").await.unwrap();
    leftover.close().await;

    take_over_file(&new_pc, &file, admin, "123456".to_owned(), NOW)
        .await
        .unwrap();

    let backups = store::list(&backup_file::dir(&new_pc)).await.unwrap();
    assert_eq!(backups[0].name.kind, BackupKind::BeforeTakeOver);
    let kept = inspect(&backup_file::dir(&new_pc).join(backups[0].name.to_string()))
        .await
        .unwrap();
    assert_eq!(kept.member_count, 2);
}

#[tokio::test]
async fn broken_leftover_is_moved_aside() {
    let dir = tempfile::tempdir().unwrap();
    let (file, admin) = handover_file(dir.path()).await;
    let new_pc = dir.path().join("새 PC");
    tokio::fs::create_dir_all(&new_pc).await.unwrap();
    tokio::fs::write(db::file(&new_pc), "망가진 파일 ".repeat(1000))
        .await
        .unwrap();

    take_over_file(&new_pc, &file, admin, "123456".to_owned(), NOW)
        .await
        .unwrap();

    assert!(new_pc.join(KEPT_DB).exists());
    let pool = db::open(&new_pc).await.unwrap();
    assert_eq!(data_bundle::read(&pool).await.unwrap().generation, 2);
}

#[test]
fn only_pc_that_is_not_yet_host_takes_over() {
    assert!(can_take_over(None));
    assert!(can_take_over(Some(RunMode::Remote)));
    assert!(!can_take_over(Some(RunMode::Host)));
    assert!(!can_take_over(Some(RunMode::HandingOver)));
    assert!(!can_take_over(Some(RunMode::HandedOver)));
}
