//! 처음 설정 (BR-K11): 직원이 없는지 확인하고 첫 관리자와 복구 코드를 한 트랜잭션에서 저장한다.

use baeum_domain::StaffName;
use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::error::SetupError;
use super::repository;
use crate::infra::db::timestamp_text;

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

    if repository::has_staff(&mut tx).await? {
        // 저장하지 않고 끝나면 tx가 사라지며 되돌린다
        return Err(SetupError::AlreadyDone);
    }
    let admin_id = repository::insert_admin(&mut tx, admin_name, pin_hash, &now).await?;
    repository::insert_recovery_code(&mut tx, recovery_code_hash, &now).await?;

    tx.commit().await?;
    Ok(admin_id)
}
