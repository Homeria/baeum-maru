//! 학기 SQL 테스트.

use baeum_domain::NewTerm;
use baeum_domain::TermInput;
use time::macros::{date, datetime};
use time::{Date, OffsetDateTime};

use crate::db::test_support::{insert_course, insert_subject};
use crate::features::terms::error::*;
use crate::features::terms::repository::*;
use crate::infra::db::{DeleteError, open_temp};

fn now() -> OffsetDateTime {
    datetime!(2026-11-15 01:02:03 UTC)
}

fn new_term(name: &str, starts_on: Date, ends_on: Date) -> NewTerm {
    NewTerm::parse(TermInput {
        name,
        starts_on: Some(starts_on),
        ends_on: Some(ends_on),
    })
    .unwrap()
}

#[tokio::test]
async fn saves_and_lists_term() {
    let (_dir, pool) = open_temp().await;

    let id = insert(
        &pool,
        &new_term("2027년 1학기", date!(2027 - 03 - 02), date!(2027 - 06 - 30)),
        now(),
    )
    .await
    .unwrap();

    assert_eq!(
        list(&pool).await.unwrap(),
        [TermRecord {
            id,
            name: "2027년 1학기".to_owned(),
            starts_on: date!(2027 - 03 - 02),
            ends_on: date!(2027 - 06 - 30),
        }]
    );
}

#[tokio::test]
async fn lists_latest_term_first() {
    let (_dir, pool) = open_temp().await;
    for (name, starts_on, ends_on) in [
        ("2027년 1학기", date!(2027 - 03 - 02), date!(2027 - 06 - 30)),
        ("2027년 2학기", date!(2027 - 09 - 01), date!(2027 - 12 - 31)),
        ("2026년 2학기", date!(2026 - 09 - 01), date!(2026 - 12 - 31)),
    ] {
        insert(&pool, &new_term(name, starts_on, ends_on), now())
            .await
            .unwrap();
    }

    let names: Vec<String> = list(&pool)
        .await
        .unwrap()
        .into_iter()
        .map(|term| term.name)
        .collect();

    assert_eq!(names, ["2027년 2학기", "2027년 1학기", "2026년 2학기"]);
}

#[tokio::test]
async fn br_t1_duplicate_name_is_told_apart() {
    let (_dir, pool) = open_temp().await;
    insert(
        &pool,
        &new_term("2027년 1학기", date!(2027 - 03 - 02), date!(2027 - 06 - 30)),
        now(),
    )
    .await
    .unwrap();

    let error = insert(
        &pool,
        &new_term("2027년 1학기", date!(2027 - 09 - 01), date!(2027 - 12 - 31)),
        now(),
    )
    .await
    .unwrap_err();

    assert!(matches!(error, SaveTermError::DuplicateName), "{error}");
}

#[tokio::test]
async fn get_update_and_unknown_id() {
    let (_dir, pool) = open_temp().await;
    let id = insert(
        &pool,
        &new_term("2027년 1학기", date!(2027 - 03 - 02), date!(2027 - 06 - 30)),
        now(),
    )
    .await
    .unwrap();

    let term = update(
        &pool,
        id,
        &new_term(
            "2027년 봄학기",
            date!(2027 - 03 - 04),
            date!(2027 - 06 - 27),
        ),
        now(),
    )
    .await
    .unwrap()
    .unwrap();

    assert_eq!(get(&pool, id).await.unwrap(), Some(term.clone()));
    assert_eq!(term.name, "2027년 봄학기");
    assert_eq!(term.starts_on, date!(2027 - 03 - 04));
    assert!(get(&pool, 999).await.unwrap().is_none());
    assert!(
        update(
            &pool,
            999,
            &new_term("없음", date!(2027 - 03 - 02), date!(2027 - 06 - 30)),
            now()
        )
        .await
        .unwrap()
        .is_none()
    );
}

#[tokio::test]
async fn deletes_term_without_course() {
    let (_dir, pool) = open_temp().await;
    let id = insert(
        &pool,
        &new_term("2027년 1학기", date!(2027 - 03 - 02), date!(2027 - 06 - 30)),
        now(),
    )
    .await
    .unwrap();

    assert!(delete(&pool, id).await.unwrap());
    assert!(!delete(&pool, id).await.unwrap());
}

#[tokio::test]
async fn br_t5_term_with_course_is_in_use() {
    let (_dir, pool) = open_temp().await;
    let id = insert(
        &pool,
        &new_term("2027년 1학기", date!(2027 - 03 - 02), date!(2027 - 06 - 30)),
        now(),
    )
    .await
    .unwrap();
    let subject_id = insert_subject(&pool, "요가", "요가", None).await.unwrap();
    insert_course(&pool, id, subject_id, None, None)
        .await
        .unwrap();

    let error = delete(&pool, id).await.unwrap_err();

    assert!(matches!(error, DeleteError::InUse), "{error}");
}
