//! 선발 마감 뒤 포기, 대기자 연락 결과, 잘못 처리 바로잡기의 SQL. 순서와 규칙 확인은
//! `service::changes`가 한다.

use sqlx::SqliteConnection;
use time::OffsetDateTime;

use crate::infra::db::{decode_error, timestamp_text};

/// 바꿀 신청과 그 강좌.
pub(in crate::features::selection_results) struct Target {
    pub(in crate::features::selection_results) course_id: i64,
    /// 정원. DB 값 그대로이고 빈자리를 셀 때 확인한다.
    pub(in crate::features::selection_results) capacity: i64,
    /// 신청 상태. DB 글자다.
    pub(in crate::features::selection_results) status: String,
    /// 선발을 마감한 강좌인지.
    pub(in crate::features::selection_results) closed: bool,
}

/// 신청과 그 강좌를 읽는다. 없으면 `None`.
pub(in crate::features::selection_results) async fn target(
    conn: &mut SqliteConnection,
    application_id: i64,
) -> Result<Option<Target>, sqlx::Error> {
    let row: Option<(i64, i64, String, bool)> = sqlx::query_as(
        "SELECT c.id, c.capacity, a.status,
                EXISTS (SELECT 1 FROM selection_closures s WHERE s.course_id = c.id)
         FROM applications a JOIN courses c ON c.id = a.course_id
         WHERE a.id = ?",
    )
    .bind(application_id)
    .fetch_optional(conn)
    .await?;
    let Some((course_id, capacity, status, closed)) = row else {
        return Ok(None);
    };
    Ok(Some(Target {
        course_id,
        capacity,
        status,
        closed,
    }))
}

/// 강좌의 당첨(확정 포함) 인원.
pub(in crate::features::selection_results) async fn selected_count(
    conn: &mut SqliteConnection,
    course_id: i64,
) -> Result<usize, sqlx::Error> {
    let selected: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM applications
         WHERE course_id = ? AND status IN ('selected', 'confirmed')",
    )
    .bind(course_id)
    .fetch_one(conn)
    .await?;
    usize::try_from(selected).map_err(|_| decode_error("인원이 범위 밖입니다"))
}

/// 마지막 포기 기록의 포기 전 상태(DB 글자). 기록이 없으면 `None`.
pub(in crate::features::selection_results) async fn withdrawn_from_status(
    conn: &mut SqliteConnection,
    application_id: i64,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT from_status FROM selection_changes
         WHERE application_id = ? AND to_status = 'withdrawn'
         ORDER BY id DESC LIMIT 1",
    )
    .bind(application_id)
    .fetch_optional(conn)
    .await
}

/// 신청 상태 하나의 바뀜. 모두 DB 글자다.
pub(in crate::features::selection_results) struct Change<'a> {
    pub(in crate::features::selection_results) application_id: i64,
    pub(in crate::features::selection_results) kind: &'a str,
    pub(in crate::features::selection_results) from: &'a str,
    pub(in crate::features::selection_results) to: &'a str,
    pub(in crate::features::selection_results) reason: Option<&'a str>,
}

/// 신청 상태를 바꾸고 기록을 남긴다(BR-W7).
pub(in crate::features::selection_results) async fn apply(
    conn: &mut SqliteConnection,
    change: &Change<'_>,
    staff_id: i64,
    now: OffsetDateTime,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE applications SET status = ? WHERE id = ?")
        .bind(change.to)
        .bind(change.application_id)
        .execute(&mut *conn)
        .await?;
    sqlx::query(
        "INSERT INTO selection_changes
             (application_id, kind, from_status, to_status, reason, changed_by, changed_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(change.application_id)
    .bind(change.kind)
    .bind(change.from)
    .bind(change.to)
    .bind(change.reason)
    .bind(staff_id)
    .bind(timestamp_text(now))
    .execute(&mut *conn)
    .await?;
    Ok(())
}
