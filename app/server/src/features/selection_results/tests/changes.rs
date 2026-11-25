//! 포기·연락 기록 SQL 테스트.

use baeum_domain::{ContactResult, WithdrawnFrom};
use sqlx::SqlitePool;
use time::OffsetDateTime;
use time::macros::datetime;

use crate::db::test_support::insert_drawn_course;
use crate::features::selection_results::repository::changes::*;
use crate::features::selection_results::repository::{CourseResult, course_result};
use crate::infra::db::open_temp;

fn now() -> OffsetDateTime {
    datetime!(2026-11-23 01:02:03 UTC)
}

/// 정원 2명 강좌에 신청 5건. 당첨 2명, 대기 3명.
struct Fixture {
    _dir: tempfile::TempDir,
    pool: SqlitePool,
    course_id: i64,
    staff_id: i64,
}

async fn fixture(close: bool) -> Fixture {
    let (dir, pool) = open_temp().await;
    let drawn = insert_drawn_course(&pool, 2, 5, close).await;
    Fixture {
        _dir: dir,
        pool,
        course_id: drawn.course_id,
        staff_id: drawn.staff_id,
    }
}

impl Fixture {
    async fn result(&self) -> CourseResult {
        course_result(&self.pool, self.course_id).await.unwrap()
    }

    async fn withdraw(&self, id: i64, reason: Option<&str>) -> Result<(), ChangeSelectionError> {
        withdraw(&self.pool, id, reason, self.staff_id, now()).await
    }

    async fn contact(&self, id: i64, result: ContactResult) -> Result<(), ChangeSelectionError> {
        record_contact(&self.pool, id, result, self.staff_id, now()).await
    }

    async fn correct(&self, id: i64) -> Result<(), ChangeSelectionError> {
        correct_withdrawal(
            &self.pool,
            id,
            "다른 회원을 잘못 누름",
            self.staff_id,
            now(),
        )
        .await
    }

    async fn records(&self) -> Vec<(i64, String, String, String, Option<String>)> {
        sqlx::query_as(
            "SELECT application_id, kind, from_status, to_status, reason
             FROM selection_changes ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .unwrap()
    }
}

#[tokio::test]
async fn br_c3_withdraws_selected_with_record() {
    let f = fixture(true).await;
    let leaving = f.result().await.selected[0].member.application_id;

    f.withdraw(leaving, Some("본인 전화")).await.unwrap();

    let result = f.result().await;
    assert_eq!(result.open_seats, 1);
    assert_eq!(result.withdrawn[0].member.application_id, leaving);
    assert_eq!(
        f.records().await,
        [(
            leaving,
            "withdrawn".to_owned(),
            "selected".to_owned(),
            "withdrawn".to_owned(),
            Some("본인 전화".to_owned())
        )]
    );
}

#[tokio::test]
async fn br_c3_not_closed_course_is_refused() {
    let f = fixture(false).await;
    let id: i64 = sqlx::query_scalar("SELECT MIN(id) FROM applications")
        .fetch_one(&f.pool)
        .await
        .unwrap();

    let error = f.withdraw(id, None).await.unwrap_err();
    let missing = f.withdraw(999, None).await.unwrap_err();

    assert!(matches!(error, ChangeSelectionError::NotClosed), "{error}");
    assert!(
        matches!(missing, ChangeSelectionError::ApplicationNotFound),
        "{missing}"
    );
}

#[tokio::test]
async fn br_w3_accept_needs_open_seat() {
    let f = fixture(true).await;
    let before = f.result().await;
    let first = before.waitlisted[0].member.application_id;

    let full = f.contact(first, ContactResult::Accepted).await.unwrap_err();
    f.withdraw(before.selected[0].member.application_id, None)
        .await
        .unwrap();
    f.contact(first, ContactResult::Accepted).await.unwrap();

    assert!(matches!(full, ChangeSelectionError::NoOpenSeat), "{full}");
    let after = f.result().await;
    assert_eq!(after.open_seats, 0);
    assert!(
        after
            .selected
            .iter()
            .any(|s| s.member.application_id == first)
    );
}

#[tokio::test]
async fn br_w4_w5_declined_withdraws_and_unreachable_stays() {
    let f = fixture(true).await;
    let before = f.result().await;
    let first = before.waitlisted[0].member.application_id;
    let second = before.waitlisted[1].member.application_id;

    f.contact(first, ContactResult::Unreachable).await.unwrap();
    f.contact(second, ContactResult::Declined).await.unwrap();

    let after = f.result().await;
    assert_eq!(after.waitlisted[0].member.application_id, first);
    assert_eq!(
        after.waitlisted[0].last_contact,
        Some(ContactResult::Unreachable)
    );
    assert_eq!(after.withdrawn[0].member.application_id, second);
    assert_eq!(after.withdrawn[0].withdrawn_from, WithdrawnFrom::Waitlisted);
    // 대기자가 아닌 신청에는 연락 결과를 남기지 않는다
    let selected = before.selected[0].member.application_id;
    let wrong = f
        .contact(selected, ContactResult::Declined)
        .await
        .unwrap_err();
    assert!(
        matches!(wrong, ChangeSelectionError::WrongStatus),
        "{wrong}"
    );
}

#[tokio::test]
async fn br_w8_corrects_waitlisted_back_to_same_number() {
    let f = fixture(true).await;
    let second = f.result().await.waitlisted[1].member.application_id;
    f.withdraw(second, None).await.unwrap();

    f.correct(second).await.unwrap();

    let after = f.result().await;
    assert_eq!(after.waitlisted[1].member.application_id, second);
    assert_eq!(after.waitlisted[1].waitlist_number, 2);
    let records = f.records().await;
    assert_eq!(records[1].1, "corrected");
    assert_eq!(records[1].3, "waitlisted");
    assert_eq!(records[1].4.as_deref(), Some("다른 회원을 잘못 누름"));
}

#[tokio::test]
async fn br_w8_selected_comes_back_only_while_seat_is_open() {
    let f = fixture(true).await;
    let before = f.result().await;
    let leaving = before.selected[0].member.application_id;
    let other = before.selected[1].member.application_id;
    f.withdraw(leaving, None).await.unwrap();
    f.withdraw(other, None).await.unwrap();
    // 빈자리 2개 중 하나를 대기 1번이 채운다
    f.contact(
        before.waitlisted[0].member.application_id,
        ContactResult::Accepted,
    )
    .await
    .unwrap();

    f.correct(leaving).await.unwrap();
    let full = f.correct(other).await.unwrap_err();
    let not_withdrawn = f.correct(leaving).await.unwrap_err();

    assert!(matches!(full, ChangeSelectionError::NoOpenSeat), "{full}");
    assert!(
        matches!(not_withdrawn, ChangeSelectionError::WrongStatus),
        "{not_withdrawn}"
    );
    assert_eq!(f.result().await.open_seats, 0);
}
