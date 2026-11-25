//! 강의실 저장, 목록, 고치기, 지우기.

use baeum_domain::{Floor, NewRoom};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use crate::infra::db::{DeleteError, is_unique_violation, timestamp_text};

/// 강의실을 저장하지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum SaveRoomError {
    /// BR-R1: 같은 건물 안에서 강의실 이름은 겹치지 않는다.
    #[error("같은 건물에 같은 이름의 강의실이 있습니다")]
    DuplicateName,
    #[error("강의실을 저장하지 못했습니다: {0}")]
    Db(sqlx::Error),
}

impl From<sqlx::Error> for SaveRoomError {
    fn from(error: sqlx::Error) -> Self {
        // rooms에서 UNIQUE는 (building, name) 하나뿐이다
        if is_unique_violation(&error) {
            SaveRoomError::DuplicateName
        } else {
            SaveRoomError::Db(error)
        }
    }
}

/// DB에 저장된 강의실 하나. 개인정보가 아니라 감싸지 않는다.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct RoomRecord {
    pub id: i64,
    pub building: String,
    pub floor: Option<i8>,
    pub name: String,
}

/// 확인을 통과한 강의실을 저장하고 새 `id`를 돌려준다.
pub async fn insert(
    pool: &SqlitePool,
    room: &NewRoom,
    now: OffsetDateTime,
) -> Result<i64, SaveRoomError> {
    let now = timestamp_text(now);
    let result = sqlx::query(
        "INSERT INTO rooms (building, floor, name, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(room.building().as_str())
    .bind(room.floor().map(Floor::value))
    .bind(room.name().as_str())
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

/// 강의실을 모두 읽는다. 건물, 층(지하부터, 층 없음은 끝), 이름 순서.
pub async fn list(pool: &SqlitePool) -> Result<Vec<RoomRecord>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, building, floor, name FROM rooms
         ORDER BY building, floor IS NULL, floor, name",
    )
    .fetch_all(pool)
    .await
}

/// `id`인 강의실을 읽는다. 없으면 `None`.
pub async fn get(pool: &SqlitePool, id: i64) -> Result<Option<RoomRecord>, sqlx::Error> {
    sqlx::query_as("SELECT id, building, floor, name FROM rooms WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// `id`인 강의실의 칸을 모두 `room`으로 바꾸고 고친 뒤 모습을 돌려준다. 없으면 `None`.
pub async fn update(
    pool: &SqlitePool,
    id: i64,
    room: &NewRoom,
    now: OffsetDateTime,
) -> Result<Option<RoomRecord>, SaveRoomError> {
    let record = sqlx::query_as(
        "UPDATE rooms SET building = ?, floor = ?, name = ?, updated_at = ?
         WHERE id = ?
         RETURNING id, building, floor, name",
    )
    .bind(room.building().as_str())
    .bind(room.floor().map(Floor::value))
    .bind(room.name().as_str())
    .bind(timestamp_text(now))
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(record)
}

/// `id`인 강의실을 지운다. 지웠으면 `true`, 없으면 `false`.
///
/// BR-R2: 수업 시간이 쓰는 강의실은 외래 키가 막아 [`DeleteError::InUse`].
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<bool, DeleteError> {
    let result = sqlx::query("DELETE FROM rooms WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
