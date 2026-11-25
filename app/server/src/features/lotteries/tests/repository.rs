//! 추첨 SQL 테스트.

use baeum_domain::{Capacity, LotterySeed, LotteryVersion};
use baeum_domain::{LotteryDraw, draw_lottery};
use sqlx::SqlitePool;
use time::OffsetDateTime;
use time::macros::datetime;

use crate::db::test_support::{
    insert_application, insert_course, insert_member, insert_priority_reason, insert_staff,
    insert_subject, insert_term,
};
use crate::features::lotteries::repository::*;
use crate::infra::db::open_temp;

fn now() -> OffsetDateTime {
    datetime!(2026-11-22 01:02:03 UTC)
}

const SEED: u64 = 0x0123_4567_89AB_CDEF;

/// 정원 `capacity`명인 강좌 하나와 접수됨 신청 `applicants`건.
struct Fixture {
    _dir: tempfile::TempDir,
    pool: SqlitePool,
    term_id: i64,
    course_id: i64,
    staff_id: i64,
    /// 신청 id. 접수한 순서.
    application_ids: Vec<i64>,
}

async fn fixture(capacity: i64, applicants: usize) -> Fixture {
    let (dir, pool) = open_temp().await;
    let term_id = insert_term(&pool, "2027년 1학기", "2027-03-02", "2027-06-30")
        .await
        .unwrap();
    let subject_id = insert_subject(&pool, "요가", "요가", None).await.unwrap();
    let course_id = insert_course(&pool, term_id, subject_id, None, None)
        .await
        .unwrap();
    sqlx::query("UPDATE courses SET capacity = ? WHERE id = ?")
        .bind(capacity)
        .bind(course_id)
        .execute(&pool)
        .await
        .unwrap();
    let staff_id = insert_staff(&pool, "직원1", "staff", None).await.unwrap();
    let mut application_ids = Vec::with_capacity(applicants);
    for number in 1..=applicants {
        let member_id = insert_member(&pool, &format!("{number:05}")).await.unwrap();
        let id = insert_application(&pool, member_id, course_id, "submitted", staff_id)
            .await
            .unwrap();
        application_ids.push(id);
    }
    Fixture {
        _dir: dir,
        pool,
        term_id,
        course_id,
        staff_id,
        application_ids,
    }
}

impl Fixture {
    async fn run(&self, seed: u64) -> Result<i64, RunLotteryError> {
        run(
            &self.pool,
            self.course_id,
            self.staff_id,
            LotterySeed::new(seed),
            now(),
        )
        .await
    }

    async fn lottery(&self) -> CourseLottery {
        course_lottery(&self.pool, self.course_id)
            .await
            .unwrap()
            .unwrap()
    }

    async fn statuses(&self) -> Vec<(i64, String)> {
        sqlx::query_as("SELECT id, status FROM applications ORDER BY id")
            .fetch_all(&self.pool)
            .await
            .unwrap()
    }

    fn expected(&self, capacity: i64, seed: u64) -> LotteryDraw {
        draw_lottery(
            &self.application_ids,
            Capacity::new(capacity).unwrap(),
            LotterySeed::new(seed),
            LotteryVersion::CURRENT,
        )
    }
}

impl Fixture {
    /// 신청들에 자리 보장 사유("반장")를 붙인다.
    async fn guarantee(&self, application_ids: &[i64]) {
        let reason_id = insert_priority_reason(&self.pool, "반장").await.unwrap();
        for &id in application_ids {
            sqlx::query(
                "INSERT INTO application_priorities (application_id, reason_id) VALUES (?, ?)",
            )
            .bind(id)
            .bind(reason_id)
            .execute(&self.pool)
            .await
            .unwrap();
        }
    }
}

#[tokio::test]
async fn br_s4_before_lottery_shows_guaranteed_first() {
    let f = fixture(3, 3).await;
    let last = f.application_ids[2];
    f.guarantee(&[last]).await;

    let lottery = f.lottery().await;

    assert_eq!(ids(&lottery)[0], last);
    assert!(lottery.entrants[0].guaranteed);
    assert!(!lottery.entrants[1].guaranteed);
    assert_eq!(lottery.selected_count, 0);
}

#[tokio::test]
async fn br_s4_guaranteed_are_selected_and_rest_drawn_for_remaining_seats() {
    let f = fixture(3, 6).await;
    let a = &f.application_ids;
    f.guarantee(&[a[4], a[0]]).await;

    let run_id = f.run(SEED).await.unwrap();

    let others = [a[1], a[2], a[3], a[5]];
    let expected = draw_lottery(
        &others,
        Capacity::new(1).unwrap(),
        LotterySeed::new(SEED),
        LotteryVersion::CURRENT,
    );
    let lottery = f.lottery().await;
    let mut shown = vec![a[0], a[4]];
    shown.extend_from_slice(expected.order());
    assert_eq!(ids(&lottery), shown);
    assert!(lottery.entrants[..2].iter().all(|e| e.guaranteed));
    assert!(lottery.entrants[2..].iter().all(|e| !e.guaranteed));
    assert_eq!(lottery.selected_count, 3);

    let selected: Vec<i64> = f
        .statuses()
        .await
        .into_iter()
        .filter(|(_, status)| status == "selected")
        .map(|(id, _)| id)
        .collect();
    let mut want = vec![a[0], a[4], expected.selected()[0]];
    want.sort_unstable();
    assert_eq!(selected, want);
    let recorded: Vec<i64> = sqlx::query_scalar(
        "SELECT application_id FROM lottery_guaranteed_entries
         WHERE lottery_run_id = ? ORDER BY application_id",
    )
    .bind(run_id)
    .fetch_all(&f.pool)
    .await
    .unwrap();
    assert_eq!(recorded, [a[0], a[4]]);
}

#[tokio::test]
async fn br_s4_guaranteed_filling_capacity_waitlists_everyone_else() {
    let f = fixture(2, 4).await;
    let a = &f.application_ids;
    f.guarantee(&[a[0], a[1]]).await;

    f.run(SEED).await.unwrap();

    let lottery = f.lottery().await;
    assert_eq!(lottery.selected_count, 2);
    let waitlisted = f
        .statuses()
        .await
        .into_iter()
        .filter(|(_, status)| status == "waitlisted")
        .count();
    assert_eq!(waitlisted, 2);
}

#[tokio::test]
async fn br_s5_more_guaranteed_than_capacity_keeps_previous_run() {
    let f = fixture(1, 3).await;
    let previous = f.run(SEED).await.unwrap();
    let a = &f.application_ids;
    // 추첨 뒤 붙이면 원래는 추첨이 취소되지만(BR-S13), 여기서는 DB에 바로 넣어 재추첨을 시험한다
    f.guarantee(&[a[0], a[1]]).await;

    let error = f.run(SEED).await.unwrap_err();

    assert!(
        matches!(
            error,
            RunLotteryError::OverCapacity {
                guaranteed: 2,
                capacity: 1
            }
        ),
        "{error}"
    );
    let active: i64 = sqlx::query_scalar(
        "SELECT id FROM lottery_runs WHERE course_id = ? AND cancelled_at IS NULL",
    )
    .bind(f.course_id)
    .fetch_one(&f.pool)
    .await
    .unwrap();
    assert_eq!(active, previous);
}

#[tokio::test]
async fn br_l4_redraw_returns_guaranteed_to_submitted_before_drawing_again() {
    let f = fixture(2, 3).await;
    let a = &f.application_ids;
    f.guarantee(&[a[2]]).await;
    f.run(SEED).await.unwrap();

    f.run(SEED + 1).await.unwrap();

    let recorded: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM lottery_guaranteed_entries")
        .fetch_one(&f.pool)
        .await
        .unwrap();
    assert_eq!(recorded, 2);
    assert_eq!(f.lottery().await.selected_count, 2);
    let status: String = sqlx::query_scalar("SELECT status FROM applications WHERE id = ?")
        .bind(a[2])
        .fetch_one(&f.pool)
        .await
        .unwrap();
    assert_eq!(status, "selected");
}

fn ids(lottery: &CourseLottery) -> Vec<i64> {
    lottery.entrants.iter().map(|e| e.application_id).collect()
}

#[tokio::test]
async fn before_lottery_shows_submitted_in_order() {
    let f = fixture(2, 3).await;

    let lottery = f.lottery().await;

    assert_eq!(lottery.stage, SelectionStage::NotDrawn);
    assert_eq!(ids(&lottery), f.application_ids);
    assert_eq!(lottery.selected_count, 0);
    assert_eq!(lottery.entrants[0].member_no.expose(), "00001");
}

#[tokio::test]
async fn br_s7_draw_selects_capacity_and_waitlists_rest_in_order() {
    let f = fixture(2, 5).await;

    let run_id = f.run(SEED).await.unwrap();

    let expected = f.expected(2, SEED);
    let lottery = f.lottery().await;
    assert_eq!(lottery.stage, SelectionStage::Drawn);
    assert_eq!(ids(&lottery), expected.order());
    assert_eq!(lottery.selected_count, 2);
    for (id, status) in f.statuses().await {
        let want = if expected.selected().contains(&id) {
            "selected"
        } else {
            "waitlisted"
        };
        assert_eq!(status, want, "신청 {id}");
    }
    let (seed, version, capacity): (String, i64, i64) =
        sqlx::query_as("SELECT seed, version, capacity FROM lottery_runs WHERE id = ?")
            .bind(run_id)
            .fetch_one(&f.pool)
            .await
            .unwrap();
    assert_eq!(
        (seed.as_str(), version, capacity),
        ("0123456789abcdef", 1, 2)
    );
}

#[tokio::test]
async fn br_s6_everyone_selected_when_not_over_capacity() {
    let f = fixture(5, 3).await;

    f.run(SEED).await.unwrap();

    assert_eq!(f.lottery().await.selected_count, 3);
    assert!(f.statuses().await.iter().all(|(_, s)| s == "selected"));
}

#[tokio::test]
async fn br_l2_record_reproduces_result_even_after_capacity_changes() {
    let f = fixture(2, 5).await;
    f.run(SEED).await.unwrap();
    sqlx::query("UPDATE courses SET capacity = 4")
        .execute(&f.pool)
        .await
        .unwrap();

    let lottery = f.lottery().await;

    assert_eq!(ids(&lottery), f.expected(2, SEED).order());
    assert_eq!(lottery.selected_count, 2);
}

#[tokio::test]
async fn br_l4_redraw_cancels_previous_and_draws_everyone_again() {
    let f = fixture(2, 5).await;
    let first = f.run(SEED).await.unwrap();

    let second = f.run(7).await.unwrap();

    assert_ne!(first, second);
    assert_eq!(ids(&f.lottery().await), f.expected(2, 7).order());
    let cancelled: Option<String> =
        sqlx::query_scalar("SELECT cancelled_at FROM lottery_runs WHERE id = ?")
            .bind(first)
            .fetch_one(&f.pool)
            .await
            .unwrap();
    assert!(cancelled.is_some());
    // 이전 기록의 대상과 순서는 그대로 남는다
    let kept: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM lottery_entries WHERE lottery_run_id = ?")
            .bind(first)
            .fetch_one(&f.pool)
            .await
            .unwrap();
    assert_eq!(kept, 5);
}

#[tokio::test]
async fn br_l5_closed_course_cannot_be_drawn_again() {
    let f = fixture(2, 3).await;
    f.run(SEED).await.unwrap();

    close(&f.pool, f.course_id, f.staff_id, now())
        .await
        .unwrap();

    assert_eq!(f.lottery().await.stage, SelectionStage::Closed);
    assert!(matches!(f.run(7).await, Err(RunLotteryError::Closed)));
    assert!(matches!(
        close(&f.pool, f.course_id, f.staff_id, now()).await,
        Err(CloseSelectionError::AlreadyClosed)
    ));
}

#[tokio::test]
async fn cannot_close_before_lottery() {
    let f = fixture(2, 3).await;

    let result = close(&f.pool, f.course_id, f.staff_id, now()).await;

    assert!(matches!(result, Err(CloseSelectionError::NotDrawn)));
}

#[tokio::test]
async fn no_applicants_is_not_drawn() {
    let f = fixture(2, 0).await;

    assert!(matches!(
        f.run(SEED).await,
        Err(RunLotteryError::NoApplicants)
    ));
    assert_eq!(f.lottery().await.stage, SelectionStage::NotDrawn);
}

#[tokio::test]
async fn missing_course_is_not_found() {
    let f = fixture(2, 1).await;
    let missing = f.course_id + 100;

    assert!(course_lottery(&f.pool, missing).await.unwrap().is_none());
    assert!(matches!(
        run(&f.pool, missing, f.staff_id, LotterySeed::new(SEED), now()).await,
        Err(RunLotteryError::CourseNotFound)
    ));
    assert!(matches!(
        close(&f.pool, missing, f.staff_id, now()).await,
        Err(CloseSelectionError::CourseNotFound)
    ));
}

#[tokio::test]
async fn br_s2_only_submitted_applications_are_drawn() {
    let f = fixture(2, 2).await;
    let member_id = insert_member(&f.pool, "09999").await.unwrap();
    insert_application(&f.pool, member_id, f.course_id, "withdrawn", f.staff_id)
        .await
        .unwrap();

    f.run(SEED).await.unwrap();

    assert_eq!(f.lottery().await.entrants.len(), 2);
}

#[tokio::test]
async fn stages_in_term_lists_every_course() {
    let f = fixture(2, 1).await;
    let subject_id = insert_subject(&f.pool, "한글교실", "한글교실", None)
        .await
        .unwrap();
    let other = insert_course(&f.pool, f.term_id, subject_id, None, None)
        .await
        .unwrap();
    f.run(SEED).await.unwrap();

    let stages = stages_in_term(&f.pool, f.term_id).await.unwrap();

    assert_eq!(
        stages,
        [
            CourseStage {
                course_id: f.course_id,
                stage: SelectionStage::Drawn
            },
            CourseStage {
                course_id: other,
                stage: SelectionStage::NotDrawn
            },
        ]
    );
}
