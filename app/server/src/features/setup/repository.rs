//! 처음 설정의 SQL: 첫 관리자와 복구 코드 (BR-K11). 트랜잭션은 `service`가 연다.

use baeum_domain::{Role, StaffName};
use sqlx::{SqliteConnection, SqlitePool};

/// 처음 설정이 필요한지. 직원이 하나도 없으면 `true`.
pub async fn is_setup_needed(pool: &SqlitePool) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT NOT EXISTS (SELECT 1 FROM staff)")
        .fetch_one(pool)
        .await
}

/// 직원이 하나라도 있는지.
pub(super) async fn has_staff(conn: &mut SqliteConnection) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM staff)")
        .fetch_one(conn)
        .await
}

/// 첫 관리자를 저장하고 `id`를 돌려준다. `now`는 [`timestamp_text`] 글자.
pub(super) async fn insert_admin(
    conn: &mut SqliteConnection,
    admin_name: &StaffName,
    pin_hash: &str,
    now: &str,
) -> Result<i64, sqlx::Error> {
    Ok(sqlx::query(
        "INSERT INTO staff (name, name_key, pin_hash, role, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(admin_name.as_str())
    .bind(admin_name.key())
    .bind(pin_hash)
    .bind(Role::Admin.code())
    .bind(now)
    .bind(now)
    .execute(conn)
    .await?
    .last_insert_rowid())
}

/// 복구 코드 해시를 처음 저장한다. `now`는 [`timestamp_text`] 글자.
pub(super) async fn insert_recovery_code(
    conn: &mut SqliteConnection,
    code_hash: &str,
    now: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO recovery_code (id, code_hash, created_at) VALUES (1, ?, ?)")
        .bind(code_hash)
        .bind(now)
        .execute(conn)
        .await?;
    Ok(())
}
