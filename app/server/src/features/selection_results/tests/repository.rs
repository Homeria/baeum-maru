//! 선발 결과 SQL 테스트.

use baeum_domain::{ContactResult, WithdrawnFrom};
use sqlx::SqlitePool;

use crate::db::test_support::{
    insert_course, insert_drawn_course, insert_selection_change, insert_subject,
};
use crate::features::selection_results::error::*;
use crate::features::selection_results::repository::*;
use crate::features::selection_results::service::closed_courses;
use crate::infra::db::open_temp;

/// 정원 2명 강좌에 신청 5건을 추첨했다. 마감하면 당첨 2명, 대기 3명.
struct Fixture {
    _dir: tempfile::TempDir,
    pool: SqlitePool,
    term_id: i64,
    course_id: i64,
    staff_id: i64,
}

async fn fixture(close: bool) -> Fixture {
    let (dir, pool) = open_temp().await;
    let drawn = insert_drawn_course(&pool, 2, 5, close).await;
    Fixture {
        _dir: dir,
        pool,
        term_id: drawn.term_id,
        course_id: drawn.course_id,
        staff_id: drawn.staff_id,
    }
}

impl Fixture {
    async fn result(&self) -> CourseResult {
        course_result(&self.pool, self.course_id).await.unwrap()
    }

    /// 상태를 바꾸고 그 기록을 남긴다. 저장 함수가 생기기 전이라 DB에 바로 넣는다.
    async fn change(&self, application_id: i64, kind: &str, from: &str, to: &str) {
        sqlx::query("UPDATE applications SET status = ? WHERE id = ?")
            .bind(to)
            .bind(application_id)
            .execute(&self.pool)
            .await
            .unwrap();
        insert_selection_change(
            &self.pool,
            application_id,
            kind,
            from,
            to,
            None,
            self.staff_id,
        )
        .await
        .unwrap();
    }
}

fn ids<'a>(members: impl IntoIterator<Item = &'a ResultMember>) -> Vec<i64> {
    members.into_iter().map(|m| m.application_id).collect()
}

#[tokio::test]
async fn br_w6_waitlist_numbers_follow_draw_order() {
    let f = fixture(true).await;

    let result = f.result().await;

    assert_eq!(result.selected.len(), 2);
    assert_eq!(result.open_seats, 0);
    let numbers: Vec<usize> = result
        .waitlisted
        .iter()
        .map(|w| w.waitlist_number)
        .collect();
    assert_eq!(numbers, [1, 2, 3]);
    assert!(result.waitlisted.iter().all(|w| !w.contact_turn));
    assert!(
        result.waitlisted[0]
            .phone_number
            .expose()
            .starts_with("010")
    );
    assert!(result.withdrawn.is_empty());
}

#[tokio::test]
async fn br_w1_withdrawal_opens_seat_and_first_waiting_is_turn() {
    let f = fixture(true).await;
    let before = f.result().await;
    let leaving = before.selected[0].member.application_id;

    f.change(leaving, "withdrawn", "selected", "withdrawn")
        .await;

    let result = f.result().await;
    assert_eq!(result.open_seats, 1);
    let turns: Vec<bool> = result.waitlisted.iter().map(|w| w.contact_turn).collect();
    assert_eq!(turns, [true, false, false]);
    assert_eq!(ids(result.withdrawn.iter().map(|w| &w.member)), [leaving]);
    assert_eq!(result.withdrawn[0].withdrawn_from, WithdrawnFrom::Selected);
    assert_eq!(result.withdrawn[0].waitlist_number, None);
}

#[tokio::test]
async fn br_w5_unreachable_passes_turn_and_promoted_keeps_number() {
    let f = fixture(true).await;
    let before = f.result().await;
    let first = before.waitlisted[0].member.application_id;
    let second = before.waitlisted[1].member.application_id;
    f.change(
        before.selected[0].member.application_id,
        "withdrawn",
        "selected",
        "withdrawn",
    )
    .await;

    f.change(first, "unreachable", "waitlisted", "waitlisted")
        .await;
    let skipped = f.result().await;
    f.change(second, "accepted", "waitlisted", "selected").await;
    let promoted = f.result().await;

    assert_eq!(
        skipped.waitlisted[0].last_contact,
        Some(ContactResult::Unreachable)
    );
    assert!(!skipped.waitlisted[0].contact_turn);
    assert!(skipped.waitlisted[1].contact_turn);
    assert_eq!(promoted.open_seats, 0);
    let promoted_entry = promoted
        .selected
        .iter()
        .find(|s| s.member.application_id == second)
        .unwrap();
    assert_eq!(promoted_entry.waitlist_number, Some(2));
    // 남은 대기자의 번호는 그대로다
    let numbers: Vec<usize> = promoted
        .waitlisted
        .iter()
        .map(|w| w.waitlist_number)
        .collect();
    assert_eq!(numbers, [1, 3]);
}

#[tokio::test]
async fn br_c3_drawn_but_not_closed_is_refused() {
    let f = fixture(false).await;

    let error = course_result(&f.pool, f.course_id).await.unwrap_err();
    let missing = course_result(&f.pool, 999).await.unwrap_err();

    assert!(matches!(error, CourseResultError::NotClosed), "{error}");
    assert!(
        matches!(missing, CourseResultError::CourseNotFound),
        "{missing}"
    );
}

#[tokio::test]
async fn closed_courses_lists_only_closed_with_counts() {
    let f = fixture(true).await;
    let subject_id = insert_subject(&f.pool, "서예", "서예", None).await.unwrap();
    insert_course(&f.pool, f.term_id, subject_id, None, None)
        .await
        .unwrap();

    let courses = closed_courses(&f.pool, f.term_id).await.unwrap();

    assert_eq!(courses.len(), 1);
    assert_eq!(courses[0].course_id, f.course_id);
    assert_eq!((courses[0].selected, courses[0].waitlisted), (2, 3));
    assert_eq!(courses[0].open_seats, 0);
}
