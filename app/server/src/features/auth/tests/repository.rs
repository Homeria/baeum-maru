//! 로그인 계정, 세션, 복구 코드 SQL 테스트.

use baeum_domain::Role;
use sqlx::SqlitePool;
use time::OffsetDateTime;
use time::macros::{date, datetime};

use crate::db::staff::setup;
use crate::db::test_support::insert_staff;
use crate::features::access_logs::repository::{AccessEvent, list_for_test};
use crate::features::auth::repository::*;
use crate::infra::db::{open_temp, timestamp_text};

fn now() -> OffsetDateTime {
    datetime!(2026-11-15 01:02:03 UTC)
}

#[tokio::test]
async fn login_list_hides_disabled_and_expired_staff() {
    let (_dir, pool) = open_temp().await;
    let kept = insert_staff(&pool, "나직원", "staff", None).await.unwrap();
    let last_day = insert_staff(&pool, "가임시", "temp_staff", Some("2026-11-15"))
        .await
        .unwrap();
    insert_staff(&pool, "다임시", "temp_staff", Some("2026-11-14"))
        .await
        .unwrap();
    let disabled = insert_staff(&pool, "라직원", "staff", None).await.unwrap();
    sqlx::query("UPDATE staff SET disabled = 1 WHERE id = ?")
        .bind(disabled)
        .execute(&pool)
        .await
        .unwrap();

    let choices = list_for_login(&pool, date!(2026 - 11 - 15)).await.unwrap();

    assert_eq!(
        choices,
        [
            LoginChoice {
                id: last_day,
                name: "가임시".to_owned(),
                role: Role::TempStaff,
            },
            LoginChoice {
                id: kept,
                name: "나직원".to_owned(),
                role: Role::Staff,
            },
        ]
    );
}

#[tokio::test]
async fn reads_login_account() {
    let (_dir, pool) = open_temp().await;
    let id = insert_staff(&pool, "임시1", "temp_staff", Some("2027-02-28"))
        .await
        .unwrap();

    let account = get_login_account(&pool, id).await.unwrap().unwrap();

    assert_eq!(
        account,
        LoginAccount {
            id,
            name: "임시1".to_owned(),
            role: Role::TempStaff,
            pin_hash: "hash".to_owned(),
            valid_until: Some(date!(2027 - 02 - 28)),
            disabled: false,
            locked_until: None,
        }
    );
    assert_eq!(get_login_account(&pool, id + 1).await.unwrap(), None);
}

#[tokio::test]
async fn fifth_failed_pin_locks_account_and_every_failure_is_logged() {
    let (_dir, pool) = open_temp().await;
    let id = insert_staff(&pool, "직원1", "staff", None).await.unwrap();

    for expected in 1..=4 {
        let failed = add_failed_pin(&pool, id, None, AccessEvent::LoginFailed, now())
            .await
            .unwrap();
        assert_eq!(failed.failed_count, expected);
        assert_eq!(failed.locked_until, None);
    }
    let failed = add_failed_pin(&pool, id, None, AccessEvent::UnlockFailed, now())
        .await
        .unwrap();

    let unlock_at = now() + baeum_domain::ACCOUNT_LOCK;
    assert_eq!(failed.locked_until, Some(unlock_at));
    let account = get_login_account(&pool, id).await.unwrap().unwrap();
    assert_eq!(account.locked_until, Some(unlock_at));
    let events: Vec<String> = list_for_test(&pool)
        .await
        .into_iter()
        .map(|(_, event)| event)
        .collect();
    assert_eq!(
        events,
        [
            "login_failed",
            "login_failed",
            "login_failed",
            "login_failed",
            "unlock_failed"
        ]
    );
}

fn at(minute: u8) -> OffsetDateTime {
    datetime!(2026-11-15 01:00 UTC)
        .replace_minute(minute)
        .unwrap()
}

async fn session_hashes(pool: &SqlitePool) -> Vec<Vec<u8>> {
    sqlx::query_scalar("SELECT token_hash FROM sessions ORDER BY id")
        .fetch_all(pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn start_saves_session_resets_failures_and_logs() {
    let (_dir, pool) = open_temp().await;
    let staff_id = insert_staff(&pool, "직원1", "staff", None).await.unwrap();
    sqlx::query("UPDATE staff SET failed_pin_count = 3, locked_until = ? WHERE id = ?")
        .bind(timestamp_text(at(0)))
        .bind(staff_id)
        .execute(&pool)
        .await
        .unwrap();

    start(&pool, staff_id, None, &[1; 32], at(10))
        .await
        .unwrap();

    assert_eq!(session_hashes(&pool).await, [vec![1; 32]]);
    let (count, locked_until): (i64, Option<String>) =
        sqlx::query_as("SELECT failed_pin_count, locked_until FROM staff WHERE id = ?")
            .bind(staff_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((count, locked_until), (0, None));
    assert_eq!(
        list_for_test(&pool).await,
        [(Some(staff_id), "login".to_owned())]
    );
}

#[tokio::test]
async fn start_clears_only_that_staffs_locked_sessions() {
    let (_dir, pool) = open_temp().await;
    let staff_id = insert_staff(&pool, "직원1", "staff", None).await.unwrap();
    let other_id = insert_staff(&pool, "직원2", "staff", None).await.unwrap();
    // 1시 0분: 잠길 세션, 1시 5분: 아직 안 잠긴 세션, 다른 직원의 1시 0분 세션
    start(&pool, staff_id, None, &[1; 32], at(0)).await.unwrap();
    start(&pool, staff_id, None, &[2; 32], at(5)).await.unwrap();
    start(&pool, other_id, None, &[3; 32], at(0)).await.unwrap();

    start(&pool, staff_id, None, &[4; 32], at(10))
        .await
        .unwrap();

    assert_eq!(
        session_hashes(&pool).await,
        [vec![2; 32], vec![3; 32], vec![4; 32]]
    );
}

#[tokio::test]
async fn end_deletes_session_and_logs_logout() {
    let (_dir, pool) = open_temp().await;
    let staff_id = insert_staff(&pool, "직원1", "staff", None).await.unwrap();
    start(&pool, staff_id, None, &[1; 32], at(0)).await.unwrap();

    assert_eq!(end(&pool, &[1; 32], at(1)).await.unwrap(), Some(staff_id));

    assert!(session_hashes(&pool).await.is_empty());
    assert_eq!(
        list_for_test(&pool).await,
        [
            (Some(staff_id), "login".to_owned()),
            (Some(staff_id), "logout".to_owned()),
        ]
    );
}

#[tokio::test]
async fn finds_session_with_account_and_touches_it() {
    let (_dir, pool) = open_temp().await;
    let staff_id = insert_staff(&pool, "임시1", "temp_staff", Some("2027-02-28"))
        .await
        .unwrap();
    start(&pool, staff_id, None, &[1; 32], at(0)).await.unwrap();

    let session = find(&pool, &[1; 32]).await.unwrap().unwrap();
    assert_eq!(session.last_seen_at, at(0));
    assert_eq!(session.account.id, staff_id);
    assert_eq!(session.account.role, baeum_domain::Role::TempStaff);
    assert_eq!(find(&pool, &[2; 32]).await.unwrap(), None);

    touch(&pool, session.id, at(3)).await.unwrap();

    let session = find(&pool, &[1; 32]).await.unwrap().unwrap();
    assert_eq!(session.last_seen_at, at(3));
}

#[tokio::test]
async fn unlock_reopens_session_and_resets_failures() {
    let (_dir, pool) = open_temp().await;
    let staff_id = insert_staff(&pool, "직원1", "staff", None).await.unwrap();
    start(&pool, staff_id, None, &[1; 32], at(0)).await.unwrap();
    sqlx::query("UPDATE staff SET failed_pin_count = 2 WHERE id = ?")
        .bind(staff_id)
        .execute(&pool)
        .await
        .unwrap();
    let session = find(&pool, &[1; 32]).await.unwrap().unwrap();

    unlock(&pool, session.id, staff_id, at(30)).await.unwrap();

    let session = find(&pool, &[1; 32]).await.unwrap().unwrap();
    assert_eq!(session.last_seen_at, at(30));
    let count: i64 = sqlx::query_scalar("SELECT failed_pin_count FROM staff")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn end_without_session_changes_nothing() {
    let (_dir, pool) = open_temp().await;

    assert_eq!(end(&pool, &[9; 32], at(0)).await.unwrap(), None);

    assert!(list_for_test(&pool).await.is_empty());
}

#[tokio::test]
async fn no_code_before_setup() {
    let (_dir, pool) = open_temp().await;

    assert_eq!(code_hash(&pool).await.unwrap(), None);
}

#[tokio::test]
async fn using_code_replaces_code_and_pin_once() {
    let (_dir, pool) = open_temp().await;
    let name = baeum_domain::StaffName::parse("관리자").unwrap();
    let admin = setup(&pool, &name, "pin-1", "code-1", now()).await.unwrap();
    sqlx::query("UPDATE staff SET failed_pin_count = 3")
        .execute(&pool)
        .await
        .unwrap();

    assert!(
        use_code(&pool, admin, "code-1", "code-2", "pin-2", now())
            .await
            .unwrap()
    );
    // 같은 코드를 또 쓰면 아무것도 바뀌지 않는다
    assert!(
        !use_code(&pool, admin, "code-1", "code-3", "pin-3", now())
            .await
            .unwrap()
    );

    assert_eq!(code_hash(&pool).await.unwrap().as_deref(), Some("code-2"));
    let account = get_login_account(&pool, admin).await.unwrap().unwrap();
    assert_eq!(account.pin_hash, "pin-2");
    let count: i64 = sqlx::query_scalar("SELECT failed_pin_count FROM staff")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
    assert_eq!(
        list_for_test(&pool).await,
        [(Some(admin), "recovery_used".to_owned())]
    );
}
