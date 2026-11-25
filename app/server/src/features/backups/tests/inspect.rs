//! 백업 검사 테스트.

use std::path::Path;

use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{ConnectOptions, Connection};

use time::macros::datetime;

use crate::db::test_support::{insert_member, insert_term};
use crate::features::backups::service::inspect::*;
use crate::infra::backup_file::name::BackupKind;
use crate::infra::backup_file::store;
use crate::infra::db::open_temp;

async fn backup_of(pool: &sqlx::SqlitePool, dir: &Path) -> std::path::PathBuf {
    let name = store::create(pool, dir, BackupKind::Manual, datetime!(2026-11-23 09:00))
        .await
        .unwrap();
    dir.join(name.to_string())
}

#[tokio::test]
async fn counts_terms_members_and_applications() {
    let (dir, pool) = open_temp().await;
    insert_term(&pool, "2027년 1학기", "2027-03-01", "2027-06-30")
        .await
        .unwrap();
    insert_term(&pool, "2026년 2학기", "2026-09-01", "2026-12-31")
        .await
        .unwrap();
    insert_member(&pool, "00001").await.unwrap();
    insert_member(&pool, "00002").await.unwrap();
    let file = backup_of(&pool, &dir.path().join("backups")).await;

    let preview = inspect(&file).await.unwrap();

    assert_eq!(
        preview,
        BackupPreview {
            term_names: vec!["2026년 2학기".to_owned(), "2027년 1학기".to_owned()],
            member_count: 2,
            application_count: 0,
        }
    );
    // 살펴보기만 하고 보조 파일을 남기지 않는다
    let mut entries = tokio::fs::read_dir(dir.path().join("backups"))
        .await
        .unwrap();
    let mut count = 0;
    while entries.next_entry().await.unwrap().is_some() {
        count += 1;
    }
    assert_eq!(count, 1);
}

#[tokio::test]
async fn refuses_files_that_are_not_baeum_maru_backups() {
    let dir = tempfile::tempdir().unwrap();
    let text = dir.path().join("메모.db");
    tokio::fs::write(&text, "SQLite가 아닌 글자 파일입니다. ".repeat(100))
        .await
        .unwrap();
    let other = dir.path().join("다른 프로그램.db");
    let mut conn = SqliteConnectOptions::new()
        .filename(&other)
        .create_if_missing(true)
        .connect()
        .await
        .unwrap();
    sqlx::query("CREATE TABLE notes (body TEXT)")
        .execute(&mut conn)
        .await
        .unwrap();
    conn.close().await.unwrap();

    assert!(matches!(inspect(&text).await, Err(InspectError::Broken(_))));
    assert!(matches!(
        inspect(&other).await,
        Err(InspectError::NotBaeumMaru)
    ));
    assert!(matches!(
        inspect(&dir.path().join("없는 파일.db")).await,
        Err(InspectError::Broken(_))
    ));
}

#[tokio::test]
async fn refuses_backups_from_newer_app() {
    let (dir, pool) = open_temp().await;
    // 이 앱이 모르는 구조 변경이 적용된 DB
    sqlx::query(
        "INSERT INTO _sqlx_migrations (version, description, installed_on, success, checksum, execution_time)
         VALUES (99991231000000, 'from newer app', '2099-01-01', 1, x'00', 0)",
    )
    .execute(&pool)
    .await
    .unwrap();
    let file = backup_of(&pool, &dir.path().join("backups")).await;

    assert!(matches!(
        inspect(&file).await,
        Err(InspectError::NewerVersion)
    ));
}
