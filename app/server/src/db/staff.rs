//! 직원 계정과 처음 설정 저장. 로그인에 쓰는 계정 읽기는 `features::auth::repository`에 있다.

use baeum_domain::{NewStaff, Role, StaffName, StaffProfile};
use sqlx::SqlitePool;
use time::{Date, OffsetDateTime};

use crate::features::auth::repository::decode_role;
use crate::infra::db::{is_unique_violation, timestamp_text};

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
    use crate::db::test_support::insert_staff;
    use crate::features::access_logs::repository::AccessEvent;
    use crate::features::auth::repository::{add_failed_pin, get_login_account};

    use crate::infra::db::open_temp;

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
