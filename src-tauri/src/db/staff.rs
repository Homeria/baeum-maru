//! 직원 계정과 복구 코드 저장.

use baeum_domain::{
    FailedPin, NewStaff, Role, StaffName, StaffProfile, is_valid_on, record_failed_pin,
};
use sqlx::SqlitePool;
use time::{Date, OffsetDateTime};

use super::access_logs::{self, AccessEvent};
use super::{is_unique_violation, timestamp_text};
use crate::clock::local_today;

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
pub(super) struct LoginAccountRow {
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

/// BR-K7: `staff_id`인 직원이 PIN을 틀렸다. 틀린 횟수를 세고(5번째면 잠그고)
/// 접속 기록에 `event`(로그인 실패나 잠금 풀기 실패)를 남긴 뒤, 바뀐 상태를 돌려준다.
///
/// 동시에 틀려도 횟수를 빠뜨리지 않게 읽기와 쓰기를 `BEGIN IMMEDIATE` 하나에서 한다.
pub async fn add_failed_pin(
    pool: &SqlitePool,
    staff_id: i64,
    device_id: Option<i64>,
    event: AccessEvent,
    now: OffsetDateTime,
) -> Result<FailedPin, sqlx::Error> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

    let failed_count: u32 = sqlx::query_scalar("SELECT failed_pin_count FROM staff WHERE id = ?")
        .bind(staff_id)
        .fetch_one(&mut *tx)
        .await?;
    let failed = record_failed_pin(failed_count, now);

    sqlx::query("UPDATE staff SET failed_pin_count = ?, locked_until = ? WHERE id = ?")
        .bind(failed.failed_count)
        .bind(failed.locked_until.map(timestamp_text))
        .bind(staff_id)
        .execute(&mut *tx)
        .await?;
    access_logs::insert_on_device(
        &mut tx,
        Some(staff_id),
        device_id,
        event,
        &timestamp_text(now),
    )
    .await?;

    tx.commit().await?;
    Ok(failed)
}

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
struct StaffRow {
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

/// DB의 역할 글자를 [`Role`]로 읽는다. 마이그레이션의 CHECK가 막아서 틀릴 일은 없다.
fn decode_role(code: &str) -> Result<Role, sqlx::Error> {
    Role::from_code(code).ok_or_else(|| sqlx::Error::Decode(format!("모르는 역할: {code}").into()))
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

    let row: Option<StaffRow> = sqlx::query_as(
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
    .fetch_optional(&mut *tx)
    .await?;
    let Some(row) = row else {
        return Ok(None);
    };

    let has_admin: bool = sqlx::query_scalar(
        "SELECT EXISTS (
             SELECT 1 FROM staff
             WHERE role = 'admin' AND disabled = 0
               AND (valid_until IS NULL OR valid_until >= ?)
         )",
    )
    .bind(today)
    .fetch_one(&mut *tx)
    .await?;
    if !has_admin {
        // tx를 저장하지 않고 버려 바꾼 것을 되돌린다
        return Err(SaveStaffError::LastAdmin);
    }

    tx.commit().await?;
    Ok(Some(StaffRecord::try_from(row)?))
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

#[cfg(test)]
mod tests {

    use time::macros::{date, datetime};

    use super::*;
    use crate::db::access_logs::list_for_test;
    use crate::db::open_temp;
    use crate::db::test_support::insert_staff;

    fn now() -> OffsetDateTime {
        datetime!(2026-11-15 01:02:03 UTC)
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

    #[tokio::test]
    async fn login_list_hides_disabled_and_expired_staff() {
        let (_dir, pool) = open_temp().await;
        let kept = insert_staff(&pool, "나직원", "staff", None).await.unwrap();
        let last_day = insert_staff(&pool, "가임시", "temp_staff", Some("2026-11-15"))
            .await
            .unwrap();
        insert_staff(&pool, "다임시", "temp_staff", Some("2026-11-14"))
            .await
            .unwrap();
        let disabled = insert_staff(&pool, "라직원", "staff", None).await.unwrap();
        sqlx::query("UPDATE staff SET disabled = 1 WHERE id = ?")
            .bind(disabled)
            .execute(&pool)
            .await
            .unwrap();

        let choices = list_for_login(&pool, date!(2026 - 11 - 15)).await.unwrap();

        assert_eq!(
            choices,
            [
                LoginChoice {
                    id: last_day,
                    name: "가임시".to_owned(),
                    role: Role::TempStaff,
                },
                LoginChoice {
                    id: kept,
                    name: "나직원".to_owned(),
                    role: Role::Staff,
                },
            ]
        );
    }

    #[tokio::test]
    async fn reads_login_account() {
        let (_dir, pool) = open_temp().await;
        let id = insert_staff(&pool, "임시1", "temp_staff", Some("2027-02-28"))
            .await
            .unwrap();

        let account = get_login_account(&pool, id).await.unwrap().unwrap();

        assert_eq!(
            account,
            LoginAccount {
                id,
                name: "임시1".to_owned(),
                role: Role::TempStaff,
                pin_hash: "hash".to_owned(),
                valid_until: Some(date!(2027 - 02 - 28)),
                disabled: false,
                locked_until: None,
            }
        );
        assert_eq!(get_login_account(&pool, id + 1).await.unwrap(), None);
    }

    #[tokio::test]
    async fn fifth_failed_pin_locks_account_and_every_failure_is_logged() {
        let (_dir, pool) = open_temp().await;
        let id = insert_staff(&pool, "직원1", "staff", None).await.unwrap();

        for expected in 1..=4 {
            let failed = add_failed_pin(&pool, id, None, AccessEvent::LoginFailed, now())
                .await
                .unwrap();
            assert_eq!(failed.failed_count, expected);
            assert_eq!(failed.locked_until, None);
        }
        let failed = add_failed_pin(&pool, id, None, AccessEvent::UnlockFailed, now())
            .await
            .unwrap();

        let unlock_at = now() + baeum_domain::ACCOUNT_LOCK;
        assert_eq!(failed.locked_until, Some(unlock_at));
        let account = get_login_account(&pool, id).await.unwrap().unwrap();
        assert_eq!(account.locked_until, Some(unlock_at));
        let events: Vec<String> = list_for_test(&pool)
            .await
            .into_iter()
            .map(|(_, event)| event)
            .collect();
        assert_eq!(
            events,
            [
                "login_failed",
                "login_failed",
                "login_failed",
                "login_failed",
                "unlock_failed"
            ]
        );
    }

    fn profile(name: &str, role: Role, valid_until: Option<Date>) -> StaffProfile {
        StaffProfile::parse(baeum_domain::StaffInput {
            name,
            role,
            valid_until,
        })
        .unwrap()
    }

    fn new_staff(name: &str, role: Role) -> NewStaff {
        let input = baeum_domain::StaffInput {
            name,
            role,
            valid_until: None,
        };
        NewStaff::parse(input, "123456").unwrap()
    }

    #[tokio::test]
    async fn saves_and_lists_staff_active_first() {
        let (_dir, pool) = open_temp().await;
        let admin = insert(&pool, &new_staff("관리자", Role::Admin), "h1", now())
            .await
            .unwrap();
        let staff = insert(&pool, &new_staff("가직원", Role::Staff), "h2", now())
            .await
            .unwrap();
        update(
            &pool,
            staff,
            &profile("가직원", Role::Staff, None),
            true,
            date!(2026 - 11 - 15),
            now(),
        )
        .await
        .unwrap();

        let records = list(&pool).await.unwrap();

        let ids: Vec<i64> = records.iter().map(|record| record.id).collect();
        assert_eq!(ids, [admin, staff]);
        assert_eq!(
            get(&pool, staff).await.unwrap(),
            Some(StaffRecord {
                id: staff,
                name: "가직원".to_owned(),
                role: Role::Staff,
                valid_until: None,
                disabled: true,
                locked_until: None,
            })
        );
    }

    #[tokio::test]
    async fn rejects_name_differing_only_in_spaces() {
        let (_dir, pool) = open_temp().await;
        insert(&pool, &new_staff("김 직원", Role::Staff), "h", now())
            .await
            .unwrap();

        let result = insert(&pool, &new_staff("김직원", Role::Staff), "h", now()).await;

        assert!(
            matches!(result, Err(SaveStaffError::DuplicateName)),
            "{result:?}"
        );
    }

    #[tokio::test]
    async fn last_usable_admin_cannot_be_disabled_demoted_or_expired() {
        let (_dir, pool) = open_temp().await;
        let admin = insert(&pool, &new_staff("관리자", Role::Admin), "h", now())
            .await
            .unwrap();
        let today = date!(2026 - 11 - 15);
        let changes = [
            (profile("관리자", Role::Admin, None), true),
            (profile("관리자", Role::Staff, None), false),
            (
                profile("관리자", Role::Admin, Some(date!(2026 - 11 - 14))),
                false,
            ),
        ];

        for (change, disabled) in &changes {
            let result = update(&pool, admin, change, *disabled, today, now()).await;
            assert!(
                matches!(result, Err(SaveStaffError::LastAdmin)),
                "{result:?}"
            );
        }
        let record = get(&pool, admin).await.unwrap().unwrap();
        assert_eq!((record.role, record.disabled), (Role::Admin, false));

        // 다른 관리자가 있으면 바꿀 수 있다
        insert(&pool, &new_staff("관리자2", Role::Admin), "h", now())
            .await
            .unwrap();
        let (change, disabled) = &changes[0];
        let record = update(&pool, admin, change, *disabled, today, now())
            .await
            .unwrap()
            .unwrap();
        assert!(record.disabled);
    }

    #[tokio::test]
    async fn update_of_missing_staff_is_none() {
        let (_dir, pool) = open_temp().await;
        insert(&pool, &new_staff("관리자", Role::Admin), "h", now())
            .await
            .unwrap();

        let result = update(
            &pool,
            999,
            &profile("없음", Role::Staff, None),
            false,
            date!(2026 - 11 - 15),
            now(),
        )
        .await
        .unwrap();

        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn reset_pin_replaces_hash_and_unlocks() {
        let (_dir, pool) = open_temp().await;
        let id = insert_staff(&pool, "직원1", "staff", None).await.unwrap();
        for _ in 0..5 {
            add_failed_pin(&pool, id, None, AccessEvent::LoginFailed, now())
                .await
                .unwrap();
        }

        assert!(reset_pin(&pool, id, "new-hash", now()).await.unwrap());

        let account = get_login_account(&pool, id).await.unwrap().unwrap();
        assert_eq!(account.pin_hash, "new-hash");
        assert_eq!(account.locked_until, None);
        assert!(!reset_pin(&pool, 999, "h", now()).await.unwrap());
    }
}
