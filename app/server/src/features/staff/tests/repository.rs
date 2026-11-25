//! 직원 SQL 테스트.

use baeum_domain::{NewStaff, Role, StaffProfile};
use time::macros::{date, datetime};
use time::{Date, OffsetDateTime};

use crate::db::test_support::insert_staff;
use crate::features::access_logs::repository::AccessEvent;
use crate::features::auth::repository::get_login_account;
use crate::features::auth::service::add_failed_pin;
use crate::features::staff::repository::{StaffRecord, get, list, reset_pin};
use crate::features::staff::service::*;
use crate::infra::db::open_temp;

fn now() -> OffsetDateTime {
    datetime!(2026-11-15 01:02:03 UTC)
}

fn profile(name: &str, role: Role, valid_until: Option<Date>) -> StaffProfile {
    StaffProfile::parse(baeum_domain::StaffInput {
        name,
        role,
        valid_until,
    })
    .unwrap()
}

fn new_staff(name: &str, role: Role) -> NewStaff {
    let input = baeum_domain::StaffInput {
        name,
        role,
        valid_until: None,
    };
    NewStaff::parse(input, "123456").unwrap()
}

#[tokio::test]
async fn saves_and_lists_staff_active_first() {
    let (_dir, pool) = open_temp().await;
    let admin = insert(&pool, &new_staff("관리자", Role::Admin), "h1", now())
        .await
        .unwrap();
    let staff = insert(&pool, &new_staff("가직원", Role::Staff), "h2", now())
        .await
        .unwrap();
    update(
        &pool,
        staff,
        &profile("가직원", Role::Staff, None),
        true,
        date!(2026 - 11 - 15),
        now(),
    )
    .await
    .unwrap();

    let records = list(&pool).await.unwrap();

    let ids: Vec<i64> = records.iter().map(|record| record.id).collect();
    assert_eq!(ids, [admin, staff]);
    assert_eq!(
        get(&pool, staff).await.unwrap(),
        Some(StaffRecord {
            id: staff,
            name: "가직원".to_owned(),
            role: Role::Staff,
            valid_until: None,
            disabled: true,
            locked_until: None,
        })
    );
}

#[tokio::test]
async fn rejects_name_differing_only_in_spaces() {
    let (_dir, pool) = open_temp().await;
    insert(&pool, &new_staff("김 직원", Role::Staff), "h", now())
        .await
        .unwrap();

    let result = insert(&pool, &new_staff("김직원", Role::Staff), "h", now()).await;

    assert!(
        matches!(result, Err(SaveStaffError::DuplicateName)),
        "{result:?}"
    );
}

#[tokio::test]
async fn last_usable_admin_cannot_be_disabled_demoted_or_expired() {
    let (_dir, pool) = open_temp().await;
    let admin = insert(&pool, &new_staff("관리자", Role::Admin), "h", now())
        .await
        .unwrap();
    let today = date!(2026 - 11 - 15);
    let changes = [
        (profile("관리자", Role::Admin, None), true),
        (profile("관리자", Role::Staff, None), false),
        (
            profile("관리자", Role::Admin, Some(date!(2026 - 11 - 14))),
            false,
        ),
    ];

    for (change, disabled) in &changes {
        let result = update(&pool, admin, change, *disabled, today, now()).await;
        assert!(
            matches!(result, Err(SaveStaffError::LastAdmin)),
            "{result:?}"
        );
    }
    let record = get(&pool, admin).await.unwrap().unwrap();
    assert_eq!((record.role, record.disabled), (Role::Admin, false));

    // 다른 관리자가 있으면 바꿀 수 있다
    insert(&pool, &new_staff("관리자2", Role::Admin), "h", now())
        .await
        .unwrap();
    let (change, disabled) = &changes[0];
    let record = update(&pool, admin, change, *disabled, today, now())
        .await
        .unwrap()
        .unwrap();
    assert!(record.disabled);
}

#[tokio::test]
async fn update_of_missing_staff_is_none() {
    let (_dir, pool) = open_temp().await;
    insert(&pool, &new_staff("관리자", Role::Admin), "h", now())
        .await
        .unwrap();

    let result = update(
        &pool,
        999,
        &profile("없음", Role::Staff, None),
        false,
        date!(2026 - 11 - 15),
        now(),
    )
    .await
    .unwrap();

    assert_eq!(result, None);
}

#[tokio::test]
async fn reset_pin_replaces_hash_and_unlocks() {
    let (_dir, pool) = open_temp().await;
    let id = insert_staff(&pool, "직원1", "staff", None).await.unwrap();
    for _ in 0..5 {
        add_failed_pin(&pool, id, None, AccessEvent::LoginFailed, now())
            .await
            .unwrap();
    }

    assert!(reset_pin(&pool, id, "new-hash", now()).await.unwrap());

    let account = get_login_account(&pool, id).await.unwrap().unwrap();
    assert_eq!(account.pin_hash, "new-hash");
    assert_eq!(account.locked_until, None);
    assert!(!reset_pin(&pool, 999, "h", now()).await.unwrap());
}
