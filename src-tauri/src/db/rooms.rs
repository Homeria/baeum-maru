//! 강의실 저장, 목록, 고치기, 지우기.

use baeum_domain::{Floor, NewRoom};
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

use super::{DeleteError, is_unique_violation, timestamp_text};

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
    now: DateTime<Utc>,
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

#[cfg(test)]
mod tests {
    use baeum_domain::RoomInput;
    use chrono::TimeZone;

    use super::*;
    use crate::db::open_temp;

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

    #[tokio::test]
    async fn saves_and_lists_room() {
        let (_dir, pool) = open_temp().await;

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
        let (_dir, pool) = open_temp().await;

        insert(&pool, &new_room("본관", None, "상담실"), now())
            .await
            .unwrap();

        assert_eq!(list(&pool).await.unwrap()[0].floor, None);
    }

    #[tokio::test]
    async fn br_r1_duplicate_name_is_told_apart() {
        let (_dir, pool) = open_temp().await;
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
        let (_dir, pool) = open_temp().await;
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

    #[tokio::test]
    async fn get_reads_room_or_none() {
        let (_dir, pool) = open_temp().await;
        let id = insert(&pool, &new_room("본관", Some(3), "다목적실"), now())
            .await
            .unwrap();

        let room = get(&pool, id).await.unwrap().unwrap();

        assert_eq!(room.name, "다목적실");
        assert!(get(&pool, 999).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn update_replaces_every_field() {
        let (_dir, pool) = open_temp().await;
        let id = insert(&pool, &new_room("본관", Some(3), "다목적실"), now())
            .await
            .unwrap();

        let room = update(&pool, id, &new_room("별관", None, "요가실"), now())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            room,
            RoomRecord {
                id,
                building: "별관".to_owned(),
                floor: None,
                name: "요가실".to_owned(),
            }
        );
    }

    #[tokio::test]
    async fn update_unknown_id_is_none() {
        let (_dir, pool) = open_temp().await;

        let room = update(&pool, 999, &new_room("본관", None, "요가실"), now())
            .await
            .unwrap();

        assert!(room.is_none());
    }

    #[tokio::test]
    async fn br_r1_update_to_taken_name_is_told_apart() {
        let (_dir, pool) = open_temp().await;
        insert(&pool, &new_room("본관", None, "다목적실"), now())
            .await
            .unwrap();
        let id = insert(&pool, &new_room("본관", None, "요가실"), now())
            .await
            .unwrap();

        let error = update(&pool, id, &new_room("본관", Some(1), "다목적실"), now())
            .await
            .unwrap_err();

        assert!(matches!(error, SaveRoomError::DuplicateName), "{error}");
    }

    #[tokio::test]
    async fn delete_removes_room() {
        let (_dir, pool) = open_temp().await;
        let id = insert(&pool, &new_room("본관", None, "다목적실"), now())
            .await
            .unwrap();

        assert!(delete(&pool, id).await.unwrap());
        assert!(!delete(&pool, id).await.unwrap());
        assert!(list(&pool).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn br_r2_room_used_by_schedule_is_in_use() {
        let (_dir, pool) = open_temp().await;
        let id = insert(&pool, &new_room("본관", Some(2), "201호"), now())
            .await
            .unwrap();
        for sql in [
            "INSERT INTO terms (name, starts_on, ends_on, created_at, updated_at)
             VALUES ('2027년 1학기', '2027-03-02', '2027-06-30', '', '')",
            "INSERT INTO subjects (name, name_key, created_at, updated_at)
             VALUES ('요가', '요가', '', '')",
            "INSERT INTO courses (term_id, subject_id, capacity, created_at, updated_at)
             VALUES ((SELECT id FROM terms), (SELECT id FROM subjects), 20, '', '')",
        ] {
            sqlx::query(sql).execute(&pool).await.unwrap();
        }
        sqlx::query(
            "INSERT INTO course_schedules (course_id, weekday, start_minute, end_minute, room_id)
             VALUES ((SELECT id FROM courses), 1, 600, 660, ?)",
        )
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();

        let error = delete(&pool, id).await.unwrap_err();

        assert!(matches!(error, DeleteError::InUse), "{error}");
    }
}
