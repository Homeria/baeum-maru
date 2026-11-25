//! 복구 코드 읽기와 쓰기 (BR-K11, BR-K12). DB에는 코드의 해시만 둔다.

use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::access_logs::{self, AccessEvent};
use crate::infra::db::timestamp_text;

/// 저장된 복구 코드의 해시. 처음 설정 전이면 `None`.
pub async fn code_hash(pool: &SqlitePool) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT code_hash FROM recovery_code WHERE id = 1")
        .fetch_optional(pool)
        .await
}

/// BR-K12: 복구 코드를 써서 `staff_id`인 관리자의 PIN을 다시 정하고 코드를 새것으로 바꾼다.
///
/// 한 트랜잭션에서 다음을 함께 한다.
/// - 복구 코드 해시를 `used_code_hash`에서 `new_code_hash`로 바꾼다. 그사이 다른 요청이
///   먼저 썼으면 바뀌는 줄이 없어서 아무것도 하지 않고 `false`를 돌려준다.
/// - 관리자 PIN을 바꾸고 틀린 횟수와 잠금(BR-K7)을 푼다.
/// - 접속 기록에 복구 코드 사용을 남긴다 (BR-K13).
pub async fn use_code(
    pool: &SqlitePool,
    staff_id: i64,
    used_code_hash: &str,
    new_code_hash: &str,
    new_pin_hash: &str,
    now: OffsetDateTime,
) -> Result<bool, sqlx::Error> {
    let now = timestamp_text(now);
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

    let replaced = sqlx::query(
        "UPDATE recovery_code SET code_hash = ?, created_at = ? WHERE id = 1 AND code_hash = ?",
    )
    .bind(new_code_hash)
    .bind(&now)
    .bind(used_code_hash)
    .execute(&mut *tx)
    .await?;
    if replaced.rows_affected() == 0 {
        return Ok(false);
    }

    sqlx::query(
        "UPDATE staff SET pin_hash = ?, failed_pin_count = 0, locked_until = NULL, updated_at = ?
         WHERE id = ?",
    )
    .bind(new_pin_hash)
    .bind(&now)
    .bind(staff_id)
    .execute(&mut *tx)
    .await?;
    access_logs::insert(&mut tx, Some(staff_id), AccessEvent::RecoveryUsed, &now).await?;

    tx.commit().await?;
    Ok(true)
}

#[cfg(test)]
mod tests {

    use time::macros::datetime;

    use super::*;
    use crate::db::access_logs::list_for_test;
    use crate::db::staff::{get_login_account, setup};
    use crate::infra::db::open_temp;

    fn now() -> OffsetDateTime {
        datetime!(2026-11-15 01:02:03 UTC)
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
}
