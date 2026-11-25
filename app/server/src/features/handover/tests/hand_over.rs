//! 호스트 넘기기 테스트.

use std::path::Path;

use time::OffsetDateTime;
use time::macros::datetime;

use crate::features::backups::service::inspect::inspect;
use crate::features::handover::service::run_mode::{self, RunMode};
use crate::infra::backup_file::name::BackupKind;
use crate::infra::backup_file::{self, store};
use crate::infra::db;

use crate::db::test_support::insert_staff;
use crate::features::access_logs::repository::list_for_test;
use crate::features::handover::service::hand_over::*;
use crate::infra::data_bundle;

const NOW: OffsetDateTime = datetime!(2026-11-25 01:00 UTC);

#[tokio::test]
async fn br_h1_prepare_makes_before_handover_backup() {
    let dir = tempfile::tempdir().unwrap();
    db::open(dir.path()).await.unwrap().close().await;

    let handing_over = prepare(dir.path(), NOW).await;

    let file = handing_over.file.unwrap();
    assert!(file.starts_with(backup_file::dir(dir.path())));
    assert!(inspect(&file).await.is_ok());
    let listed = store::list(&backup_file::dir(dir.path())).await.unwrap();
    assert_eq!(listed[0].name.kind, BackupKind::BeforeHandover);
}

#[tokio::test]
async fn saved_file_is_checked() {
    let dir = tempfile::tempdir().unwrap();
    db::open(dir.path()).await.unwrap().close().await;
    let source = prepare(dir.path(), NOW).await.file.unwrap();
    let target = dir.path().join("넘기기.db");

    copy_checked(&source, &target).await.unwrap();

    assert!(inspect(&target).await.is_ok());
    assert!(
        copy_checked(&source, &dir.path().join("없는 폴더/넘기기.db"))
            .await
            .is_err()
    );
}

/// 넘긴 PC의 DB에 PIN 123456인 관리자를 만든다.
async fn handed_over_with_admin(data_dir: &Path) -> i64 {
    let pool = db::open(data_dir).await.unwrap();
    let admin = insert_staff(&pool, "관리자", "admin", None).await.unwrap();
    sqlx::query("UPDATE staff SET pin_hash = ?")
        .bind(crate::infra::secret::hash("123456").unwrap())
        .execute(&pool)
        .await
        .unwrap();
    pool.close().await;
    admin
}

#[tokio::test]
async fn br_h2_admin_pin_cancels_without_raising_generation() {
    let dir = tempfile::tempdir().unwrap();
    let admin = handed_over_with_admin(dir.path()).await;
    let pool = db::open(dir.path()).await.unwrap();

    let checked = check_admin_pin(&pool, admin, "123456".to_owned(), NOW).await;
    pool.close().await;
    back_to_host(dir.path(), checked.ok(), NOW).await.unwrap();

    assert_eq!(run_mode::resolve(dir.path()), Some(RunMode::Host));
    let pool = db::open(dir.path()).await.unwrap();
    assert_eq!(data_bundle::read(&pool).await.unwrap().generation, 1);
    assert_eq!(
        list_for_test(&pool).await.last().unwrap(),
        &(Some(admin), "handover_canceled".to_owned())
    );
}

#[tokio::test]
async fn br_h2_wrong_pin_or_non_admin_does_not_cancel() {
    let dir = tempfile::tempdir().unwrap();
    let admin = handed_over_with_admin(dir.path()).await;
    let pool = db::open(dir.path()).await.unwrap();
    let worker = insert_staff(&pool, "직원1", "staff", None).await.unwrap();

    let wrong = check_admin_pin(&pool, admin, "000000".to_owned(), NOW).await;
    let not_admin = check_admin_pin(&pool, worker, "123456".to_owned(), NOW).await;

    assert_eq!(
        wrong,
        Err("PIN이 맞지 않습니다. 5번 연속 틀리면 5분 동안 잠깁니다".to_owned())
    );
    assert!(not_admin.is_err());
    assert_eq!(
        list_for_test(&pool).await.last().unwrap(),
        &(Some(admin), "login_failed".to_owned())
    );
}

#[test]
fn suggested_file_name_has_local_date() {
    assert!(file_name(NOW).starts_with("배움마루 넘기기 2026-11-2"));
    assert!(file_name(NOW).ends_with(".db"));
}
