//! 로그인 세션 시작과 끝. ADR 0014: 증표 자체는 두지 않고 SHA-256 해시만 둔다.

use baeum_domain::SCREEN_LOCK;
use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::staff::{LoginAccount, LoginAccountRow};
use crate::features::access_logs::repository::{self as access_logs, AccessEvent};
use crate::infra::db::timestamp_text;

/// 요청에 실린 증표의 세션과 그 직원 계정. 요청마다 함께 읽는다 (ADR 0014).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRecord {
    pub id: i64,
    pub last_seen_at: OffsetDateTime,
    pub account: LoginAccount,
}

#[derive(sqlx::FromRow)]
struct SessionRow {
    session_id: i64,
    last_seen_at: OffsetDateTime,
    #[sqlx(flatten)]
    account: LoginAccountRow,
}

/// 증표 해시가 `token_hash`인 세션을 직원 계정과 함께 읽는다. 없으면 `None`.
///
/// 잠겼는지, 계정을 쓸 수 있는지는 부르는 쪽이 판단한다.
pub async fn find(
    pool: &SqlitePool,
    token_hash: &[u8; 32],
) -> Result<Option<SessionRecord>, sqlx::Error> {
    let row: Option<SessionRow> = sqlx::query_as(
        "SELECT se.id AS session_id, se.last_seen_at,
                s.id, s.name, s.role, s.pin_hash, s.valid_until, s.disabled, s.locked_until
         FROM sessions se JOIN staff s ON s.id = se.staff_id
         WHERE se.token_hash = ?",
    )
    .bind(token_hash.as_slice())
    .fetch_optional(pool)
    .await?;
    row.map(|row| {
        Ok(SessionRecord {
            id: row.session_id,
            last_seen_at: row.last_seen_at,
            account: LoginAccount::try_from(row.account)?,
        })
    })
    .transpose()
}

/// BR-K8: 세션의 마지막 요청 시각을 `now`로 바꾼다. 잠기지 않은 세션에만 부른다.
pub async fn touch(
    pool: &SqlitePool,
    session_id: i64,
    now: OffsetDateTime,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE sessions SET last_seen_at = ? WHERE id = ?")
        .bind(timestamp_text(now))
        .bind(session_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// BR-K8: PIN을 맞게 넣어 잠긴 세션을 연다.
/// BR-K7: 틀린 횟수도 0으로 돌린다 (로그인에서 맞게 넣은 것과 같다).
pub async fn unlock(
    pool: &SqlitePool,
    session_id: i64,
    staff_id: i64,
    now: OffsetDateTime,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE staff SET failed_pin_count = 0, locked_until = NULL WHERE id = ?")
        .bind(staff_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE sessions SET last_seen_at = ? WHERE id = ?")
        .bind(timestamp_text(now))
        .bind(session_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await
}

/// `staff_id`인 직원의 로그인을 기록한다.
///
/// 한 트랜잭션에서 다음을 함께 한다.
/// - BR-K7: 틀린 횟수와 잠금을 푼다.
/// - ADR 0014: 그 직원의 잠긴(마지막 요청에서 10분 지난) 세션을 지워 정리한다.
/// - 새 세션을 저장하고 접속 기록에 로그인을 남긴다.
pub async fn start(
    pool: &SqlitePool,
    staff_id: i64,
    device_id: Option<i64>,
    token_hash: &[u8; 32],
    now: OffsetDateTime,
) -> Result<(), sqlx::Error> {
    let locked_before = timestamp_text(now - SCREEN_LOCK);
    let now = timestamp_text(now);
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

    sqlx::query("UPDATE staff SET failed_pin_count = 0, locked_until = NULL WHERE id = ?")
        .bind(staff_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM sessions WHERE staff_id = ? AND last_seen_at <= ?")
        .bind(staff_id)
        .bind(&locked_before)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "INSERT INTO sessions (staff_id, token_hash, started_at, last_seen_at)
         VALUES (?, ?, ?, ?)",
    )
    .bind(staff_id)
    .bind(token_hash.as_slice())
    .bind(&now)
    .bind(&now)
    .execute(&mut *tx)
    .await?;
    access_logs::insert_on_device(&mut tx, Some(staff_id), device_id, AccessEvent::Login, &now)
        .await?;

    tx.commit().await
}

/// 증표 해시가 `token_hash`인 세션을 지우고 접속 기록에 로그아웃을 남긴 뒤
/// 그 세션의 직원 id를 돌려준다. 그런 세션이 없으면 아무것도 하지 않고 `None`.
///
/// 잠긴 세션도 지운다. BR-K8: 다른 직원은 로그아웃한 뒤 로그인한다.
pub async fn end(
    pool: &SqlitePool,
    token_hash: &[u8; 32],
    now: OffsetDateTime,
) -> Result<Option<i64>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let staff_id: Option<i64> =
        sqlx::query_scalar("DELETE FROM sessions WHERE token_hash = ? RETURNING staff_id")
            .bind(token_hash.as_slice())
            .fetch_optional(&mut *tx)
            .await?;
    let Some(staff_id) = staff_id else {
        return Ok(None);
    };
    access_logs::insert(
        &mut tx,
        Some(staff_id),
        AccessEvent::Logout,
        &timestamp_text(now),
    )
    .await?;

    tx.commit().await?;
    Ok(Some(staff_id))
}

#[cfg(test)]
mod tests {

    use time::macros::datetime;

    use super::*;
    use crate::db::test_support::insert_staff;
    use crate::features::access_logs::repository::list_for_test;
    use crate::infra::db::open_temp;

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
}
