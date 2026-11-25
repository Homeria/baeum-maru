//! 기기 승인 SQL 테스트.

use baeum_domain::{ApprovalCode, DeviceName};
use sqlx::SqlitePool;
use time::Duration;
use time::OffsetDateTime;
use time::macros::datetime;

use crate::db::test_support::insert_staff;
use crate::features::access_logs::repository::list_for_test;
use crate::features::auth::repository::find_approved;
use crate::features::devices::error::*;
use crate::features::devices::repository::*;
use crate::infra::db::open_temp;

const NOW: OffsetDateTime = datetime!(2026-11-24 01:00 UTC);

fn hash_of(byte: u8) -> [u8; 32] {
    [byte; 32]
}

async fn request_as(pool: &SqlitePool, byte: u8, at: OffsetDateTime) -> i64 {
    request(
        pool,
        &DeviceName::parse("접수 노트북").unwrap(),
        &ApprovalCode::from_random(123_456),
        &hash_of(byte),
        at,
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn approved_request_becomes_device() {
    let (_dir, pool) = open_temp().await;
    let admin = insert_staff(&pool, "관리자", "admin", None).await.unwrap();
    let id = request_as(&pool, 1, NOW).await;
    assert_eq!(
        request_status(&pool, &hash_of(1), NOW).await.unwrap(),
        Some(RequestStatus::Pending)
    );
    assert_eq!(find_approved(&pool, &hash_of(1)).await.unwrap(), None);

    let device_id = approve(&pool, id, admin, NOW).await.unwrap();

    assert_eq!(
        find_approved(&pool, &hash_of(1)).await.unwrap(),
        Some(device_id)
    );
    assert_eq!(
        request_status(&pool, &hash_of(1), NOW).await.unwrap(),
        Some(RequestStatus::Approved)
    );
    assert!(open_requests(&pool, NOW).await.unwrap().is_empty());
    let listed = list(&pool).await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].name, "접수 노트북");
    assert_eq!(listed[0].approved_by, "관리자");
    assert_eq!(
        list_for_test(&pool).await,
        [(Some(admin), "device_approved".to_owned())]
    );
}

#[tokio::test]
async fn request_closes_after_five_minutes() {
    let (_dir, pool) = open_temp().await;
    let admin = insert_staff(&pool, "관리자", "admin", None).await.unwrap();
    let id = request_as(&pool, 1, NOW).await;
    let later = NOW + Duration::minutes(5);

    assert!(open_requests(&pool, later).await.unwrap().is_empty());
    assert_eq!(
        request_status(&pool, &hash_of(1), later).await.unwrap(),
        Some(RequestStatus::Expired)
    );
    assert!(matches!(
        approve(&pool, id, admin, later).await,
        Err(DecideError::NotOpen)
    ));
    assert_eq!(find_approved(&pool, &hash_of(1)).await.unwrap(), None);
}

#[tokio::test]
async fn rejected_request_cannot_be_approved() {
    let (_dir, pool) = open_temp().await;
    let admin = insert_staff(&pool, "관리자", "admin", None).await.unwrap();
    let id = request_as(&pool, 1, NOW).await;

    reject(&pool, id, admin, NOW).await.unwrap();

    assert_eq!(
        request_status(&pool, &hash_of(1), NOW).await.unwrap(),
        Some(RequestStatus::Rejected)
    );
    assert!(matches!(
        approve(&pool, id, admin, NOW).await,
        Err(DecideError::NotOpen)
    ));
    assert!(matches!(
        reject(&pool, id, admin, NOW).await,
        Err(DecideError::NotOpen)
    ));
    assert_eq!(
        list_for_test(&pool).await,
        [(Some(admin), "device_rejected".to_owned())]
    );
}

#[tokio::test]
async fn too_many_open_requests_are_refused() {
    let (_dir, pool) = open_temp().await;
    for byte in 1..=5 {
        request_as(&pool, byte, NOW).await;
    }

    let refused = request(
        &pool,
        &DeviceName::parse("여섯째").unwrap(),
        &ApprovalCode::from_random(1),
        &hash_of(6),
        NOW,
    )
    .await;

    assert!(matches!(refused, Err(RequestError::TooMany)));
    // 5분이 지난 요청은 세지 않는다
    request_as(&pool, 7, NOW + Duration::minutes(5)).await;
}

#[tokio::test]
async fn revoked_device_is_no_longer_approved() {
    let (_dir, pool) = open_temp().await;
    let admin = insert_staff(&pool, "관리자", "admin", None).await.unwrap();
    let id = request_as(&pool, 1, NOW).await;
    let device_id = approve(&pool, id, admin, NOW).await.unwrap();

    assert!(revoke(&pool, device_id, admin, NOW).await.unwrap());

    assert_eq!(find_approved(&pool, &hash_of(1)).await.unwrap(), None);
    assert!(list(&pool).await.unwrap().is_empty());
    assert!(!revoke(&pool, device_id, admin, NOW).await.unwrap());
    assert_eq!(
        list_for_test(&pool).await,
        [
            (Some(admin), "device_approved".to_owned()),
            (Some(admin), "device_revoked".to_owned()),
        ]
    );
}
