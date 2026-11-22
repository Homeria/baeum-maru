//! 신청 더미 데이터. 화면과 같은 저장 함수를 써서 신청 규칙을 그대로 거친다.

use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::basics::{Basics, COURSES};
use super::{SeedError, fail};
use crate::db::applications::{self, NewApplications, SubmitError};

/// [`COURSES`]에서의 자리.
const YOGA_BEGINNER_1: usize = 0;
const HARMONICA: usize = 6;
/// 요가 초급 1반(월 09:00~10:30)과 월요일 수업 시간이 겹친다.
const SMARTPHONE_BASIC: usize = 9;

/// 규칙 예외로 넣는 신청 수 (BR-A6).
const OVERRIDE_COUNT: usize = 3;
/// 넣었다가 지우는 신청 수 (BR-D1~D3).
const DELETION_COUNT: usize = 5;

pub async fn insert(
    pool: &SqlitePool,
    basics: &Basics,
    member_ids: &[i64],
    now: OffsetDateTime,
) -> Result<(), SeedError> {
    let applied = insert_by_demand(pool, basics, member_ids, now).await?;

    // 요가 초급 1반 회원 몇 명이 시간이 겹치는 스마트폰 활용 기초도 사유를 적고 신청한다
    let overridden = applied[YOGA_BEGINNER_1]
        .iter()
        .filter(|id| !applied[SMARTPHONE_BASIC].contains(id))
        .take(OVERRIDE_COUNT);
    for &member_id in overridden {
        submit(
            pool,
            basics,
            member_id,
            SMARTPHONE_BASIC,
            Some("요가 강사와 시간 조정 합의"),
            now,
        )
        .await
        .map_err(fail("규칙 예외 신청"))?;
    }

    // 하모니카는 신청이 없는 강좌로 남기되 삭제 기록만 둔다
    for &member_id in member_ids.iter().take(DELETION_COUNT) {
        let ids = submit(pool, basics, member_id, HARMONICA, None, now)
            .await
            .map_err(fail("지울 신청"))?;
        for id in ids {
            applications::delete(pool, id, basics.admin_id, Some("본인 취소"), now)
                .await
                .map_err(fail("신청 삭제"))?;
        }
    }
    Ok(())
}

/// 강좌마다 `demand`명이 될 때까지 회원을 돌아가며 신청시킨다.
/// 규칙에 걸리는 회원(시간 겹침, 같은 과목)은 건너뛴다. 강좌마다 신청한 회원 id를 돌려준다.
async fn insert_by_demand(
    pool: &SqlitePool,
    basics: &Basics,
    member_ids: &[i64],
    now: OffsetDateTime,
) -> Result<Vec<Vec<i64>>, SeedError> {
    let mut applied = Vec::with_capacity(COURSES.len());
    let mut cursor = 0;
    for (course, spec) in COURSES.iter().enumerate() {
        let mut members = Vec::with_capacity(spec.demand);
        let mut tried = 0;
        while members.len() < spec.demand {
            if tried == member_ids.len() {
                return Err(SeedError::Dummy {
                    what: "신청",
                    detail: format!("{} 강좌에 신청할 수 있는 회원이 모자랍니다", spec.subject),
                });
            }
            let member_id = member_ids[cursor % member_ids.len()];
            cursor += 1;
            tried += 1;
            match submit(pool, basics, member_id, course, None, now).await {
                Ok(_) => members.push(member_id),
                Err(SubmitError::RulesBroken(_)) => {}
                Err(error) => return Err(fail("신청")(error)),
            }
        }
        applied.push(members);
    }
    Ok(applied)
}

/// `course`는 [`COURSES`]에서의 자리다.
async fn submit(
    pool: &SqlitePool,
    basics: &Basics,
    member_id: i64,
    course: usize,
    override_reason: Option<&str>,
    now: OffsetDateTime,
) -> Result<Vec<i64>, SubmitError> {
    applications::submit(
        pool,
        NewApplications {
            term_id: basics.term_id,
            member_id,
            course_ids: &basics.course_ids[course..=course],
            override_reason,
            staff_id: basics.admin_id,
        },
        now,
    )
    .await
}
