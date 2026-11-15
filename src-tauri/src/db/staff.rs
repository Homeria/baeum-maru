//! 직원 계정과 복구 코드 저장.

use baeum_domain::{FailedPin, Role, StaffName, record_failed_pin};
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::SqlitePool;

use super::access_logs::{self, AccessEvent};
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

/// 로그인 화면에서 고를 수 있는 직원 하나.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct LoginChoice {
    pub id: i64,
    pub name: String,
}

/// 로그인 화면에 보여 줄 직원을 이름 순서로 읽는다.
///
/// 사용 중지(BR-K6)했거나 `today`에 유효기간이 지난(BR-K5) 계정은 뺀다.
/// 잠긴 계정(BR-K7)은 넣는다. 고르면 잠겼다고 알려 준다.
pub async fn list_for_login(
    pool: &SqlitePool,
    today: NaiveDate,
) -> Result<Vec<LoginChoice>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, name FROM staff
         WHERE disabled = 0 AND (valid_until IS NULL OR valid_until >= ?)
         ORDER BY name, id",
    )
    .bind(today)
    .fetch_all(pool)
    .await
}

/// 로그인할 때 확인하는 직원 계정.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginAccount {
    pub id: i64,
    pub name: String,
    pub role: Role,
    /// argon2id 해시. PIN 자체가 아니다.
    pub pin_hash: String,
    pub valid_until: Option<NaiveDate>,
    pub disabled: bool,
    /// 잠갔으면 풀리는 시각. 지난 시각이면 이미 풀렸다.
    pub locked_until: Option<DateTime<Utc>>,
}

/// DB에서 읽은 그대로의 한 줄. 역할은 글자다.
#[derive(sqlx::FromRow)]
struct LoginAccountRow {
    id: i64,
    name: String,
    role: String,
    pin_hash: String,
    valid_until: Option<NaiveDate>,
    disabled: bool,
    locked_until: Option<DateTime<Utc>>,
}

impl TryFrom<LoginAccountRow> for LoginAccount {
    type Error = sqlx::Error;

    fn try_from(row: LoginAccountRow) -> Result<Self, Self::Error> {
        let role = Role::from_code(&row.role)
            .ok_or_else(|| sqlx::Error::Decode(format!("모르는 역할: {}", row.role).into()))?;
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
/// 접속 기록에 로그인 실패를 남긴 뒤, 바뀐 상태를 돌려준다.
///
/// 동시에 틀려도 횟수를 빠뜨리지 않게 읽기와 쓰기를 `BEGIN IMMEDIATE` 하나에서 한다.
pub async fn add_failed_pin(
    pool: &SqlitePool,
    staff_id: i64,
    now: DateTime<Utc>,
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
    access_logs::insert(
        &mut tx,
        Some(staff_id),
        AccessEvent::LoginFailed,
        &timestamp_text(now),
    )
    .await?;

    tx.commit().await?;
    Ok(failed)
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::db::access_logs::list_for_test;
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

    fn date(text: &str) -> NaiveDate {
        text.parse().unwrap()
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

        let choices = list_for_login(&pool, date("2026-11-15")).await.unwrap();

        let ids: Vec<i64> = choices.iter().map(|choice| choice.id).collect();
        assert_eq!(ids, [last_day, kept]);
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
                valid_until: Some(date("2027-02-28")),
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
            let failed = add_failed_pin(&pool, id, now()).await.unwrap();
            assert_eq!(failed.failed_count, expected);
            assert_eq!(failed.locked_until, None);
        }
        let failed = add_failed_pin(&pool, id, now()).await.unwrap();

        let unlock_at = now() + baeum_domain::ACCOUNT_LOCK;
        assert_eq!(failed.locked_until, Some(unlock_at));
        let account = get_login_account(&pool, id).await.unwrap().unwrap();
        assert_eq!(account.locked_until, Some(unlock_at));
        let logs = list_for_test(&pool).await;
        assert_eq!(logs.len(), 5);
        assert!(
            logs.iter()
                .all(|log| *log == (Some(id), "login_failed".to_owned()))
        );
    }
}
