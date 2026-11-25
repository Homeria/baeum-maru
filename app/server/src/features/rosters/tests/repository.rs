//! 결과 명단 SQL 테스트.

use baeum_domain::{MaskChar, RosterChoice, RosterKind};
use baeum_domain::{RosterColumn, RosterSpec};
use time::macros::datetime;

use crate::db::test_support::{insert_course, insert_drawn_course, insert_subject};
use crate::features::rosters::repository::*;
use crate::features::selection_results::service::changes as selection_changes;
use crate::infra::db::open_temp;

#[tokio::test]
async fn br_p1_reads_only_closed_courses_with_member_details() {
    let (_dir, pool) = open_temp().await;
    let drawn = insert_drawn_course(&pool, 2, 5, true).await;
    let subject_id = insert_subject(&pool, "서예", "서예", None).await.unwrap();
    insert_course(&pool, drawn.term_id, subject_id, Some("초급"), None)
        .await
        .unwrap();

    let roster = term_roster(&pool, drawn.term_id).await.unwrap();

    assert_eq!(roster.len(), 1);
    let course = &roster[0];
    assert_eq!(course.course_id, drawn.course_id);
    assert_eq!(course.name, "요가");
    assert_eq!(course.capacity.value(), 2);
    let results: Vec<(RosterResult, Option<usize>)> = course
        .entries
        .iter()
        .map(|entry| (entry.result, entry.waitlist_number))
        .collect();
    assert_eq!(
        results,
        [
            (RosterResult::Selected, None),
            (RosterResult::Selected, None),
            (RosterResult::Waitlisted, Some(1)),
            (RosterResult::Waitlisted, Some(2)),
            (RosterResult::Waitlisted, Some(3)),
        ]
    );
    assert!(
        course.entries[0]
            .member
            .phone_number
            .expose()
            .starts_with("010")
    );
}

#[tokio::test]
async fn br_p2_withdrawn_comes_last_without_number() {
    let (_dir, pool) = open_temp().await;
    let drawn = insert_drawn_course(&pool, 2, 5, true).await;
    let now = datetime!(2026-11-23 01:00:00 UTC);
    let first_waiting = term_roster(&pool, drawn.term_id).await.unwrap()[0].entries[2]
        .member
        .member_id;
    let application_id: i64 =
        sqlx::query_scalar("SELECT id FROM applications WHERE member_id = ? AND course_id = ?")
            .bind(first_waiting)
            .bind(drawn.course_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    selection_changes::withdraw(&pool, application_id, None, drawn.staff_id, now)
        .await
        .unwrap();

    let roster = term_roster(&pool, drawn.term_id).await.unwrap();

    let last = roster[0].entries.last().unwrap();
    assert_eq!(last.result, RosterResult::Withdrawn);
    assert_eq!(last.member.member_id, first_waiting);
    assert_eq!(last.waitlist_number, None);
    let numbers: Vec<Option<usize>> = roster[0]
        .entries
        .iter()
        .filter(|entry| entry.result == RosterResult::Waitlisted)
        .map(|entry| entry.waitlist_number)
        .collect();
    assert_eq!(numbers, [Some(2), Some(3)]);
}

#[tokio::test]
async fn br_p11_records_export_with_column_codes() {
    let (_dir, pool) = open_temp().await;
    let drawn = insert_drawn_course(&pool, 2, 5, true).await;
    let spec = RosterSpec::new(RosterChoice {
        kind: RosterKind::PostedByCourse,
        include: &[RosterColumn::PhoneNumber],
        mask: &[RosterColumn::Name],
        mask_char: MaskChar::X,
    })
    .unwrap();

    record_export(
        &pool,
        drawn.term_id,
        &spec,
        1,
        drawn.staff_id,
        datetime!(2026-11-23 01:00:00 UTC),
    )
    .await
    .unwrap();

    let row: (String, String, String, String, i64) = sqlx::query_as(
        "SELECT kind, included_columns, masked_columns, mask_char, course_count
         FROM roster_exports",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        row,
        (
            "posted_by_course".to_owned(),
            "course_name,capacity,result,waitlist_number,name,member_no,phone_number,guaranteed"
                .to_owned(),
            "name,phone_number".to_owned(),
            "x".to_owned(),
            1
        )
    );
}
