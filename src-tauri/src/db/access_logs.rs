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
}

impl AccessEvent {
    /// DB `access_logs.event`에 쓰는 값.
    fn code(self) -> &'static str {
        match self {
            AccessEvent::Login => "login",
            AccessEvent::LoginFailed => "login_failed",
            AccessEvent::Logout => "logout",
        }
    }
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
}
