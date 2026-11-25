//! 신청 SQL 테스트.

use baeum_domain::{AdditionalOutcome, ApplicationRule, RuleViolation};
use sqlx::SqlitePool;
use time::OffsetDateTime;
use time::macros::datetime;

use crate::db::test_support::{
    insert_application, insert_course, insert_lottery_run, insert_member, insert_schedule,
    insert_staff, insert_subject, insert_term,
};
use crate::features::applications::repository::*;
use crate::features::applications::service::*;
use crate::infra::db::open_temp;

fn now() -> OffsetDateTime {
    datetime!(2026-11-22 01:02:03 UTC)
}

/// 학기 둘, 회원, 직원과 강좌 넷을 넣어 둔 DB.
///
/// - 요가 초급: 월 10:00~11:00
/// - 요가 중급: 화 10:00~11:00 (같은 과목)
/// - 한글교실: 월 10:30~12:00 (요가 초급과 시간 겹침)
/// - 국선도: 수 10:00~11:00
/// - 다른 학기 강좌 하나
struct Fixture {
    _dir: tempfile::TempDir,
    pool: SqlitePool,
    term_id: i64,
    member_id: i64,
    staff_id: i64,
    yoga_basic: i64,
    yoga_middle: i64,
    hangul: i64,
    kuksundo: i64,
    other_term_course: i64,
}

async fn fixture() -> Fixture {
    let (dir, pool) = open_temp().await;
    let term_id = insert_term(&pool, "2027년 1학기", "2027-03-02", "2027-06-30")
        .await
        .unwrap();
    let other_term = insert_term(&pool, "2027년 2학기", "2027-09-01", "2027-12-31")
        .await
        .unwrap();
    let yoga = insert_subject(&pool, "요가", "요가", None).await.unwrap();
    let hangul_subject = insert_subject(&pool, "한글교실", "한글교실", None)
        .await
        .unwrap();
    let kuksundo_subject = insert_subject(&pool, "국선도", "국선도", None)
        .await
        .unwrap();
    let course = async |term_id, subject_id, level, weekday, start, end| {
        let id = insert_course(&pool, term_id, subject_id, level, None)
            .await
            .unwrap();
        insert_schedule(&pool, id, weekday, start, end, None)
            .await
            .unwrap();
        id
    };
    let yoga_basic = course(term_id, yoga, Some("초급"), 1, 600, 660).await;
    let yoga_middle = course(term_id, yoga, Some("중급"), 2, 600, 660).await;
    let hangul = course(term_id, hangul_subject, None, 1, 630, 720).await;
    let kuksundo = course(term_id, kuksundo_subject, None, 3, 600, 660).await;
    let other_term_course = course(other_term, yoga, None, 1, 600, 660).await;
    let member_id = insert_member(&pool, "00001").await.unwrap();
    let staff_id = insert_staff(&pool, "직원1", "staff", None).await.unwrap();
    Fixture {
        _dir: dir,
        pool,
        term_id,
        member_id,
        staff_id,
        yoga_basic,
        yoga_middle,
        hangul,
        kuksundo,
        other_term_course,
    }
}

impl Fixture {
    fn applying<'a>(&self, course_ids: &'a [i64], reason: Option<&'a str>) -> NewApplications<'a> {
        NewApplications {
            term_id: self.term_id,
            member_id: self.member_id,
            course_ids,
            override_reason: reason,
            staff_id: self.staff_id,
            priorities: &[],
        }
    }

    async fn list(&self) -> Vec<ApplicationRecord> {
        list_for_member(&self.pool, self.term_id, self.member_id)
            .await
            .unwrap()
    }

    async fn count(&self, table: &'static str) -> i64 {
        let sql = match table {
            "rule_overrides" => "SELECT COUNT(*) FROM rule_overrides",
            "application_deletions" => "SELECT COUNT(*) FROM application_deletions",
            _ => "SELECT COUNT(*) FROM applications",
        };
        sqlx::query_scalar(sql).fetch_one(&self.pool).await.unwrap()
    }
}

#[tokio::test]
async fn submits_several_courses_at_once() {
    let f = fixture().await;

    let ids = submit(
        &f.pool,
        f.applying(&[f.yoga_basic, f.kuksundo], None),
        now(),
    )
    .await
    .unwrap();

    let list = f.list().await;
    assert_eq!(list.len(), 2);
    assert_eq!(
        list.iter().map(|a| a.id).collect::<Vec<_>>(),
        ids.iter().map(|item| item.id).collect::<Vec<_>>()
    );
    assert!(ids.iter().all(|item| item.additional.is_none()));
    assert_eq!(list[0].course_id, f.yoga_basic);
    assert_eq!(list[0].submitted_at, now());
    assert_eq!(list[0].overridden_rules, []);
}

#[tokio::test]
async fn br_a5_broken_rules_save_nothing() {
    let f = fixture().await;
    submit(&f.pool, f.applying(&[f.yoga_basic], None), now())
        .await
        .unwrap();

    let error = submit(&f.pool, f.applying(&[f.kuksundo, f.hangul], None), now())
        .await
        .unwrap_err();

    let SubmitError::RulesBroken(violations) = error else {
        panic!("{error:?}");
    };
    assert_eq!(
        violations,
        [RuleViolation {
            course_id: f.hangul,
            rule: ApplicationRule::NoTimeOverlap,
            other_course_id: f.yoga_basic,
        }]
    );
    assert_eq!(f.list().await.len(), 1, "국선도도 저장하지 않는다");
}

#[tokio::test]
async fn br_a6_reason_saves_with_rule_overrides() {
    let f = fixture().await;
    submit(&f.pool, f.applying(&[f.yoga_basic], None), now())
        .await
        .unwrap();

    submit(
        &f.pool,
        f.applying(&[f.yoga_middle, f.hangul], Some("강사 허락")),
        now(),
    )
    .await
    .unwrap();

    let list = f.list().await;
    assert_eq!(list[1].overridden_rules, [ApplicationRule::NoSameSubject]);
    assert_eq!(list[2].overridden_rules, [ApplicationRule::NoTimeOverlap]);
    assert_eq!(f.count("rule_overrides").await, 2);
}

#[tokio::test]
async fn reason_without_broken_rule_leaves_no_override() {
    let f = fixture().await;

    submit(&f.pool, f.applying(&[f.kuksundo], Some("사유")), now())
        .await
        .unwrap();

    assert_eq!(f.count("rule_overrides").await, 0);
}

#[tokio::test]
async fn br_a7_duplicate_is_refused_even_with_reason() {
    let f = fixture().await;
    submit(&f.pool, f.applying(&[f.yoga_basic], None), now())
        .await
        .unwrap();

    let error = submit(&f.pool, f.applying(&[f.yoga_basic], Some("사유")), now())
        .await
        .unwrap_err();

    assert!(matches!(error, SubmitError::RulesBroken(_)), "{error:?}");
    assert_eq!(f.count("applications").await, 1);
}

#[tokio::test]
async fn course_of_other_term_or_missing_is_not_found() {
    let f = fixture().await;

    let error = submit(
        &f.pool,
        f.applying(&[f.kuksundo, f.other_term_course, 999], None),
        now(),
    )
    .await
    .unwrap_err();

    let SubmitError::CourseNotFound(missing) = error else {
        panic!("{error:?}");
    };
    assert_eq!(missing, [f.other_term_course, 999]);
}

#[tokio::test]
async fn br_l3_drawn_course_takes_no_new_application() {
    let f = fixture().await;
    let other_member = insert_member(&f.pool, "00002").await.unwrap();
    insert_application(&f.pool, other_member, f.yoga_basic, "selected", f.staff_id)
        .await
        .unwrap();
    insert_lottery_run(&f.pool, f.yoga_basic, f.staff_id)
        .await
        .unwrap();
    let course_ids = [f.kuksundo, f.yoga_basic];

    let result = submit(&f.pool, f.applying(&course_ids, None), now()).await;

    assert!(
        matches!(&result, Err(SubmitError::CourseDrawn(ids)) if *ids == [f.yoga_basic]),
        "{result:?}"
    );
    assert_eq!(f.count("applications").await, 1);
}

#[tokio::test]
async fn missing_member_is_not_found() {
    let f = fixture().await;
    let course_ids = [f.kuksundo];
    let new = NewApplications {
        member_id: 999,
        ..f.applying(&course_ids, None)
    };

    let error = submit(&f.pool, new, now()).await.unwrap_err();

    assert!(matches!(error, SubmitError::MemberNotFound), "{error:?}");
}

#[tokio::test]
async fn withdrawn_application_is_left_out_of_rules_and_counts() {
    let f = fixture().await;
    insert_application(&f.pool, f.member_id, f.yoga_basic, "withdrawn", f.staff_id)
        .await
        .unwrap();

    submit(&f.pool, f.applying(&[f.yoga_basic], None), now())
        .await
        .unwrap();

    assert_eq!(f.list().await.len(), 1);
    assert_eq!(
        count_by_course(&f.pool, f.term_id).await.unwrap(),
        [CourseCount {
            course_id: f.yoga_basic,
            count: 1,
        }]
    );
}

#[tokio::test]
async fn counts_valid_applications_by_course_in_term() {
    let f = fixture().await;
    let other_member = insert_member(&f.pool, "00002").await.unwrap();
    submit(
        &f.pool,
        f.applying(&[f.yoga_basic, f.kuksundo], None),
        now(),
    )
    .await
    .unwrap();
    let course_ids = [f.yoga_basic];
    let other = NewApplications {
        member_id: other_member,
        ..f.applying(&course_ids, None)
    };
    submit(&f.pool, other, now()).await.unwrap();

    assert_eq!(
        count_by_course(&f.pool, f.term_id).await.unwrap(),
        [
            CourseCount {
                course_id: f.yoga_basic,
                count: 2,
            },
            CourseCount {
                course_id: f.kuksundo,
                count: 1,
            },
        ]
    );
}

#[tokio::test]
async fn br_d2_d3_delete_leaves_deletion_record_and_frees_rules() {
    let f = fixture().await;
    let ids = submit(&f.pool, f.applying(&[f.yoga_basic], None), now())
        .await
        .unwrap();

    delete(&f.pool, ids[0].id, f.staff_id, Some("잘못 고름"), now())
        .await
        .unwrap();

    assert_eq!(f.list().await, []);
    assert_eq!(f.count("application_deletions").await, 1);
    let (member_id, course_id, reason): (i64, i64, Option<String>) =
        sqlx::query_as("SELECT member_id, course_id, reason FROM application_deletions")
            .fetch_one(&f.pool)
            .await
            .unwrap();
    assert_eq!((member_id, course_id), (f.member_id, f.yoga_basic));
    assert_eq!(reason.as_deref(), Some("잘못 고름"));
    submit(&f.pool, f.applying(&[f.hangul], None), now())
        .await
        .unwrap();
}

#[tokio::test]
async fn br_d1_only_submitted_application_is_deleted() {
    let f = fixture().await;
    let id = insert_application(&f.pool, f.member_id, f.yoga_basic, "selected", f.staff_id)
        .await
        .unwrap();

    let error = delete(&f.pool, id, f.staff_id, None, now())
        .await
        .unwrap_err();

    assert!(
        matches!(error, DeleteApplicationError::NotSubmitted),
        "{error:?}"
    );
    assert_eq!(f.count("application_deletions").await, 0);
}

#[tokio::test]
async fn deleting_missing_application_is_not_found() {
    let f = fixture().await;

    let error = delete(&f.pool, 999, f.staff_id, None, now())
        .await
        .unwrap_err();

    assert!(
        matches!(error, DeleteApplicationError::NotFound),
        "{error:?}"
    );
}

/// 추가 신청(BR-X1~X3). 정원과 신청자 수를 정해 추첨하고 마감한 강좌 하나에 받는다.
mod additional {
    use super::*;
    use crate::db::test_support::{DrawnCourse, insert_drawn_course, insert_priority_reason};
    use crate::features::selection_results::repository::course_result;
    use crate::features::selection_results::service::changes as selection_changes;

    async fn closed_course(
        capacity: i64,
        applicants: usize,
    ) -> (tempfile::TempDir, SqlitePool, DrawnCourse) {
        let (dir, pool) = open_temp().await;
        let course = insert_drawn_course(&pool, capacity, applicants, true).await;
        (dir, pool, course)
    }

    /// 새 회원 하나가 그 강좌에 추가 신청한다.
    async fn apply_new_member(
        pool: &SqlitePool,
        course: &DrawnCourse,
        member_no: &str,
        now: OffsetDateTime,
    ) -> Result<Vec<Submitted>, SubmitError> {
        let member_id = insert_member(pool, member_no).await.unwrap();
        submit(
            pool,
            NewApplications {
                term_id: course.term_id,
                member_id,
                course_ids: &[course.course_id],
                override_reason: None,
                staff_id: course.staff_id,
                priorities: &[],
            },
            now,
        )
        .await
    }

    fn outcome(submitted: &[Submitted]) -> Option<AdditionalOutcome> {
        submitted[0].additional
    }

    async fn waitlist_numbers(pool: &SqlitePool, course_id: i64) -> Vec<usize> {
        course_result(pool, course_id)
            .await
            .unwrap()
            .waitlisted
            .iter()
            .map(|entry| entry.waitlist_number)
            .collect()
    }

    #[tokio::test]
    async fn br_x2_open_seat_without_waiting_is_selected_at_once() {
        let (_dir, pool, course) = closed_course(3, 2).await;

        let submitted = apply_new_member(&pool, &course, "10001", now())
            .await
            .unwrap();

        assert_eq!(outcome(&submitted), Some(AdditionalOutcome::Selected));
        let result = course_result(&pool, course.course_id).await.unwrap();
        assert_eq!(result.selected.len(), 3);
        assert_eq!(result.open_seats, 0);
        // 추첨 순서가 없어도 자리 보장이 아니다
        let last = result.selected.last().unwrap();
        assert_eq!(last.member.application_id, submitted[0].id);
        assert!(!last.guaranteed);
        assert_eq!(last.waitlist_number, None);
    }

    #[tokio::test]
    async fn br_x2_full_course_waits_after_last_number() {
        let (_dir, pool, course) = closed_course(2, 4).await;
        assert_eq!(waitlist_numbers(&pool, course.course_id).await, [1, 2]);

        let first = apply_new_member(&pool, &course, "10001", now())
            .await
            .unwrap();
        let second = apply_new_member(&pool, &course, "10002", now())
            .await
            .unwrap();

        assert_eq!(outcome(&first), Some(AdditionalOutcome::Waitlisted(3)));
        assert_eq!(outcome(&second), Some(AdditionalOutcome::Waitlisted(4)));
        assert_eq!(
            waitlist_numbers(&pool, course.course_id).await,
            [1, 2, 3, 4]
        );
    }

    #[tokio::test]
    async fn br_x2_open_seat_with_waiting_members_still_waits() {
        let (_dir, pool, course) = closed_course(2, 4).await;
        let result = course_result(&pool, course.course_id).await.unwrap();
        let winner = result.selected[0].member.application_id;
        selection_changes::withdraw(&pool, winner, None, course.staff_id, now())
            .await
            .unwrap();

        let submitted = apply_new_member(&pool, &course, "10001", now())
            .await
            .unwrap();

        // 빈자리는 연락을 기다리는 대기자 차례다 (BR-W1)
        assert_eq!(outcome(&submitted), Some(AdditionalOutcome::Waitlisted(3)));
    }

    #[tokio::test]
    async fn br_w6_number_of_withdrawn_waiting_member_is_not_reused() {
        let (_dir, pool, course) = closed_course(2, 3).await;
        let result = course_result(&pool, course.course_id).await.unwrap();
        let waiting = result.waitlisted[0].member.application_id;
        selection_changes::withdraw(&pool, waiting, None, course.staff_id, now())
            .await
            .unwrap();

        let submitted = apply_new_member(&pool, &course, "10001", now())
            .await
            .unwrap();

        assert_eq!(outcome(&submitted), Some(AdditionalOutcome::Waitlisted(2)));
    }

    #[tokio::test]
    async fn br_x2_later_of_two_at_once_waits_for_last_seat() {
        let (_dir, pool, course) = closed_course(2, 1).await;

        let (first, second) = tokio::join!(
            apply_new_member(&pool, &course, "10001", now()),
            apply_new_member(&pool, &course, "10002", now()),
        );

        let mut outcomes = [outcome(&first.unwrap()), outcome(&second.unwrap())];
        outcomes.sort_by_key(|outcome| format!("{outcome:?}"));
        assert_eq!(
            outcomes,
            [
                Some(AdditionalOutcome::Selected),
                Some(AdditionalOutcome::Waitlisted(1))
            ]
        );
    }

    #[tokio::test]
    async fn br_x1_term_end_date_is_last_day() {
        let (_dir, pool, course) = closed_course(3, 1).await;
        // 학기 종료일 2027-06-30. 이 PC 시간대로 그날 마지막 순간과 다음 날 0시.
        // 검사 서버는 UTC라 한국 시간을 박아 두면 안 된다
        let offset = time::UtcOffset::local_offset_at(datetime!(2027-06-30 12:00 UTC))
            .unwrap_or(time::UtcOffset::UTC);
        let last_day = datetime!(2027-06-30 23:59:59).assume_offset(offset);
        let next_day = datetime!(2027-07-01 00:00:00).assume_offset(offset);

        apply_new_member(&pool, &course, "10001", last_day)
            .await
            .unwrap();
        let result = apply_new_member(&pool, &course, "10002", next_day).await;

        assert!(
            matches!(&result, Err(SubmitError::TermEnded(ids)) if *ids == [course.course_id]),
            "{result:?}"
        );
    }

    #[tokio::test]
    async fn br_x3_rules_apply_to_additional_application() {
        let (_dir, pool, course) = closed_course(3, 1).await;
        // 이미 이 강좌에 당첨된 회원 (BR-A4)
        let member_id: i64 =
            sqlx::query_scalar("SELECT member_id FROM applications WHERE course_id = ?")
                .bind(course.course_id)
                .fetch_one(&pool)
                .await
                .unwrap();

        let result = submit(
            &pool,
            NewApplications {
                term_id: course.term_id,
                member_id,
                course_ids: &[course.course_id],
                override_reason: Some("사유"),
                staff_id: course.staff_id,
                priorities: &[],
            },
            now(),
        )
        .await;

        assert!(
            matches!(&result, Err(SubmitError::RulesBroken(_))),
            "{result:?}"
        );
    }

    #[tokio::test]
    async fn br_s13_additional_application_takes_no_priority() {
        let (_dir, pool, course) = closed_course(3, 1).await;
        let reason_id = insert_priority_reason(&pool, "반장").await.unwrap();
        let member_id = insert_member(&pool, "10001").await.unwrap();
        let reason_ids = [reason_id];
        let priorities = [CoursePriorities {
            course_id: course.course_id,
            reason_ids: &reason_ids,
        }];

        let result = submit(
            &pool,
            NewApplications {
                term_id: course.term_id,
                member_id,
                course_ids: &[course.course_id],
                override_reason: None,
                staff_id: course.staff_id,
                priorities: &priorities,
            },
            now(),
        )
        .await;

        assert!(
            matches!(&result, Err(SubmitError::PriorityForAdditional(ids)) if *ids == [course.course_id]),
            "{result:?}"
        );
    }

    #[tokio::test]
    async fn br_x2_additional_record_cannot_change() {
        let (_dir, pool, course) = closed_course(3, 1).await;
        apply_new_member(&pool, &course, "10001", now())
            .await
            .unwrap();

        let update = sqlx::query("UPDATE additional_applications SET waitlist_number = 9")
            .execute(&pool)
            .await;
        let delete = sqlx::query("DELETE FROM additional_applications")
            .execute(&pool)
            .await;

        assert!(update.is_err());
        assert!(delete.is_err());
    }
}
