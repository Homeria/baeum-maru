//! 직원 계정 저장(BR-K2, BR-K9). 마지막 관리자 확인을 트랜잭션 하나에서 하고, DB 오류를
//! 직원 저장 오류로 바꾼다.

use baeum_domain::{NewStaff, StaffProfile};
use sqlx::SqlitePool;
use time::{Date, OffsetDateTime};

use super::repository::{self, StaffRecord};
use crate::infra::db::is_unique_violation;

/// 직원 계정을 저장하지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum SaveStaffError {
    /// BR-K2: 직원 이름은 겹치지 않는다.
    #[error("같은 이름의 직원이 있습니다")]
    DuplicateName,
    /// BR-K9: 사용 중인 관리자가 하나도 남지 않게 된다.
    #[error("사용 중인 관리자가 한 명은 있어야 합니다")]
    LastAdmin,
    #[error("직원을 저장하지 못했습니다: {0}")]
    Db(sqlx::Error),
}

impl From<sqlx::Error> for SaveStaffError {
    fn from(error: sqlx::Error) -> Self {
        // staff에서 UNIQUE는 name_key 하나뿐이다
        if is_unique_violation(&error) {
            SaveStaffError::DuplicateName
        } else {
            SaveStaffError::Db(error)
        }
    }
}

/// 확인을 통과한 새 직원을 저장하고 새 `id`를 돌려준다. PIN 해시는 부르는 쪽이 만든다.
pub async fn insert(
    pool: &SqlitePool,
    staff: &NewStaff,
    pin_hash: &str,
    now: OffsetDateTime,
) -> Result<i64, SaveStaffError> {
    Ok(repository::insert(pool, staff, pin_hash, now).await?)
}

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
