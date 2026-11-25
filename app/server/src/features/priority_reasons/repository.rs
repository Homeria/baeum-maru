//! 우선 배정 사유 저장, 목록, 고치기, 지우기 (BR-S10, BR-S11).

use baeum_domain::{NewPriorityReason, PriorityMethod};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::error::SavePriorityReasonError;
use crate::infra::db::{DeleteError, timestamp_text};

/// DB에 저장된 사유 하나.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriorityReasonRecord {
    pub id: i64,
    pub name: String,
    pub method: PriorityMethod,
}

#[derive(sqlx::FromRow)]
struct PriorityReasonRow {
    id: i64,
    name: String,
    method: String,
}

impl TryFrom<PriorityReasonRow> for PriorityReasonRecord {
    type Error = sqlx::Error;

    fn try_from(row: PriorityReasonRow) -> Result<Self, Self::Error> {
        // 마이그레이션의 CHECK가 막아서 모르는 글자는 나오지 않는다
        let method = PriorityMethod::from_code(&row.method).ok_or_else(|| {
            sqlx::Error::Decode(format!("모르는 배정 방식: {}", row.method).into())
        })?;
        Ok(PriorityReasonRecord {
            id: row.id,
            name: row.name,
            method,
        })
    }
}

/// 확인을 통과한 사유를 저장하고 새 `id`를 돌려준다.
pub async fn insert(
    pool: &SqlitePool,
    reason: &NewPriorityReason,
    now: OffsetDateTime,
) -> Result<i64, SavePriorityReasonError> {
    let now = timestamp_text(now);
    let result = sqlx::query(
        "INSERT INTO priority_reasons (name, name_key, method, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(reason.name().as_str())
    .bind(reason.name().key())
    .bind(reason.method().code())
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

/// 사유를 모두 읽는다. 이름 순서.
pub async fn list(pool: &SqlitePool) -> Result<Vec<PriorityReasonRecord>, sqlx::Error> {
    let rows: Vec<PriorityReasonRow> =
        sqlx::query_as("SELECT id, name, method FROM priority_reasons ORDER BY name")
            .fetch_all(pool)
            .await?;
    rows.into_iter()
        .map(PriorityReasonRecord::try_from)
        .collect()
}

/// `id`인 사유의 이름과 방식을 바꾸고 고친 뒤 모습을 돌려준다. 없으면 `None`.
pub async fn update(
    pool: &SqlitePool,
    id: i64,
    reason: &NewPriorityReason,
    now: OffsetDateTime,
) -> Result<Option<PriorityReasonRecord>, SavePriorityReasonError> {
    let row: Option<PriorityReasonRow> = sqlx::query_as(
        "UPDATE priority_reasons SET name = ?, name_key = ?, method = ?, updated_at = ?
         WHERE id = ?
         RETURNING id, name, method",
    )
    .bind(reason.name().as_str())
    .bind(reason.name().key())
    .bind(reason.method().code())
    .bind(timestamp_text(now))
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(PriorityReasonRecord::try_from).transpose()?)
}

/// `id`인 사유를 지운다. 지웠으면 `true`, 없으면 `false`.
///
/// BR-S10: 신청에 한 번이라도 붙었던 사유는 변경 기록의 외래 키가 막아 [`DeleteError::InUse`].
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<bool, DeleteError> {
    let result = sqlx::query("DELETE FROM priority_reasons WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
