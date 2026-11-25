//! 처음 설정 저장: 첫 관리자와 복구 코드 (BR-K11).

use baeum_domain::{Role, StaffName};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use crate::infra::db::timestamp_text;

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
    now: OffsetDateTime,
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
