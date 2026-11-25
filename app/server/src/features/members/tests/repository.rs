//! 회원 SQL 테스트.

use baeum_domain::MemberInput;
use baeum_domain::{Gender, NewMember};
use sqlx::SqlitePool;
use time::macros::{date, datetime};
use time::{Date, OffsetDateTime};

use crate::features::members::repository::*;
use crate::infra::db::open_temp;

fn today() -> Date {
    date!(2026 - 11 - 15)
}

fn now() -> OffsetDateTime {
    datetime!(2026-11-15 01:02:03 UTC)
}

fn input(member_no: &str) -> MemberInput<'_> {
    MemberInput {
        member_no,
        name: "회원1",
        phone_number: "010-0000-0001",
        birth_date: None,
        gender: None,
    }
}

fn new_member(input: MemberInput<'_>) -> NewMember {
    NewMember::parse(input, today()).unwrap()
}

type Row = (
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    String,
    String,
);

async fn row(pool: &SqlitePool, id: i64) -> Row {
    sqlx::query_as(
        "SELECT member_no, name, phone_number, birth_date, gender, created_at, updated_at
         FROM members WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .unwrap()
}

#[tokio::test]
async fn saves_all_fields() {
    let (_dir, pool) = open_temp().await;
    let member = new_member(MemberInput {
        birth_date: Some(date!(1950 - 03 - 01)),
        gender: Some(Gender::Female),
        ..input("A-001")
    });

    let id = insert(&pool, &member, now()).await.unwrap();

    let (member_no, name, phone_number, birth_date, gender, created_at, updated_at) =
        row(&pool, id).await;
    assert_eq!(member_no, "A-001");
    assert_eq!(name, "회원1");
    assert_eq!(phone_number, "01000000001");
    assert_eq!(birth_date.as_deref(), Some("1950-03-01"));
    assert_eq!(gender.as_deref(), Some("female"));
    assert_eq!(created_at, "2026-11-15T01:02:03.000Z");
    assert_eq!(updated_at, created_at);
}

#[tokio::test]
async fn br_m4_saves_without_optional_fields() {
    let (_dir, pool) = open_temp().await;

    let id = insert(&pool, &new_member(input("00001")), now())
        .await
        .unwrap();

    let (_, _, _, birth_date, gender, _, _) = row(&pool, id).await;
    assert_eq!(birth_date, None);
    assert_eq!(gender, None);
}

#[tokio::test]
async fn saves_male_as_text() {
    let (_dir, pool) = open_temp().await;
    let member = new_member(MemberInput {
        gender: Some(Gender::Male),
        ..input("00001")
    });

    let id = insert(&pool, &member, now()).await.unwrap();

    let (_, _, _, _, gender, _, _) = row(&pool, id).await;
    assert_eq!(gender.as_deref(), Some("male"));
}

#[tokio::test]
async fn br_m2_duplicate_member_no_is_told_apart() {
    let (_dir, pool) = open_temp().await;
    insert(&pool, &new_member(input("00001")), now())
        .await
        .unwrap();

    let error = insert(&pool, &new_member(input("00001")), now())
        .await
        .unwrap_err();

    assert!(
        matches!(error, SaveMemberError::DuplicateMemberNo),
        "{error}"
    );
}

#[tokio::test]
async fn gives_new_id_each_time() {
    let (_dir, pool) = open_temp().await;

    let first = insert(&pool, &new_member(input("00001")), now())
        .await
        .unwrap();
    let second = insert(&pool, &new_member(input("00002")), now())
        .await
        .unwrap();

    assert_ne!(first, second);
}

async fn add(pool: &SqlitePool, member_no: &str, name: &str, phone_number: &str) {
    let member = new_member(MemberInput {
        name,
        phone_number,
        ..input(member_no)
    });
    insert(pool, &member, now()).await.unwrap();
}

async fn found(pool: &SqlitePool, query: &str) -> Vec<String> {
    search(pool, query, 1, 20)
        .await
        .unwrap()
        .items
        .into_iter()
        .map(|member| member.member_no.into_inner())
        .collect()
}

async fn sample() -> (tempfile::TempDir, SqlitePool) {
    let (dir, pool) = open_temp().await;
    add(&pool, "00001", "나회원", "01011112222").await;
    add(&pool, "00002", "가회원", "01033334444").await;
    add(&pool, "A-100", "다손님", "0325556666").await;
    (dir, pool)
}

#[tokio::test]
async fn empty_query_finds_everyone_by_name() {
    let (_dir, pool) = sample().await;

    assert_eq!(found(&pool, "").await, ["00002", "00001", "A-100"]);
}

#[tokio::test]
async fn finds_by_part_of_name() {
    let (_dir, pool) = sample().await;

    assert_eq!(found(&pool, "회원").await, ["00002", "00001"]);
    assert_eq!(found(&pool, " 손님 ").await, ["A-100"]);
}

#[tokio::test]
async fn finds_by_part_of_member_no() {
    let (_dir, pool) = sample().await;

    assert_eq!(found(&pool, "A-1").await, ["A-100"]);
}

#[tokio::test]
async fn finds_by_phone_digits_with_hyphens() {
    let (_dir, pool) = sample().await;

    assert_eq!(found(&pool, "3333-4444").await, ["00002"]);
    assert_eq!(found(&pool, "032 555").await, ["A-100"]);
}

#[tokio::test]
async fn like_symbols_are_plain_text() {
    let (_dir, pool) = sample().await;

    assert!(found(&pool, "%").await.is_empty());
    assert!(found(&pool, "_").await.is_empty());
    assert!(found(&pool, "\\").await.is_empty());
}

#[tokio::test]
async fn pages_and_total() {
    let (_dir, pool) = sample().await;

    let first = search(&pool, "", 1, 2).await.unwrap();
    let second = search(&pool, "", 2, 2).await.unwrap();

    assert_eq!(first.total, 3);
    assert_eq!(first.items.len(), 2);
    assert_eq!(second.total, 3);
    assert_eq!(second.items[0].member_no.expose(), "A-100");
}

#[tokio::test]
async fn search_returns_saved_fields() {
    let (_dir, pool) = open_temp().await;
    let member = new_member(MemberInput {
        birth_date: Some(date!(1950 - 03 - 01)),
        gender: Some(Gender::Male),
        ..input("00001")
    });
    let id = insert(&pool, &member, now()).await.unwrap();

    let page = search(&pool, "", 1, 20).await.unwrap();

    let record = &page.items[0];
    assert_eq!(record.id, id);
    assert_eq!(record.name.expose(), "회원1");
    assert_eq!(record.phone_number.expose(), "01000000001");
    let birth_date = record.birth_date.as_ref().map(|date| *date.expose());
    assert_eq!(birth_date, Some(date!(1950 - 03 - 01)));
    let gender = record.gender.as_ref().map(|gender| *gender.expose());
    assert_eq!(gender, Some(Gender::Male));
}

#[test]
fn phone_digits_only_for_phone_like_query() {
    assert_eq!(phone_digits("010-1234").as_deref(), Some("0101234"));
    assert_eq!(phone_digits("회원1"), None);
    assert_eq!(phone_digits(" - "), None);
}

fn later() -> OffsetDateTime {
    datetime!(2026-11-16 09:00:00 UTC)
}

#[tokio::test]
async fn get_reads_member_with_times() {
    let (_dir, pool) = open_temp().await;
    let id = insert(&pool, &new_member(input("00001")), now())
        .await
        .unwrap();

    let record = get(&pool, id).await.unwrap().unwrap();

    assert_eq!(record.member_no.expose(), "00001");
    assert_eq!(record.created_at, now());
    assert_eq!(record.updated_at, now());
}

#[tokio::test]
async fn get_unknown_id_is_none() {
    let (_dir, pool) = open_temp().await;

    assert!(get(&pool, 999).await.unwrap().is_none());
}

#[tokio::test]
async fn update_replaces_every_field() {
    let (_dir, pool) = open_temp().await;
    let original = new_member(MemberInput {
        birth_date: Some(date!(1950 - 03 - 01)),
        gender: Some(Gender::Female),
        ..input("00001")
    });
    let id = insert(&pool, &original, now()).await.unwrap();
    let changed = new_member(MemberInput {
        member_no: "00009",
        name: "회원2",
        phone_number: "0320000000",
        birth_date: None,
        gender: None,
    });

    let record = update(&pool, id, &changed, later()).await.unwrap().unwrap();

    assert_eq!(record.id, id);
    assert_eq!(record.member_no.expose(), "00009");
    assert_eq!(record.name.expose(), "회원2");
    assert_eq!(record.phone_number.expose(), "0320000000");
    assert!(record.birth_date.is_none());
    assert!(record.gender.is_none());
    assert_eq!(record.created_at, now());
    assert_eq!(record.updated_at, later());
}

#[tokio::test]
async fn update_unknown_id_is_none() {
    let (_dir, pool) = open_temp().await;

    let result = update(&pool, 999, &new_member(input("00001")), now())
        .await
        .unwrap();

    assert!(result.is_none());
}

#[tokio::test]
async fn br_m2_update_to_taken_member_no_is_told_apart() {
    let (_dir, pool) = open_temp().await;
    insert(&pool, &new_member(input("00001")), now())
        .await
        .unwrap();
    let id = insert(&pool, &new_member(input("00002")), now())
        .await
        .unwrap();

    let error = update(&pool, id, &new_member(input("00001")), later())
        .await
        .unwrap_err();

    assert!(
        matches!(error, SaveMemberError::DuplicateMemberNo),
        "{error}"
    );
}

#[tokio::test]
async fn update_keeping_own_member_no_is_fine() {
    let (_dir, pool) = open_temp().await;
    let id = insert(&pool, &new_member(input("00001")), now())
        .await
        .unwrap();

    let record = update(&pool, id, &new_member(input("00001")), later())
        .await
        .unwrap();

    assert!(record.is_some());
}

#[tokio::test]
async fn br_m5_deletes_member_without_records() {
    let (_dir, pool) = open_temp().await;
    let id = insert(&pool, &new_member(input("00001")), now())
        .await
        .unwrap();

    assert!(delete(&pool, id).await.unwrap());
    assert!(get(&pool, id).await.unwrap().is_none());
}

#[tokio::test]
async fn delete_unknown_id_is_false() {
    let (_dir, pool) = open_temp().await;

    assert!(!delete(&pool, 999).await.unwrap());
}

#[tokio::test]
async fn member_no_can_be_reused_after_delete() {
    let (_dir, pool) = open_temp().await;
    let id = insert(&pool, &new_member(input("00001")), now())
        .await
        .unwrap();
    delete(&pool, id).await.unwrap();

    insert(&pool, &new_member(input("00001")), now())
        .await
        .unwrap();
}
