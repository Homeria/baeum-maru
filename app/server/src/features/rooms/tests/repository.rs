//! 강의실 SQL 테스트.

use baeum_domain::{NewRoom, RoomInput};
use time::OffsetDateTime;
use time::macros::datetime;

use crate::db::test_support::{insert_course, insert_schedule, insert_subject, insert_term};
use crate::features::rooms::repository::*;
use crate::infra::db::{DeleteError, open_temp};

fn now() -> OffsetDateTime {
    datetime!(2026-11-15 01:02:03 UTC)
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
    let term_id = insert_term(&pool, "2027년 1학기", "2027-03-02", "2027-06-30")
        .await
        .unwrap();
    let subject_id = insert_subject(&pool, "요가", "요가", None).await.unwrap();
    let course_id = insert_course(&pool, term_id, subject_id, None, None)
        .await
        .unwrap();
    insert_schedule(&pool, course_id, 1, 600, 660, Some(id))
        .await
        .unwrap();

    let error = delete(&pool, id).await.unwrap_err();

    assert!(matches!(error, DeleteError::InUse), "{error}");
}
