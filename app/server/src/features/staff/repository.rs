//! 직원 계정 저장. 로그인에 쓰는 계정 읽기는 `features::auth::repository`, 첫 관리자 만들기는
//! `features::setup::repository`에 있다.

use baeum_domain::{NewStaff, Role, StaffProfile};
use sqlx::{SqliteConnection, SqlitePool};
use time::{Date, OffsetDateTime};

use super::error::SaveStaffError;
use crate::features::auth::repository::decode_role;
use crate::infra::db::timestamp_text;

/// 직원 관리 화면에 보여 주는 직원 하나. PIN 해시는 담지 않는다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffRecord {
    pub id: i64,
    pub name: String,
    pub role: Role,
    pub valid_until: Option<Date>,
    pub disabled: bool,
    pub locked_until: Option<OffsetDateTime>,
}

#[derive(sqlx::FromRow)]
pub(super) struct StaffRow {
    id: i64,
    name: String,
    role: String,
    valid_until: Option<Date>,
    disabled: bool,
    locked_until: Option<OffsetDateTime>,
}

impl TryFrom<StaffRow> for StaffRecord {
    type Error = sqlx::Error;

    fn try_from(row: StaffRow) -> Result<Self, Self::Error> {
        Ok(StaffRecord {
            id: row.id,
            name: row.name,
            role: decode_role(&row.role)?,
            valid_until: row.valid_until,
            disabled: row.disabled,
            locked_until: row.locked_until,
        })
    }
}

/// 직원을 모두 읽는다. 사용 중인 직원 먼저, 이름 순서.
pub async fn list(pool: &SqlitePool) -> Result<Vec<StaffRecord>, sqlx::Error> {
    let rows: Vec<StaffRow> = sqlx::query_as(
        "SELECT id, name, role, valid_until, disabled, locked_until FROM staff
         ORDER BY disabled, name, id",
    )
    .fetch_all(pool)
    .await?;
    rows.into_iter().map(StaffRecord::try_from).collect()
}

/// `id`인 직원을 읽는다. 없으면 `None`.
pub async fn get(pool: &SqlitePool, id: i64) -> Result<Option<StaffRecord>, sqlx::Error> {
    let row: Option<StaffRow> = sqlx::query_as(
        "SELECT id, name, role, valid_until, disabled, locked_until FROM staff WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    row.map(StaffRecord::try_from).transpose()
}

/// 확인을 통과한 새 직원을 저장하고 새 `id`를 돌려준다. PIN 해시는 부르는 쪽이 만든다.
/// 이름이 겹치면 [`SaveStaffError::DuplicateName`] (BR-K2).
pub async fn insert(
    pool: &SqlitePool,
    staff: &NewStaff,
    pin_hash: &str,
    now: OffsetDateTime,
) -> Result<i64, SaveStaffError> {
    let profile = staff.profile();
    let now = timestamp_text(now);
    let result = sqlx::query(
        "INSERT INTO staff (name, name_key, pin_hash, role, valid_until, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(profile.name().as_str())
    .bind(profile.name().key())
    .bind(pin_hash)
    .bind(profile.role().code())
    .bind(profile.valid_until())
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

/// `id`인 직원의 이름·역할·유효기간·사용 중지를 바꾸고 바꾼 뒤 줄을 돌려준다. 없으면 `None`.
pub(super) async fn update_row(
    conn: &mut SqliteConnection,
    id: i64,
    profile: &StaffProfile,
    disabled: bool,
    now: OffsetDateTime,
) -> Result<Option<StaffRow>, sqlx::Error> {
    sqlx::query_as(
        "UPDATE staff SET name = ?, name_key = ?, role = ?, valid_until = ?, disabled = ?,
                          updated_at = ?
         WHERE id = ?
         RETURNING id, name, role, valid_until, disabled, locked_until",
    )
    .bind(profile.name().as_str())
    .bind(profile.name().key())
    .bind(profile.role().code())
    .bind(profile.valid_until())
    .bind(disabled)
    .bind(timestamp_text(now))
    .bind(id)
    .fetch_optional(conn)
    .await
}

/// `today`에 사용 중인 관리자가 하나라도 있는지(BR-K9).
pub(super) async fn has_active_admin(
    conn: &mut SqliteConnection,
    today: Date,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT EXISTS (
             SELECT 1 FROM staff
             WHERE role = 'admin' AND disabled = 0
               AND (valid_until IS NULL OR valid_until >= ?)
         )",
    )
    .bind(today)
    .fetch_one(conn)
    .await
}

/// BR-K10: `id`인 직원의 PIN을 새 해시로 바꾼다. 틀린 횟수와 계정 잠금도 푼다.
/// 바꿨으면 `true`, 없으면 `false`.
pub async fn reset_pin(
    pool: &SqlitePool,
    id: i64,
    pin_hash: &str,
    now: OffsetDateTime,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE staff SET pin_hash = ?, failed_pin_count = 0, locked_until = NULL, updated_at = ?
         WHERE id = ?",
    )
    .bind(pin_hash)
    .bind(timestamp_text(now))
    .bind(id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}
