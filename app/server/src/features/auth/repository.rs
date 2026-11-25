//! 로그인 계정, PIN 실패 기록, 세션, 복구 코드의 SQL (BR-K5~K8, BR-K11~K13). 트랜잭션은 `service`가 연다.
//!
//! ADR 0014: 세션 증표 자체는 두지 않고 SHA-256 해시만 둔다. 복구 코드도 해시만 둔다.

use baeum_domain::{FailedPin, Role, is_valid_on};
use sqlx::{SqliteConnection, SqlitePool};
use time::{Date, OffsetDateTime};

use crate::infra::clock::local_today;
use crate::infra::db::timestamp_text;

// 로그인 계정

/// 로그인 화면에서 고를 수 있는 직원 하나. 화면은 역할별로 묶어 보여 준다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginChoice {
    pub id: i64,
    pub name: String,
    pub role: Role,
}

#[derive(sqlx::FromRow)]
struct LoginChoiceRow {
    id: i64,
    name: String,
    role: String,
}

impl TryFrom<LoginChoiceRow> for LoginChoice {
    type Error = sqlx::Error;

    fn try_from(row: LoginChoiceRow) -> Result<Self, Self::Error> {
        Ok(LoginChoice {
            id: row.id,
            name: row.name,
            role: decode_role(&row.role)?,
        })
    }
}

/// 로그인 화면에 보여 줄 직원을 이름 순서로 읽는다.
///
/// 사용 중지(BR-K6)했거나 `today`에 유효기간이 지난(BR-K5) 계정은 뺀다.
/// 잠긴 계정(BR-K7)은 넣는다. 고르면 잠겼다고 알려 준다.
pub async fn list_for_login(
    pool: &SqlitePool,
    today: Date,
) -> Result<Vec<LoginChoice>, sqlx::Error> {
    let rows: Vec<LoginChoiceRow> = sqlx::query_as(
        "SELECT id, name, role FROM staff
         WHERE disabled = 0 AND (valid_until IS NULL OR valid_until >= ?)
         ORDER BY name, id",
    )
    .bind(today)
    .fetch_all(pool)
    .await?;
    rows.into_iter().map(LoginChoice::try_from).collect()
}

/// 로그인할 때 확인하는 직원 계정.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginAccount {
    pub id: i64,
    pub name: String,
    pub role: Role,
    /// argon2id 해시. PIN 자체가 아니다.
    pub pin_hash: String,
    pub valid_until: Option<Date>,
    pub disabled: bool,
    /// 잠갔으면 풀리는 시각. 지난 시각이면 이미 풀렸다.
    pub locked_until: Option<OffsetDateTime>,
}

impl LoginAccount {
    /// BR-K5, BR-K6: 사용 중지하지 않았고 유효기간이 남은 계정인지.
    /// 유효기간은 기관의 달력 날짜라 이 PC의 날짜로 본다.
    pub fn is_usable(&self) -> bool {
        !self.disabled && is_valid_on(self.valid_until, local_today())
    }
}

/// DB에서 읽은 그대로의 한 줄. 역할은 글자다.
#[derive(sqlx::FromRow)]
struct LoginAccountRow {
    id: i64,
    name: String,
    role: String,
    pin_hash: String,
    valid_until: Option<Date>,
    disabled: bool,
    locked_until: Option<OffsetDateTime>,
}

impl TryFrom<LoginAccountRow> for LoginAccount {
    type Error = sqlx::Error;

    fn try_from(row: LoginAccountRow) -> Result<Self, Self::Error> {
        let role = decode_role(&row.role)?;
        Ok(LoginAccount {
            id: row.id,
            name: row.name,
            role,
            pin_hash: row.pin_hash,
            valid_until: row.valid_until,
            disabled: row.disabled,
            locked_until: row.locked_until,
        })
    }
}

/// `id`인 직원 계정을 읽는다. 없으면 `None`.
pub async fn get_login_account(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<LoginAccount>, sqlx::Error> {
    let row: Option<LoginAccountRow> = sqlx::query_as(
        "SELECT id, name, role, pin_hash, valid_until, disabled, locked_until
         FROM staff WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    row.map(LoginAccount::try_from).transpose()
}

/// 직원의 PIN 틀린 횟수.
pub(super) async fn failed_pin_count(
    conn: &mut SqliteConnection,
    staff_id: i64,
) -> Result<u32, sqlx::Error> {
    sqlx::query_scalar("SELECT failed_pin_count FROM staff WHERE id = ?")
        .bind(staff_id)
        .fetch_one(conn)
        .await
}

/// 틀린 횟수와 잠금을 `failed`로 바꾼다(BR-K7).
pub(super) async fn set_failed_pin(
    conn: &mut SqliteConnection,
    staff_id: i64,
    failed: &FailedPin,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE staff SET failed_pin_count = ?, locked_until = ? WHERE id = ?")
        .bind(failed.failed_count)
        .bind(failed.locked_until.map(timestamp_text))
        .bind(staff_id)
        .execute(conn)
        .await?;
    Ok(())
}

/// 틀린 횟수와 잠금을 푼다(BR-K7).
pub(super) async fn clear_failed_pin(
    conn: &mut SqliteConnection,
    staff_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE staff SET failed_pin_count = 0, locked_until = NULL WHERE id = ?")
        .bind(staff_id)
        .execute(conn)
        .await?;
    Ok(())
}

/// DB의 역할 글자를 [`Role`]로 읽는다. 마이그레이션의 CHECK가 막아서 틀릴 일은 없다.
pub(crate) fn decode_role(code: &str) -> Result<Role, sqlx::Error> {
    Role::from_code(code).ok_or_else(|| sqlx::Error::Decode(format!("모르는 역할: {code}").into()))
}

// 세션

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
    let mut conn = pool.acquire().await?;
    set_last_seen(&mut conn, session_id, now).await
}

/// 세션의 마지막 요청 시각을 `now`로 바꾼다.
pub(super) async fn set_last_seen(
    conn: &mut SqliteConnection,
    session_id: i64,
    now: OffsetDateTime,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE sessions SET last_seen_at = ? WHERE id = ?")
        .bind(timestamp_text(now))
        .bind(session_id)
        .execute(conn)
        .await?;
    Ok(())
}

/// 세션을 하나 저장한다. `now`는 [`timestamp_text`] 글자.
pub(super) async fn insert_session(
    conn: &mut SqliteConnection,
    staff_id: i64,
    token_hash: &[u8; 32],
    now: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO sessions (staff_id, token_hash, started_at, last_seen_at)
         VALUES (?, ?, ?, ?)",
    )
    .bind(staff_id)
    .bind(token_hash.as_slice())
    .bind(now)
    .bind(now)
    .execute(conn)
    .await?;
    Ok(())
}

/// 직원의 세션 가운데 마지막 요청이 `locked_before` 이전인(잠긴) 것을 지운다.
pub(super) async fn delete_locked_sessions(
    conn: &mut SqliteConnection,
    staff_id: i64,
    locked_before: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM sessions WHERE staff_id = ? AND last_seen_at <= ?")
        .bind(staff_id)
        .bind(locked_before)
        .execute(conn)
        .await?;
    Ok(())
}

/// 증표 해시가 `token_hash`인 세션을 지우고 그 직원 id를 돌려준다. 없으면 `None`.
pub(super) async fn delete_session(
    conn: &mut SqliteConnection,
    token_hash: &[u8; 32],
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar("DELETE FROM sessions WHERE token_hash = ? RETURNING staff_id")
        .bind(token_hash.as_slice())
        .fetch_optional(conn)
        .await
}

// 복구 코드

/// 저장된 복구 코드의 해시. 처음 설정 전이면 `None`.
pub async fn code_hash(pool: &SqlitePool) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT code_hash FROM recovery_code WHERE id = 1")
        .fetch_optional(pool)
        .await
}

/// 복구 코드 해시를 `used_code_hash`에서 `new_code_hash`로 바꾼다. 그사이 다른 요청이 먼저
/// 바꿨으면 바뀌는 줄이 없어 `false`. `now`는 [`timestamp_text`] 글자.
pub(super) async fn replace_code(
    conn: &mut SqliteConnection,
    used_code_hash: &str,
    new_code_hash: &str,
    now: &str,
) -> Result<bool, sqlx::Error> {
    let replaced = sqlx::query(
        "UPDATE recovery_code SET code_hash = ?, created_at = ? WHERE id = 1 AND code_hash = ?",
    )
    .bind(new_code_hash)
    .bind(now)
    .bind(used_code_hash)
    .execute(conn)
    .await?;
    Ok(replaced.rows_affected() > 0)
}

/// 관리자 PIN을 바꾸고 틀린 횟수와 잠금(BR-K7)을 푼다. `now`는 [`timestamp_text`] 글자.
pub(super) async fn set_admin_pin(
    conn: &mut SqliteConnection,
    staff_id: i64,
    new_pin_hash: &str,
    now: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE staff SET pin_hash = ?, failed_pin_count = 0, locked_until = NULL, updated_at = ?
         WHERE id = ?",
    )
    .bind(new_pin_hash)
    .bind(now)
    .bind(staff_id)
    .execute(conn)
    .await?;
    Ok(())
}

// 기기 증표

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
