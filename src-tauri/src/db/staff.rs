//! 직원 계정과 복구 코드 저장.

use baeum_domain::{Role, StaffName};
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

use super::timestamp_text;

/// 처음 설정을 하지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum SetupError {
    /// BR-K11: 처음 설정은 직원이 하나도 없을 때 한 번만 한다.
    #[error("이미 처음 설정을 마쳤습니다")]
    AlreadyDone,
    #[error("처음 설정을 저장하지 못했습니다: {0}")]
    Db(#[from] sqlx::Error),
}

/// 처음 설정이 필요한지. 직원이 하나도 없으면 `true`.
pub async fn is_setup_needed(pool: &SqlitePool) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT NOT EXISTS (SELECT 1 FROM staff)")
        .fetch_one(pool)
        .await
}

/// BR-K11: 첫 관리자와 복구 코드를 함께 저장하고 관리자의 `id`를 돌려준다.
///
/// 해시는 부르는 쪽이 만든다. 두 화면에서 동시에 설정해도 하나만 성공하도록
/// 확인과 저장을 `BEGIN IMMEDIATE` 트랜잭션 하나에서 한다 (ADR 0005).
pub async fn setup(
    pool: &SqlitePool,
    admin_name: &StaffName,
    pin_hash: &str,
    recovery_code_hash: &str,
    now: DateTime<Utc>,
) -> Result<i64, SetupError> {
    let now = timestamp_text(now);
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

    let has_staff: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM staff)")
        .fetch_one(&mut *tx)
        .await?;
    if has_staff {
        // 저장하지 않고 끝나면 tx가 사라지며 되돌린다
        return Err(SetupError::AlreadyDone);
    }

    let admin_id = sqlx::query(
        "INSERT INTO staff (name, name_key, pin_hash, role, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(admin_name.as_str())
    .bind(admin_name.key())
    .bind(pin_hash)
    .bind(Role::Admin.code())
    .bind(&now)
    .bind(&now)
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();

    sqlx::query("INSERT INTO recovery_code (id, code_hash, created_at) VALUES (1, ?, ?)")
        .bind(recovery_code_hash)
        .bind(&now)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(admin_id)
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::db::open_temp;
    use crate::db::test_support::insert_staff;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 15, 1, 2, 3).unwrap()
    }

    fn name(input: &str) -> StaffName {
        StaffName::parse(input).unwrap()
    }

    #[tokio::test]
    async fn setup_is_needed_only_until_first_staff() {
        let (_dir, pool) = open_temp().await;
        assert!(is_setup_needed(&pool).await.unwrap());

        insert_staff(&pool, "직원1", "staff", None).await.unwrap();

        assert!(!is_setup_needed(&pool).await.unwrap());
    }

    #[tokio::test]
    async fn setup_saves_admin_and_recovery_code() {
        let (_dir, pool) = open_temp().await;

        let id = setup(&pool, &name(" 관리 자 "), "pin-hash", "code-hash", now())
            .await
            .unwrap();

        let (staff_name, name_key, pin_hash, role, valid_until): (
            String,
            String,
            String,
            String,
            Option<String>,
        ) = sqlx::query_as(
            "SELECT name, name_key, pin_hash, role, valid_until FROM staff WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(staff_name, "관리 자");
        assert_eq!(name_key, "관리자");
        assert_eq!(pin_hash, "pin-hash");
        assert_eq!(role, "admin");
        assert_eq!(valid_until, None);

        let code_hash: String = sqlx::query_scalar("SELECT code_hash FROM recovery_code")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(code_hash, "code-hash");
        assert!(!is_setup_needed(&pool).await.unwrap());
    }

    #[tokio::test]
    async fn setup_runs_only_once() {
        let (_dir, pool) = open_temp().await;
        setup(&pool, &name("관리자"), "pin-hash", "code-hash", now())
            .await
            .unwrap();

        let result = setup(&pool, &name("다른관리자"), "pin-2", "code-2", now()).await;

        assert!(matches!(result, Err(SetupError::AlreadyDone)), "{result:?}");
        let staff_count: i64 = sqlx::query_scalar("SELECT count(*) FROM staff")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(staff_count, 1);
        let code_hash: String = sqlx::query_scalar("SELECT code_hash FROM recovery_code")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(code_hash, "code-hash");
    }
}
