//! 우선 배정 사유 붙이기·떼기와 변경 기록(BR-S14)의 SQL. 순서와 규칙 확인은
//! `service::priorities`가 한다.

use std::collections::BTreeSet;

use sqlx::SqliteConnection;

/// 사유를 바꿀 신청의 회원, 강좌, 상태(DB 글자), 선발을 마감한 강좌인지.
pub(in crate::features::applications) struct PriorityTarget {
    pub(in crate::features::applications) member_id: i64,
    pub(in crate::features::applications) course_id: i64,
    pub(in crate::features::applications) status: String,
    pub(in crate::features::applications) closed: bool,
}

/// 사유를 바꿀 신청을 읽는다. 없으면 `None`.
pub(in crate::features::applications) async fn priority_target(
    conn: &mut SqliteConnection,
    application_id: i64,
) -> Result<Option<PriorityTarget>, sqlx::Error> {
    let row: Option<(i64, i64, String, bool)> = sqlx::query_as(
        "SELECT a.member_id, a.course_id, a.status,
                EXISTS (SELECT 1 FROM selection_closures s WHERE s.course_id = a.course_id)
         FROM applications a WHERE a.id = ?",
    )
    .bind(application_id)
    .fetch_optional(conn)
    .await?;
    Ok(
        row.map(|(member_id, course_id, status, closed)| PriorityTarget {
            member_id,
            course_id,
            status,
            closed,
        }),
    )
}

/// 신청에 지금 붙은 사유.
pub(in crate::features::applications) async fn current_reasons(
    conn: &mut SqliteConnection,
    application_id: i64,
) -> Result<BTreeSet<i64>, sqlx::Error> {
    Ok(
        sqlx::query_scalar("SELECT reason_id FROM application_priorities WHERE application_id = ?")
            .bind(application_id)
            .fetch_all(conn)
            .await?
            .into_iter()
            .collect(),
    )
}

/// 변경 기록에 남길 신청과 직원.
pub(in crate::features::applications) struct ChangeTarget<'a> {
    pub(in crate::features::applications) application_id: i64,
    pub(in crate::features::applications) member_id: i64,
    pub(in crate::features::applications) course_id: i64,
    pub(in crate::features::applications) staff_id: i64,
    /// [`timestamp_text`] 글자.
    pub(in crate::features::applications) now: &'a str,
}

/// 사유 하나를 붙이고 기록한다. 부르는 쪽이 트랜잭션을 연다.
pub(in crate::features::applications) async fn attach(
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

/// 사유 하나를 떼고 기록한다. 부르는 쪽이 트랜잭션을 연다.
pub(in crate::features::applications) async fn detach(
    conn: &mut SqliteConnection,
    target: &ChangeTarget<'_>,
    reason_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM application_priorities WHERE application_id = ? AND reason_id = ?")
        .bind(target.application_id)
        .bind(reason_id)
        .execute(&mut *conn)
        .await?;
    record_change(conn, target, reason_id, "removed").await
}

/// `ids` 가운데 목록에 없는 사유. 작은 id부터.
pub(in crate::features::applications) async fn missing_reasons(
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
