//! 강의실 저장과 목록.

use baeum_domain::{Floor, NewRoom};
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

use super::timestamp_text;

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
        let is_unique_violation = error
            .as_database_error()
            .is_some_and(|db_error| db_error.is_unique_violation());
        if is_unique_violation {
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
    now: DateTime<Utc>,
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

#[cfg(test)]
mod tests {
    use baeum_domain::RoomInput;
    use chrono::TimeZone;

    use super::*;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 15, 1, 2, 3).unwrap()
    }

    fn new_room(building: &str, floor: Option<i32>, name: &str) -> NewRoom {
        NewRoom::parse(RoomInput {
            name,
            building,
            floor,
        })
        .unwrap()
    }

    async fn open() -> (tempfile::TempDir, SqlitePool) {
        let dir = tempfile::tempdir().unwrap();
        let pool = crate::db::open(dir.path()).await.unwrap();
        (dir, pool)
    }

    #[tokio::test]
    async fn saves_and_lists_room() {
        let (_dir, pool) = open().await;

        let id = insert(&pool, &new_room("본관", Some(-1), "다목적실"), now())
            .await
            .unwrap();

        let rooms = list(&pool).await.unwrap();
        assert_eq!(
            rooms,
            [RoomRecord {
                id,
                building: "본관".to_owned(),
                floor: Some(-1),
                name: "다목적실".to_owned(),
            }]
        );
    }

    #[tokio::test]
    async fn saves_room_without_floor() {
        let (_dir, pool) = open().await;

        insert(&pool, &new_room("본관", None, "상담실"), now())
            .await
            .unwrap();

        assert_eq!(list(&pool).await.unwrap()[0].floor, None);
    }

    #[tokio::test]
    async fn br_r1_duplicate_name_is_told_apart() {
        let (_dir, pool) = open().await;
        insert(&pool, &new_room("본관", Some(3), "다목적실"), now())
            .await
            .unwrap();

        let error = insert(&pool, &new_room("본관", Some(2), "다목적실"), now())
            .await
            .unwrap_err();

        assert!(matches!(error, SaveRoomError::DuplicateName), "{error}");
    }

    #[tokio::test]
    async fn lists_by_building_floor_then_name() {
        let (_dir, pool) = open().await;
        for (building, floor, name) in [
            ("본관", Some(3), "컴퓨터실"),
            ("본관", None, "창고"),
            ("별관", Some(2), "요가실"),
            ("본관", Some(3), "다목적실"),
            ("본관", Some(-1), "체력단련실"),
        ] {
            insert(&pool, &new_room(building, floor, name), now())
                .await
                .unwrap();
        }

        let names: Vec<String> = list(&pool)
            .await
            .unwrap()
            .into_iter()
            .map(|room| room.name)
            .collect();

        assert_eq!(
            names,
            ["요가실", "체력단련실", "다목적실", "컴퓨터실", "창고"]
        );
    }
}
