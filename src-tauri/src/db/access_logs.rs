//! 접속 기록 남기기. BR-K13: 쌓기만 하고, 고치기·지우기는 DB 트리거가 막는다.

use chrono::{DateTime, Utc};
use sqlx::{SqliteConnection, SqlitePool};

use super::timestamp_text;

/// 접속 기록에 남기는 일. 입력한 PIN은 남기지 않는다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessEvent {
    /// 로그인 성공.
    Login,
    /// 로그인 실패: PIN이 틀렸거나, 계정이 잠겼거나 쓸 수 없다.
    LoginFailed,
    /// 로그아웃.
    Logout,
    /// BR-K8: 화면 잠금을 풀 때 PIN이 틀렸거나 계정이 잠겼거나 쓸 수 없다.
    UnlockFailed,
    /// BR-K12: 복구 코드로 관리자 PIN을 다시 정했다.
    RecoveryUsed,
}

impl AccessEvent {
    /// 모든 값.
    const ALL: [AccessEvent; 5] = [
        AccessEvent::Login,
        AccessEvent::LoginFailed,
        AccessEvent::Logout,
        AccessEvent::UnlockFailed,
        AccessEvent::RecoveryUsed,
    ];

    /// DB `access_logs.event`에 쓰는 값.
    fn code(self) -> &'static str {
        match self {
            AccessEvent::Login => "login",
            AccessEvent::LoginFailed => "login_failed",
            AccessEvent::Logout => "logout",
            AccessEvent::UnlockFailed => "unlock_failed",
            AccessEvent::RecoveryUsed => "recovery_used",
        }
    }

    /// DB 값에서 읽는다. 모르는 값이면 `None`.
    fn from_code(code: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|event| event.code() == code)
    }
}

/// 접속 기록 한 줄.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessLogRecord {
    pub id: i64,
    pub occurred_at: DateTime<Utc>,
    /// 고른 직원이 없는 실패면 `None`.
    pub staff_id: Option<i64>,
    pub staff_name: Option<String>,
    pub event: AccessEvent,
}

#[derive(sqlx::FromRow)]
struct AccessLogRow {
    id: i64,
    occurred_at: DateTime<Utc>,
    staff_id: Option<i64>,
    staff_name: Option<String>,
    event: String,
}

impl TryFrom<AccessLogRow> for AccessLogRecord {
    type Error = sqlx::Error;

    fn try_from(row: AccessLogRow) -> Result<Self, Self::Error> {
        let event = AccessEvent::from_code(&row.event).ok_or_else(|| {
            sqlx::Error::Decode(format!("모르는 접속 기록: {}", row.event).into())
        })?;
        Ok(AccessLogRecord {
            id: row.id,
            occurred_at: row.occurred_at,
            staff_id: row.staff_id,
            staff_name: row.staff_name,
            event,
        })
    }
}

/// 접속 기록 한 쪽과 전체 줄 수.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessLogPage {
    pub items: Vec<AccessLogRecord>,
    pub total: i64,
}

/// 접속 기록을 최근 것부터 `page`쪽(1부터) 읽는다. 한 쪽은 `page_size`줄.
pub async fn list(
    pool: &SqlitePool,
    page: u32,
    page_size: u32,
) -> Result<AccessLogPage, sqlx::Error> {
    let offset = i64::from(page.saturating_sub(1)) * i64::from(page_size);
    let rows: Vec<AccessLogRow> = sqlx::query_as(
        "SELECT a.id, a.occurred_at, a.staff_id, s.name AS staff_name, a.event
         FROM access_logs a LEFT JOIN staff s ON s.id = a.staff_id
         ORDER BY a.id DESC
         LIMIT ? OFFSET ?",
    )
    .bind(i64::from(page_size))
    .bind(offset)
    .fetch_all(pool)
    .await?;
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM access_logs")
        .fetch_one(pool)
        .await?;
    let items = rows
        .into_iter()
        .map(AccessLogRecord::try_from)
        .collect::<Result<_, _>>()?;
    Ok(AccessLogPage { items, total })
}

/// 접속 기록 한 줄을 남긴다. `staff_id`는 고른 직원이 없으면 `None`.
pub async fn record(
    pool: &SqlitePool,
    staff_id: Option<i64>,
    event: AccessEvent,
    now: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let mut conn = pool.acquire().await?;
    insert(&mut conn, staff_id, event, &timestamp_text(now)).await
}

/// 트랜잭션 안에서 다른 저장과 함께 남길 때 쓴다. `now`는 [`timestamp_text`] 모양.
pub(super) async fn insert(
    conn: &mut SqliteConnection,
    staff_id: Option<i64>,
    event: AccessEvent,
    now: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO access_logs (occurred_at, staff_id, event) VALUES (?, ?, ?)")
        .bind(now)
        .bind(staff_id)
        .bind(event.code())
        .execute(conn)
        .await?;
    Ok(())
}

/// 테스트용: 접속 기록을 쌓인 순서대로 `(staff_id, event)`로 읽는다.
#[cfg(test)]
pub(crate) async fn list_for_test(pool: &SqlitePool) -> Vec<(Option<i64>, String)> {
    sqlx::query_as("SELECT staff_id, event FROM access_logs ORDER BY id")
        .fetch_all(pool)
        .await
        .unwrap()
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::db::open_temp;
    use crate::db::test_support::insert_staff;

    #[tokio::test]
    async fn records_event_with_and_without_staff() {
        let (_dir, pool) = open_temp().await;
        let staff_id = insert_staff(&pool, "직원1", "staff", None).await.unwrap();
        let now = Utc.with_ymd_and_hms(2026, 11, 15, 1, 2, 3).unwrap();

        record(&pool, Some(staff_id), AccessEvent::Login, now)
            .await
            .unwrap();
        record(&pool, None, AccessEvent::LoginFailed, now)
            .await
            .unwrap();

        assert_eq!(
            list_for_test(&pool).await,
            [
                (Some(staff_id), "login".to_owned()),
                (None, "login_failed".to_owned()),
            ]
        );
    }

    #[tokio::test]
    async fn lists_newest_first_by_page_with_staff_name() {
        let (_dir, pool) = open_temp().await;
        let staff_id = insert_staff(&pool, "직원1", "staff", None).await.unwrap();
        let at = |minute| Utc.with_ymd_and_hms(2026, 11, 15, 1, minute, 0).unwrap();
        record(&pool, None, AccessEvent::LoginFailed, at(0))
            .await
            .unwrap();
        record(&pool, Some(staff_id), AccessEvent::Login, at(1))
            .await
            .unwrap();
        record(&pool, Some(staff_id), AccessEvent::Logout, at(2))
            .await
            .unwrap();

        let first = list(&pool, 1, 2).await.unwrap();
        let second = list(&pool, 2, 2).await.unwrap();

        assert_eq!(first.total, 3);
        let events: Vec<AccessEvent> = first.items.iter().map(|item| item.event).collect();
        assert_eq!(events, [AccessEvent::Logout, AccessEvent::Login]);
        assert_eq!(first.items[0].staff_name.as_deref(), Some("직원1"));
        assert_eq!(first.items[0].occurred_at, at(2));
        assert_eq!(
            second.items,
            [AccessLogRecord {
                id: second.items[0].id,
                occurred_at: at(0),
                staff_id: None,
                staff_name: None,
                event: AccessEvent::LoginFailed,
            }]
        );
    }
}
