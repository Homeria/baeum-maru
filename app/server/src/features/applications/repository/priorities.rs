//! 신청에 우선 배정 사유를 붙이고 떼기(BR-S12, BR-S13)와 변경 기록(BR-S14).
//!
//! 누가 붙일 수 있는지(관리자·직원)는 서버가 확인한다.

use std::collections::BTreeSet;

use sqlx::{SqliteConnection, SqlitePool};
use time::OffsetDateTime;

use crate::features::lotteries::repository::cancel_active_run;
use crate::infra::db::timestamp_text;

/// 사유를 바꾸지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum SetPrioritiesError {
    #[error("신청을 찾지 못했습니다")]
    NotFound,
    /// 유효한 신청(접수됨, 당첨, 대기)이 아니다. 확정은 마감 뒤라 여기 오지 않는다.
    #[error("우선 배정을 바꿀 수 없는 신청입니다")]
    NotChangeable,
    /// BR-S13: 선발 마감 뒤에는 바꿀 수 없다.
    #[error("선발을 마감한 강좌입니다")]
    Closed,
    /// 목록에 없는 사유. 그 사이 지워졌을 수 있다.
    #[error("목록에 없는 사유를 골랐습니다")]
    ReasonNotFound(Vec<i64>),
    #[error("우선 배정을 바꾸지 못했습니다: {0}")]
    Db(#[from] sqlx::Error),
}

/// 사유를 바꾼 결과.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrioritiesChanged {
    /// 바뀐 것이 있었는지. 같은 사유를 다시 보내면 `false`이고 아무 기록도 남지 않는다.
    pub changed: bool,
    /// BR-S13: 추첨한 강좌라 추첨이 취소되고 추첨 전으로 돌아갔는지.
    pub lottery_cancelled: bool,
}

/// 신청의 사유를 `reason_ids`로 바꾼다. 빈 목록이면 모두 뗀다.
///
/// 추첨한 강좌면 그 추첨을 취소한다. 읽고 확인한 뒤 쓰므로 `BEGIN IMMEDIATE`로 한다.
pub async fn set(
    pool: &SqlitePool,
    application_id: i64,
    reason_ids: &[i64],
    staff_id: i64,
    now: OffsetDateTime,
) -> Result<PrioritiesChanged, SetPrioritiesError> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let row: Option<(i64, i64, String, bool)> = sqlx::query_as(
        "SELECT a.member_id, a.course_id, a.status,
                EXISTS (SELECT 1 FROM selection_closures s WHERE s.course_id = a.course_id)
         FROM applications a WHERE a.id = ?",
    )
    .bind(application_id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((member_id, course_id, status, closed)) = row else {
        return Err(SetPrioritiesError::NotFound);
    };
    if closed {
        return Err(SetPrioritiesError::Closed);
    }
    if !matches!(status.as_str(), "submitted" | "selected" | "waitlisted") {
        return Err(SetPrioritiesError::NotChangeable);
    }
    let wanted: BTreeSet<i64> = reason_ids.iter().copied().collect();
    let missing = missing_reasons(&mut tx, &wanted).await?;
    if !missing.is_empty() {
        return Err(SetPrioritiesError::ReasonNotFound(missing));
    }

    let current: BTreeSet<i64> =
        sqlx::query_scalar("SELECT reason_id FROM application_priorities WHERE application_id = ?")
            .bind(application_id)
            .fetch_all(&mut *tx)
            .await?
            .into_iter()
            .collect();
    if current == wanted {
        return Ok(PrioritiesChanged {
            changed: false,
            lottery_cancelled: false,
        });
    }

    let now = timestamp_text(now);
    let lottery_cancelled = cancel_active_run(&mut tx, course_id, staff_id, &now).await?;
    let target = ChangeTarget {
        application_id,
        member_id,
        course_id,
        staff_id,
        now: &now,
    };
    for &reason_id in current.difference(&wanted) {
        sqlx::query(
            "DELETE FROM application_priorities WHERE application_id = ? AND reason_id = ?",
        )
        .bind(application_id)
        .bind(reason_id)
        .execute(&mut *tx)
        .await?;
        record_change(&mut tx, &target, reason_id, "removed").await?;
    }
    for &reason_id in wanted.difference(&current) {
        attach(&mut tx, &target, reason_id).await?;
    }

    tx.commit().await?;
    Ok(PrioritiesChanged {
        changed: true,
        lottery_cancelled,
    })
}

/// 변경 기록에 남길 신청과 직원.
pub(super) struct ChangeTarget<'a> {
    pub application_id: i64,
    pub member_id: i64,
    pub course_id: i64,
    pub staff_id: i64,
    /// [`timestamp_text`] 글자.
    pub now: &'a str,
}

/// 사유 하나를 붙이고 기록한다. 부르는 쪽이 트랜잭션을 연다.
pub(super) async fn attach(
    conn: &mut SqliteConnection,
    target: &ChangeTarget<'_>,
    reason_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO application_priorities (application_id, reason_id) VALUES (?, ?)")
        .bind(target.application_id)
        .bind(reason_id)
        .execute(&mut *conn)
        .await?;
    record_change(conn, target, reason_id, "added").await
}

/// `ids` 가운데 목록에 없는 사유. 작은 id부터.
pub(super) async fn missing_reasons(
    conn: &mut SqliteConnection,
    ids: &BTreeSet<i64>,
) -> Result<Vec<i64>, sqlx::Error> {
    let known: BTreeSet<i64> = sqlx::query_scalar("SELECT id FROM priority_reasons")
        .fetch_all(conn)
        .await?
        .into_iter()
        .collect();
    Ok(ids.difference(&known).copied().collect())
}

async fn record_change(
    conn: &mut SqliteConnection,
    target: &ChangeTarget<'_>,
    reason_id: i64,
    change: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO priority_changes
             (application_id, member_id, course_id, reason_id, change, changed_by, changed_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(target.application_id)
    .bind(target.member_id)
    .bind(target.course_id)
    .bind(reason_id)
    .bind(change)
    .bind(target.staff_id)
    .bind(target.now)
    .execute(conn)
    .await?;
    Ok(())
}
