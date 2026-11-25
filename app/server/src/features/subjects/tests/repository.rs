//! 과목 SQL 테스트.

use baeum_domain::NewSubject;
use baeum_domain::SubjectInput;
use time::OffsetDateTime;
use time::macros::datetime;

use crate::db::test_support::{insert_course, insert_term};
use crate::features::subjects::repository::*;
use crate::infra::db::{DeleteError, open_temp};

fn now() -> OffsetDateTime {
    datetime!(2026-11-15 01:02:03 UTC)
}

fn new_subject(name: &str, category: Option<&str>) -> NewSubject {
    NewSubject::parse(SubjectInput { name, category }).unwrap()
}

#[tokio::test]
async fn saves_and_lists_subject() {
    let (_dir, pool) = open_temp().await;

    let id = insert(&pool, &new_subject(" 한글 교실 ", Some("평생교육")), now())
        .await
        .unwrap();

    assert_eq!(
        list(&pool).await.unwrap(),
        [SubjectRecord {
            id,
            name: "한글 교실".to_owned(),
            category: Some("평생교육".to_owned()),
        }]
    );
}

#[tokio::test]
async fn blank_category_is_saved_as_null() {
    let (_dir, pool) = open_temp().await;

    insert(&pool, &new_subject("요가", Some("  ")), now())
        .await
        .unwrap();

    assert_eq!(list(&pool).await.unwrap()[0].category, None);
}

#[tokio::test]
async fn br_j1_name_differing_only_in_spaces_is_told_apart() {
    let (_dir, pool) = open_temp().await;
    insert(&pool, &new_subject("한글교실", None), now())
        .await
        .unwrap();

    let error = insert(&pool, &new_subject("한글 교실", Some("평생교육")), now())
        .await
        .unwrap_err();

    assert!(matches!(error, SaveSubjectError::DuplicateName), "{error}");
}

#[tokio::test]
async fn lists_by_category_then_name() {
    let (_dir, pool) = open_temp().await;
    for (name, category) in [
        ("요가", Some("취미여가")),
        ("상담", None),
        ("한글교실", Some("평생교육")),
        ("댄스", Some("취미여가")),
        ("스마트폰", Some("평생교육")),
    ] {
        insert(&pool, &new_subject(name, category), now())
            .await
            .unwrap();
    }

    let names: Vec<String> = list(&pool)
        .await
        .unwrap()
        .into_iter()
        .map(|subject| subject.name)
        .collect();

    // 취미여가(ㅊ)가 평생교육(ㅍ)보다 앞
    assert_eq!(names, ["댄스", "요가", "스마트폰", "한글교실", "상담"]);
}

#[tokio::test]
async fn get_reads_subject_or_none() {
    let (_dir, pool) = open_temp().await;
    let id = insert(&pool, &new_subject("요가", None), now())
        .await
        .unwrap();

    assert_eq!(get(&pool, id).await.unwrap().unwrap().name, "요가");
    assert!(get(&pool, 999).await.unwrap().is_none());
}

#[tokio::test]
async fn update_replaces_every_field() {
    let (_dir, pool) = open_temp().await;
    let id = insert(&pool, &new_subject("요가", Some("취미여가")), now())
        .await
        .unwrap();

    let subject = update(&pool, id, &new_subject("필라테스", None), now())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        subject,
        SubjectRecord {
            id,
            name: "필라테스".to_owned(),
            category: None,
        }
    );
}

#[tokio::test]
async fn update_can_change_only_spaces_of_own_name() {
    let (_dir, pool) = open_temp().await;
    let id = insert(&pool, &new_subject("한글교실", None), now())
        .await
        .unwrap();

    let subject = update(&pool, id, &new_subject("한글 교실", None), now())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(subject.name, "한글 교실");
}

#[tokio::test]
async fn update_unknown_id_is_none() {
    let (_dir, pool) = open_temp().await;

    let subject = update(&pool, 999, &new_subject("요가", None), now())
        .await
        .unwrap();

    assert!(subject.is_none());
}

#[tokio::test]
async fn br_j1_update_to_taken_name_is_told_apart() {
    let (_dir, pool) = open_temp().await;
    insert(&pool, &new_subject("한글교실", None), now())
        .await
        .unwrap();
    let id = insert(&pool, &new_subject("요가", None), now())
        .await
        .unwrap();

    let error = update(&pool, id, &new_subject("한글 교실", None), now())
        .await
        .unwrap_err();

    assert!(matches!(error, SaveSubjectError::DuplicateName), "{error}");
}

#[tokio::test]
async fn delete_removes_subject() {
    let (_dir, pool) = open_temp().await;
    let id = insert(&pool, &new_subject("요가", None), now())
        .await
        .unwrap();

    assert!(delete(&pool, id).await.unwrap());
    assert!(!delete(&pool, id).await.unwrap());
    assert!(list(&pool).await.unwrap().is_empty());
}

#[tokio::test]
async fn br_j2_subject_used_by_course_is_in_use() {
    let (_dir, pool) = open_temp().await;
    let id = insert(&pool, &new_subject("요가", None), now())
        .await
        .unwrap();
    let term_id = insert_term(&pool, "2027년 1학기", "2027-03-02", "2027-06-30")
        .await
        .unwrap();
    insert_course(&pool, term_id, id, None, None).await.unwrap();

    let error = delete(&pool, id).await.unwrap_err();

    assert!(matches!(error, DeleteError::InUse), "{error}");
}
