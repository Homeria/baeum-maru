//! 접속 기록 남기기. BR-K13: 쌓기만 하고, 고치기·지우기는 DB 트리거가 막는다.

use sqlx::{SqliteConnection, SqlitePool};
use time::OffsetDateTime;

use crate::infra::db::timestamp_text;

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
    /// ADR 0011: 관리자가 백업으로 되돌렸다. 되돌린 DB에 남긴다.
    Restore,
    /// BR-K15: 관리자가 새 기기를 허락했다.
    DeviceApproved,
    /// BR-K15: 관리자가 새 기기를 거절했다.
    DeviceRejected,
    /// BR-K16: 관리자가 기기 승인을 취소했다.
    DeviceRevoked,
    /// BR-K15: 기기를 허락할 때 PIN이 틀렸거나 계정이 잠겼다.
    DeviceApprovalFailed,
    /// BR-K17: 관리자가 PIN을 넣어 어느 PC의 실행 모드 바꾸기를 허락했다.
    RunModeChanged,
    /// BR-H1: 관리자가 호스트 넘기기를 시작했다.
    Handover,
    /// BR-H2: 관리자가 넘기기를 취소하고 다시 호스트로 돌아왔다.
    HandoverCanceled,
    /// BR-H3: 관리자가 넘기기 파일로 이 PC를 새 호스트로 시작했다. 넘겨받은 DB에 남긴다.
    TakeOver,
}

impl AccessEvent {
    /// 모든 값.
    const ALL: [AccessEvent; 14] = [
        AccessEvent::Login,
        AccessEvent::LoginFailed,
        AccessEvent::Logout,
        AccessEvent::UnlockFailed,
        AccessEvent::RecoveryUsed,
        AccessEvent::Restore,
        AccessEvent::DeviceApproved,
        AccessEvent::DeviceRejected,
        AccessEvent::DeviceRevoked,
        AccessEvent::DeviceApprovalFailed,
        AccessEvent::RunModeChanged,
        AccessEvent::Handover,
        AccessEvent::HandoverCanceled,
        AccessEvent::TakeOver,
    ];

    /// DB `access_logs.event`에 쓰는 값.
    fn code(self) -> &'static str {
        match self {
            AccessEvent::Login => "login",
            AccessEvent::LoginFailed => "login_failed",
            AccessEvent::Logout => "logout",
            AccessEvent::UnlockFailed => "unlock_failed",
            AccessEvent::RecoveryUsed => "recovery_used",
            AccessEvent::Restore => "restore",
            AccessEvent::DeviceApproved => "device_approved",
            AccessEvent::DeviceRejected => "device_rejected",
            AccessEvent::DeviceRevoked => "device_revoked",
            AccessEvent::DeviceApprovalFailed => "device_approval_failed",
            AccessEvent::RunModeChanged => "run_mode_changed",
            AccessEvent::Handover => "handover",
            AccessEvent::HandoverCanceled => "handover_canceled",
            AccessEvent::TakeOver => "take_over",
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
    pub occurred_at: OffsetDateTime,
    /// 고른 직원이 없는 실패면 `None`.
    pub staff_id: Option<i64>,
    pub staff_name: Option<String>,
    /// 어느 기기의 일인지. 호스트 PC 자신이면 `None`.
    pub device_name: Option<String>,
    pub event: AccessEvent,
}

#[derive(sqlx::FromRow)]
struct AccessLogRow {
    id: i64,
    occurred_at: OffsetDateTime,
    staff_id: Option<i64>,
    staff_name: Option<String>,
    device_name: Option<String>,
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
            device_name: row.device_name,
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
        "SELECT a.id, a.occurred_at, a.staff_id, s.name AS staff_name, d.name AS device_name, a.event
         FROM access_logs a
         LEFT JOIN staff s ON s.id = a.staff_id
         LEFT JOIN devices d ON d.id = a.device_id
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
    now: OffsetDateTime,
) -> Result<(), sqlx::Error> {
    record_on_device(pool, staff_id, None, event, now).await
}

/// [`record`]와 같지만 어느 기기의 일인지도 남긴다. 호스트 PC 자신이면 `device_id`는 `None`.
pub async fn record_on_device(
    pool: &SqlitePool,
    staff_id: Option<i64>,
    device_id: Option<i64>,
    event: AccessEvent,
    now: OffsetDateTime,
) -> Result<(), sqlx::Error> {
    let mut conn = pool.acquire().await?;
    insert_on_device(&mut conn, staff_id, device_id, event, &timestamp_text(now)).await
}

/// 트랜잭션 안에서 다른 저장과 함께 남길 때 쓴다. `now`는 [`timestamp_text`] 모양.
pub(crate) async fn insert(
    conn: &mut SqliteConnection,
    staff_id: Option<i64>,
    event: AccessEvent,
    now: &str,
) -> Result<(), sqlx::Error> {
    insert_on_device(conn, staff_id, None, event, now).await
}

/// [`insert`]와 같지만 어느 기기의 일인지도 남긴다.
pub(crate) async fn insert_on_device(
    conn: &mut SqliteConnection,
    staff_id: Option<i64>,
    device_id: Option<i64>,
    event: AccessEvent,
    now: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO access_logs (occurred_at, staff_id, device_id, event) VALUES (?, ?, ?, ?)",
    )
    .bind(now)
    .bind(staff_id)
    .bind(device_id)
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
