//! 선발 결과 더미 데이터. 서예 강좌 하나를 추첨하고 마감한 뒤 포기와 연락 결과를 남긴다.
//!
//! 화면과 같은 저장 함수를 써서 규칙을 그대로 거친다. 다른 강좌는 추첨 전으로 둔다.
//! 끝나면 빈자리 1, 대기 1번은 연락 안 됨이라 건너뛰고 대기 3번이 연락할 차례다.

use baeum_domain::{ContactResult, LotterySeed};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::basics::Basics;
use super::{SeedError, fail};
use crate::features::lotteries::service as lotteries;
use crate::features::selection_results::repository::changes as selection_changes;
use crate::features::selection_results::repository::course_result;

/// [`super::basics::COURSES`]에서의 자리. 정원 15명에 18명이 신청해 대기 3명이 생긴다.
const CALLIGRAPHY: usize = 7;
/// 늘 같은 결과가 나오게 고정한다.
const SEED: u64 = 2027;

pub async fn insert(
    pool: &SqlitePool,
    basics: &Basics,
    now: OffsetDateTime,
) -> Result<(), SeedError> {
    let course_id = basics.course_ids[CALLIGRAPHY];
    let staff_id = basics.admin_id;
    lotteries::run(pool, course_id, staff_id, LotterySeed::new(SEED), now)
        .await
        .map_err(fail("추첨"))?;
    lotteries::close(pool, course_id, staff_id, now)
        .await
        .map_err(fail("선발 마감"))?;

    let result = course_result(pool, course_id)
        .await
        .map_err(fail("선발 결과"))?;
    let [.., leaving, leaving_quietly] = result.selected.as_slice() else {
        return Err(fail("선발 결과")("당첨자가 2명보다 적습니다"));
    };
    let [first, second, ..] = result.waitlisted.as_slice() else {
        return Err(fail("선발 결과")("대기자가 2명보다 적습니다"));
    };

    // 당첨자 둘이 포기해 빈자리 2
    selection_changes::withdraw(
        pool,
        leaving.member.application_id,
        Some("건강 문제"),
        staff_id,
        now,
    )
    .await
    .map_err(fail("포기"))?;
    selection_changes::withdraw(
        pool,
        leaving_quietly.member.application_id,
        None,
        staff_id,
        now,
    )
    .await
    .map_err(fail("포기"))?;
    // 대기 1번은 전화를 받지 않고, 2번이 수락해 빈자리 1이 남는다
    let contacts = [
        (first, ContactResult::Unreachable),
        (second, ContactResult::Accepted),
    ];
    for (waiting, contact) in contacts {
        selection_changes::record_contact(
            pool,
            waiting.member.application_id,
            contact,
            staff_id,
            now,
        )
        .await
        .map_err(fail("연락 결과"))?;
    }
    Ok(())
}
