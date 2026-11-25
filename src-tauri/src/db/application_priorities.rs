//! 신청에 우선 배정 사유를 붙이고 떼기(BR-S12, BR-S13)와 변경 기록(BR-S14).
//!
//! 누가 붙일 수 있는지(관리자·직원)는 서버가 확인한다.

use std::collections::BTreeSet;

use sqlx::{SqliteConnection, SqlitePool};
use time::OffsetDateTime;

use super::lotteries::cancel_active_run;
use super::timestamp_text;

/// 사유를 바꾸지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum SetPrioritiesError {
    #[error("신청을 찾지 못했습니다")]
    NotFound,
    /// 유효한 신청(접수됨, 당첨, 대기)이 아니다. 확정은 마감 뒤라 여기 오지 않는다.
    #[error("우선 배정을 바꿀 수 없는 신청입니다")]
    NotChangeable,
    /// BR-S13: 선발 마감 뒤에는 바꿀 수 없다.
    #[error("선발을 마감한 강좌입니다")]
    Closed,
    /// 목록에 없는 사유. 그 사이 지워졌을 수 있다.
    #[error("목록에 없는 사유를 골랐습니다")]
    ReasonNotFound(Vec<i64>),
    #[error("우선 배정을 바꾸지 못했습니다: {0}")]
    Db(#[from] sqlx::Error),
}

/// 사유를 바꾼 결과.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrioritiesChanged {
    /// 바뀐 것이 있었는지. 같은 사유를 다시 보내면 `false`이고 아무 기록도 남지 않는다.
    pub changed: bool,
    /// BR-S13: 추첨한 강좌라 추첨이 취소되고 추첨 전으로 돌아갔는지.
    pub lottery_cancelled: bool,
}

/// 신청의 사유를 `reason_ids`로 바꾼다. 빈 목록이면 모두 뗀다.
///
/// 추첨한 강좌면 그 추첨을 취소한다. 읽고 확인한 뒤 쓰므로 `BEGIN IMMEDIATE`로 한다.
pub async fn set(
    pool: &SqlitePool,
    application_id: i64,
    reason_ids: &[i64],
    staff_id: i64,
    now: OffsetDateTime,
) -> Result<PrioritiesChanged, SetPrioritiesError> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let row: Option<(i64, i64, String, bool)> = sqlx::query_as(
        "SELECT a.member_id, a.course_id, a.status,
                EXISTS (SELECT 1 FROM selection_closures s WHERE s.course_id = a.course_id)
         FROM applications a WHERE a.id = ?",
    )
    .bind(application_id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((member_id, course_id, status, closed)) = row else {
        return Err(SetPrioritiesError::NotFound);
    };
    if closed {
        return Err(SetPrioritiesError::Closed);
    }
    if !matches!(status.as_str(), "submitted" | "selected" | "waitlisted") {
        return Err(SetPrioritiesError::NotChangeable);
    }
    let wanted: BTreeSet<i64> = reason_ids.iter().copied().collect();
    let missing = missing_reasons(&mut tx, &wanted).await?;
    if !missing.is_empty() {
        return Err(SetPrioritiesError::ReasonNotFound(missing));
    }

    let current: BTreeSet<i64> =
        sqlx::query_scalar("SELECT reason_id FROM application_priorities WHERE application_id = ?")
            .bind(application_id)
            .fetch_all(&mut *tx)
            .await?
            .into_iter()
            .collect();
    if current == wanted {
        return Ok(PrioritiesChanged {
            changed: false,
            lottery_cancelled: false,
        });
    }

    let now = timestamp_text(now);
    let lottery_cancelled = cancel_active_run(&mut tx, course_id, staff_id, &now).await?;
    let target = ChangeTarget {
        application_id,
        member_id,
        course_id,
        staff_id,
        now: &now,
    };
    for &reason_id in current.difference(&wanted) {
        sqlx::query(
            "DELETE FROM application_priorities WHERE application_id = ? AND reason_id = ?",
        )
        .bind(application_id)
        .bind(reason_id)
        .execute(&mut *tx)
        .await?;
        record_change(&mut tx, &target, reason_id, "removed").await?;
    }
    for &reason_id in wanted.difference(&current) {
        attach(&mut tx, &target, reason_id).await?;
    }

    tx.commit().await?;
    Ok(PrioritiesChanged {
        changed: true,
        lottery_cancelled,
    })
}

/// 변경 기록에 남길 신청과 직원.
pub(super) struct ChangeTarget<'a> {
    pub application_id: i64,
    pub member_id: i64,
    pub course_id: i64,
    pub staff_id: i64,
    /// [`timestamp_text`] 글자.
    pub now: &'a str,
}

/// 사유 하나를 붙이고 기록한다. 부르는 쪽이 트랜잭션을 연다.
pub(super) async fn attach(
    conn: &mut SqliteConnection,
    target: &ChangeTarget<'_>,
    reason_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO application_priorities (application_id, reason_id) VALUES (?, ?)")
        .bind(target.application_id)
        .bind(reason_id)
        .execute(&mut *conn)
        .await?;
    record_change(conn, target, reason_id, "added").await
}

/// `ids` 가운데 목록에 없는 사유. 작은 id부터.
pub(super) async fn missing_reasons(
    conn: &mut SqliteConnection,
    ids: &BTreeSet<i64>,
) -> Result<Vec<i64>, sqlx::Error> {
    let known: BTreeSet<i64> = sqlx::query_scalar("SELECT id FROM priority_reasons")
        .fetch_all(conn)
        .await?
        .into_iter()
        .collect();
    Ok(ids.difference(&known).copied().collect())
}

async fn record_change(
    conn: &mut SqliteConnection,
    target: &ChangeTarget<'_>,
    reason_id: i64,
    change: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO priority_changes
             (application_id, member_id, course_id, reason_id, change, changed_by, changed_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(target.application_id)
    .bind(target.member_id)
    .bind(target.course_id)
    .bind(reason_id)
    .bind(change)
    .bind(target.staff_id)
    .bind(target.now)
    .execute(conn)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use time::macros::datetime;

    use super::*;
    use crate::db::applications::{
        CoursePriorities, NewApplications, SubmitError, list_for_member, submit,
    };
    use crate::db::lotteries::{self, SelectionStage};
    use crate::db::open_temp;
    use crate::db::test_support::{
        insert_course, insert_member, insert_priority_reason, insert_staff, insert_subject,
        insert_term,
    };
    use baeum_domain::LotterySeed;

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
            lotteries::course_lottery(&self.pool, self.course_id)
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
}
