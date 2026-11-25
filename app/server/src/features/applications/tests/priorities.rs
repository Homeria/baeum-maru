//! 우선 배정 사유 붙이기 SQL 테스트.

use baeum_domain::LotterySeed;
use sqlx::SqlitePool;
use time::OffsetDateTime;
use time::macros::datetime;

use crate::db::test_support::{
    insert_course, insert_member, insert_priority_reason, insert_staff, insert_subject, insert_term,
};
use crate::features::applications::repository::priorities::*;
use crate::features::applications::repository::{
    CoursePriorities, NewApplications, SubmitError, list_for_member, submit,
};
use crate::features::lotteries::repository::{SelectionStage, course_lottery};
use crate::features::lotteries::service as lotteries;
use crate::infra::db::open_temp;

fn now() -> OffsetDateTime {
    datetime!(2026-11-22 01:02:03 UTC)
}

struct Fixture {
    _dir: tempfile::TempDir,
    pool: SqlitePool,
    term_id: i64,
    course_id: i64,
    members: Vec<i64>,
    staff_id: i64,
    leader: i64,
    disability: i64,
}

/// 학기, 정원 20명 강좌 하나, 회원 셋, 직원, 사유 둘(반장, 장애인 우대).
async fn fixture() -> Fixture {
    let (dir, pool) = open_temp().await;
    let term_id = insert_term(&pool, "2027년 1학기", "2027-03-02", "2027-06-30")
        .await
        .unwrap();
    let subject_id = insert_subject(&pool, "요가", "요가", None).await.unwrap();
    let course_id = insert_course(&pool, term_id, subject_id, None, None)
        .await
        .unwrap();
    let mut members = Vec::new();
    for no in ["00001", "00002", "00003"] {
        members.push(insert_member(&pool, no).await.unwrap());
    }
    let staff_id = insert_staff(&pool, "직원1", "staff", None).await.unwrap();
    let leader = insert_priority_reason(&pool, "반장").await.unwrap();
    let disability = insert_priority_reason(&pool, "장애인 우대").await.unwrap();
    Fixture {
        _dir: dir,
        pool,
        term_id,
        course_id,
        members,
        staff_id,
        leader,
        disability,
    }
}

impl Fixture {
    /// 회원 `index`의 신청을 사유와 함께 내고 신청 id를 돌려준다.
    async fn apply(&self, index: usize, reason_ids: &[i64]) -> i64 {
        let course_ids = [self.course_id];
        let priorities = [CoursePriorities {
            course_id: self.course_id,
            reason_ids,
        }];
        let ids = submit(
            &self.pool,
            NewApplications {
                term_id: self.term_id,
                member_id: self.members[index],
                course_ids: &course_ids,
                override_reason: None,
                staff_id: self.staff_id,
                priorities: &priorities,
            },
            now(),
        )
        .await
        .unwrap();
        ids[0].id
    }

    async fn reasons_of(&self, index: usize) -> Vec<i64> {
        let records = list_for_member(&self.pool, self.term_id, self.members[index])
            .await
            .unwrap();
        records[0].priority_reason_ids.clone()
    }

    /// (붙임/뗌, 사유) 기록을 남긴 순서로.
    async fn changes(&self) -> Vec<(String, i64)> {
        sqlx::query_as("SELECT change, reason_id FROM priority_changes ORDER BY id")
            .fetch_all(&self.pool)
            .await
            .unwrap()
    }

    async fn stage(&self) -> SelectionStage {
        course_lottery(&self.pool, self.course_id)
            .await
            .unwrap()
            .unwrap()
            .stage
    }
}

#[tokio::test]
async fn br_s12_submit_attaches_several_reasons_and_records_them() {
    let f = fixture().await;

    f.apply(0, &[f.disability, f.leader, f.leader]).await;

    assert_eq!(f.reasons_of(0).await, [f.leader, f.disability]);
    assert_eq!(
        f.changes().await,
        [
            ("added".to_owned(), f.leader),
            ("added".to_owned(), f.disability)
        ]
    );
}

#[tokio::test]
async fn submit_refuses_unknown_reason_or_unchosen_course() {
    let f = fixture().await;
    let course_ids = [f.course_id];
    let submit_with = |priorities: Vec<CoursePriorities<'static>>| {
        let pool = f.pool.clone();
        let (term_id, member_id, staff_id) = (f.term_id, f.members[0], f.staff_id);
        async move {
            submit(
                &pool,
                NewApplications {
                    term_id,
                    member_id,
                    course_ids: &course_ids,
                    override_reason: None,
                    staff_id,
                    priorities: &priorities,
                },
                now(),
            )
            .await
        }
    };

    let unknown = submit_with(vec![CoursePriorities {
        course_id: f.course_id,
        reason_ids: &[999],
    }])
    .await
    .unwrap_err();
    let unchosen = submit_with(vec![CoursePriorities {
        course_id: 777,
        reason_ids: &[],
    }])
    .await
    .unwrap_err();

    assert!(
        matches!(&unknown, SubmitError::PriorityReasonNotFound(ids) if *ids == [999]),
        "{unknown}"
    );
    assert!(
        matches!(&unchosen, SubmitError::PriorityForUnchosenCourse(ids) if *ids == [777]),
        "{unchosen}"
    );
    assert!(f.changes().await.is_empty());
}

#[tokio::test]
async fn br_s13_and_br_s14_set_adds_and_removes_with_records() {
    let f = fixture().await;
    let id = f.apply(0, &[f.leader]).await;

    let changed = set(&f.pool, id, &[f.disability], f.staff_id, now())
        .await
        .unwrap();

    assert_eq!(
        changed,
        PrioritiesChanged {
            changed: true,
            lottery_cancelled: false
        }
    );
    assert_eq!(f.reasons_of(0).await, [f.disability]);
    assert_eq!(
        f.changes().await,
        [
            ("added".to_owned(), f.leader),
            ("removed".to_owned(), f.leader),
            ("added".to_owned(), f.disability),
        ]
    );
}

#[tokio::test]
async fn same_reasons_change_nothing() {
    let f = fixture().await;
    let id = f.apply(0, &[f.leader]).await;

    let changed = set(&f.pool, id, &[f.leader], f.staff_id, now())
        .await
        .unwrap();

    assert!(!changed.changed);
    assert_eq!(f.changes().await.len(), 1);
}

#[tokio::test]
async fn br_s13_changing_drawn_course_cancels_its_lottery() {
    let f = fixture().await;
    let id = f.apply(0, &[]).await;
    f.apply(1, &[]).await;
    lotteries::run(&f.pool, f.course_id, f.staff_id, LotterySeed::new(1), now())
        .await
        .unwrap();
    assert_eq!(f.stage().await, SelectionStage::Drawn);

    let changed = set(&f.pool, id, &[f.leader], f.staff_id, now())
        .await
        .unwrap();

    assert!(changed.lottery_cancelled);
    assert_eq!(f.stage().await, SelectionStage::NotDrawn);
    let statuses: Vec<String> = sqlx::query_scalar("SELECT status FROM applications")
        .fetch_all(&f.pool)
        .await
        .unwrap();
    assert_eq!(statuses, ["submitted", "submitted"]);
}

#[tokio::test]
async fn br_s13_closed_course_cannot_change() {
    let f = fixture().await;
    let id = f.apply(0, &[]).await;
    lotteries::run(&f.pool, f.course_id, f.staff_id, LotterySeed::new(1), now())
        .await
        .unwrap();
    lotteries::close(&f.pool, f.course_id, f.staff_id, now())
        .await
        .unwrap();

    let error = set(&f.pool, id, &[f.leader], f.staff_id, now())
        .await
        .unwrap_err();

    assert!(matches!(error, SetPrioritiesError::Closed), "{error}");
}

#[tokio::test]
async fn set_refuses_unknown_application_or_reason() {
    let f = fixture().await;
    let id = f.apply(0, &[]).await;

    let unknown_application = set(&f.pool, 999, &[], f.staff_id, now()).await.unwrap_err();
    let unknown_reason = set(&f.pool, id, &[f.leader, 999], f.staff_id, now())
        .await
        .unwrap_err();

    assert!(
        matches!(unknown_application, SetPrioritiesError::NotFound),
        "{unknown_application}"
    );
    assert!(
        matches!(&unknown_reason, SetPrioritiesError::ReasonNotFound(ids) if *ids == [999]),
        "{unknown_reason}"
    );
    assert!(f.reasons_of(0).await.is_empty());
}
