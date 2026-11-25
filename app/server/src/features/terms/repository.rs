//! 학기 저장, 목록, 고치기, 지우기.

use baeum_domain::NewTerm;
use sqlx::SqlitePool;
use time::{Date, OffsetDateTime};

use super::error::SaveTermError;
use crate::infra::db::{DeleteError, timestamp_text};

/// DB에 저장된 학기 하나.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct TermRecord {
    pub id: i64,
    pub name: String,
    pub starts_on: Date,
    pub ends_on: Date,
}

/// 확인을 통과한 학기를 저장하고 새 `id`를 돌려준다.
pub async fn insert(
    pool: &SqlitePool,
    term: &NewTerm,
    now: OffsetDateTime,
) -> Result<i64, SaveTermError> {
    let now = timestamp_text(now);
    let result = sqlx::query(
        "INSERT INTO terms (name, starts_on, ends_on, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(term.name().as_str())
    .bind(term.period().starts_on())
    .bind(term.period().ends_on())
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

/// 학기를 모두 읽는다. 시작일이 늦은 학기부터 (가장 최근 학기가 맨 앞).
pub async fn list(pool: &SqlitePool) -> Result<Vec<TermRecord>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, name, starts_on, ends_on FROM terms
         ORDER BY starts_on DESC, id DESC",
    )
    .fetch_all(pool)
    .await
}

/// `id`인 학기를 읽는다. 없으면 `None`.
pub async fn get(pool: &SqlitePool, id: i64) -> Result<Option<TermRecord>, sqlx::Error> {
    sqlx::query_as("SELECT id, name, starts_on, ends_on FROM terms WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// `id`인 학기의 칸을 모두 `term`으로 바꾸고 고친 뒤 모습을 돌려준다. 없으면 `None`.
pub async fn update(
    pool: &SqlitePool,
    id: i64,
    term: &NewTerm,
    now: OffsetDateTime,
) -> Result<Option<TermRecord>, SaveTermError> {
    let record = sqlx::query_as(
        "UPDATE terms SET name = ?, starts_on = ?, ends_on = ?, updated_at = ?
         WHERE id = ?
         RETURNING id, name, starts_on, ends_on",
    )
    .bind(term.name().as_str())
    .bind(term.period().starts_on())
    .bind(term.period().ends_on())
    .bind(timestamp_text(now))
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(record)
}

/// `id`인 학기를 지운다. 지웠으면 `true`, 없으면 `false`.
///
/// BR-T5: 강좌가 있는 학기는 외래 키가 막아 [`DeleteError::InUse`].
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<bool, DeleteError> {
    let result = sqlx::query("DELETE FROM terms WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
