//! 직원 계정 바꾸기(BR-K9). 마지막 관리자 확인을 트랜잭션 하나에서 한다.

use baeum_domain::StaffProfile;
use sqlx::SqlitePool;
use time::{Date, OffsetDateTime};

use super::error::SaveStaffError;
use super::repository::{self, StaffRecord};

/// `id`인 직원의 이름·역할·유효기간·사용 중지를 바꾸고 바꾼 뒤 모습을 돌려준다. 없으면 `None`.
///
/// BR-K9: 바꾼 뒤 `today`에 사용 중인 관리자가 하나도 없으면 되돌리고
/// [`SaveStaffError::LastAdmin`]. 두 관리자를 동시에 막지 못하게 `BEGIN IMMEDIATE`로 한다.
pub async fn update(
    pool: &SqlitePool,
    id: i64,
    profile: &StaffProfile,
    disabled: bool,
    today: Date,
    now: OffsetDateTime,
) -> Result<Option<StaffRecord>, SaveStaffError> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

    let Some(row) = repository::update_row(&mut tx, id, profile, disabled, now).await? else {
        return Ok(None);
    };
    if !repository::has_active_admin(&mut tx, today).await? {
        // tx를 저장하지 않고 버려 바꾼 것을 되돌린다
        return Err(SaveStaffError::LastAdmin);
    }

    tx.commit().await?;
    Ok(Some(StaffRecord::try_from(row)?))
}
