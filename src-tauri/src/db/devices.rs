//! 기기 승인 요청과 승인한 기기 (BR-K14~K16). 증표 자체는 두지 않고 SHA-256 해시만 둔다.

use baeum_domain::{APPROVAL_WINDOW, ApprovalCode, DeviceName, is_approval_open};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::access_logs::{self, AccessEvent};
use super::timestamp_text;

/// 한 번에 기다릴 수 있는 승인 요청 수. 같은 Wi-Fi의 누군가가 요청을 쏟아내도 호스트 화면이
/// 요청으로 덮이지 않게 한다.
pub const MAX_OPEN_REQUESTS: i64 = 5;

/// 승인 요청을 받지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum RequestError {
    /// 기다리는 요청이 이미 [`MAX_OPEN_REQUESTS`]개다.
    #[error("기다리는 승인 요청이 너무 많습니다")]
    TooMany,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

/// 승인 요청을 받는다. 요청한 기기는 증표를 이미 받았고, 허락되어야 쓸 수 있다.
pub async fn request(
    pool: &SqlitePool,
    name: &DeviceName,
    code: &ApprovalCode,
    token_hash: &[u8; 32],
    now: OffsetDateTime,
) -> Result<i64, RequestError> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let open: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM device_requests WHERE status = 'pending' AND requested_at > ?",
    )
    .bind(timestamp_text(now - APPROVAL_WINDOW))
    .fetch_one(&mut *tx)
    .await?;
    if open >= MAX_OPEN_REQUESTS {
        return Err(RequestError::TooMany);
    }
    let id = sqlx::query(
        "INSERT INTO device_requests (name, code, token_hash, requested_at) VALUES (?, ?, ?, ?)",
    )
    .bind(name.as_str())
    .bind(code.as_str())
    .bind(token_hash.as_slice())
    .bind(timestamp_text(now))
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();
    tx.commit().await?;
    Ok(id)
}

/// 요청한 기기가 보는 승인 요청의 상태.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestStatus {
    /// 관리자를 기다린다.
    Pending,
    /// 허락했다. 증표를 쓸 수 있다.
    Approved,
    /// 거절했다.
    Rejected,
    /// 5분 안에 허락하지 않았다.
    Expired,
}

/// 증표 해시가 `token_hash`인 승인 요청의 상태. 없으면 `None`.
pub async fn request_status(
    pool: &SqlitePool,
    token_hash: &[u8; 32],
    now: OffsetDateTime,
) -> Result<Option<RequestStatus>, sqlx::Error> {
    let row: Option<(String, OffsetDateTime)> =
        sqlx::query_as("SELECT status, requested_at FROM device_requests WHERE token_hash = ?")
            .bind(token_hash.as_slice())
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|(status, requested_at)| match status.as_str() {
        "approved" => RequestStatus::Approved,
        "rejected" => RequestStatus::Rejected,
        _ if is_approval_open(requested_at, now) => RequestStatus::Pending,
        _ => RequestStatus::Expired,
    }))
}

/// 호스트 화면에 보여 줄, 기다리는 승인 요청.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct OpenRequest {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub requested_at: OffsetDateTime,
}

/// 5분이 지나지 않은 기다리는 요청을 먼저 온 것부터 읽는다.
pub async fn open_requests(
    pool: &SqlitePool,
    now: OffsetDateTime,
) -> Result<Vec<OpenRequest>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, name, code, requested_at FROM device_requests
         WHERE status = 'pending' AND requested_at > ?
         ORDER BY id",
    )
    .bind(timestamp_text(now - APPROVAL_WINDOW))
    .fetch_all(pool)
    .await
}

/// 승인 요청을 허락하거나 거절하지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum DecideError {
    /// 그런 요청이 없거나, 이미 허락·거절했거나, 5분이 지났다.
    #[error("기다리는 승인 요청이 없습니다")]
    NotOpen,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

/// 기다리는 요청을 허락해 기기를 만들고 접속 기록을 남긴다. 만든 기기의 id를 돌려준다.
///
/// 허락한 관리자는 PIN을 맞게 넣었으므로 BR-K7의 틀린 횟수도 0으로 돌린다.
pub async fn approve(
    pool: &SqlitePool,
    request_id: i64,
    staff_id: i64,
    now: OffsetDateTime,
) -> Result<i64, DecideError> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let now_text = timestamp_text(now);
    let Some((name, token_hash)): Option<(String, Vec<u8>)> = sqlx::query_as(
        "SELECT name, token_hash FROM device_requests
         WHERE id = ? AND status = 'pending' AND requested_at > ?",
    )
    .bind(request_id)
    .bind(timestamp_text(now - APPROVAL_WINDOW))
    .fetch_optional(&mut *tx)
    .await?
    else {
        return Err(DecideError::NotOpen);
    };
    let device_id = sqlx::query(
        "INSERT INTO devices (name, token_hash, approved_at, approved_by) VALUES (?, ?, ?, ?)",
    )
    .bind(&name)
    .bind(&token_hash)
    .bind(&now_text)
    .bind(staff_id)
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();
    sqlx::query("UPDATE staff SET failed_pin_count = 0, locked_until = NULL WHERE id = ?")
        .bind(staff_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE device_requests SET status = 'approved', device_id = ? WHERE id = ?")
        .bind(device_id)
        .bind(request_id)
        .execute(&mut *tx)
        .await?;
    access_logs::insert_on_device(
        &mut tx,
        Some(staff_id),
        Some(device_id),
        AccessEvent::DeviceApproved,
        &now_text,
    )
    .await?;
    tx.commit().await?;
    Ok(device_id)
}

/// 기다리는 요청을 거절하고 접속 기록을 남긴다. 5분이 지난 요청도 거절할 수 있다.
pub async fn reject(
    pool: &SqlitePool,
    request_id: i64,
    staff_id: i64,
    now: OffsetDateTime,
) -> Result<(), DecideError> {
    let mut tx = pool.begin().await?;
    let changed = sqlx::query(
        "UPDATE device_requests SET status = 'rejected' WHERE id = ? AND status = 'pending'",
    )
    .bind(request_id)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if changed == 0 {
        return Err(DecideError::NotOpen);
    }
    access_logs::insert(
        &mut tx,
        Some(staff_id),
        AccessEvent::DeviceRejected,
        &timestamp_text(now),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

/// 승인한 기기 한 줄.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct DeviceRecord {
    pub id: i64,
    pub name: String,
    pub approved_at: OffsetDateTime,
    /// 허락한 직원의 이름.
    pub approved_by: String,
}

/// 승인을 취소하지 않은 기기를 허락한 순서대로 읽는다.
pub async fn list(pool: &SqlitePool) -> Result<Vec<DeviceRecord>, sqlx::Error> {
    sqlx::query_as(
        "SELECT d.id, d.name, d.approved_at, s.name AS approved_by
         FROM devices d JOIN staff s ON s.id = d.approved_by
         WHERE d.revoked_at IS NULL
         ORDER BY d.id",
    )
    .fetch_all(pool)
    .await
}

/// BR-K16: 기기 승인을 취소하고 접속 기록을 남긴다. 그런 기기가 없거나 이미 취소했으면 `false`.
pub async fn revoke(
    pool: &SqlitePool,
    device_id: i64,
    staff_id: i64,
    now: OffsetDateTime,
) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let now_text = timestamp_text(now);
    let changed =
        sqlx::query("UPDATE devices SET revoked_at = ? WHERE id = ? AND revoked_at IS NULL")
            .bind(&now_text)
            .bind(device_id)
            .execute(&mut *tx)
            .await?
            .rows_affected();
    if changed == 0 {
        return Ok(false);
    }
    access_logs::insert_on_device(
        &mut tx,
        Some(staff_id),
        Some(device_id),
        AccessEvent::DeviceRevoked,
        &now_text,
    )
    .await?;
    tx.commit().await?;
    Ok(true)
}

/// BR-K14, BR-K16: 증표 해시가 `token_hash`이고 승인을 취소하지 않은 기기의 id. 없으면 `None`.
pub async fn find_approved(
    pool: &SqlitePool,
    token_hash: &[u8; 32],
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar("SELECT id FROM devices WHERE token_hash = ? AND revoked_at IS NULL")
        .bind(token_hash.as_slice())
        .fetch_optional(pool)
        .await
}

#[cfg(test)]
mod tests {
    use time::Duration;
    use time::macros::datetime;

    use super::*;
    use crate::db::access_logs::list_for_test;
    use crate::db::open_temp;
    use crate::db::test_support::insert_staff;

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
}
