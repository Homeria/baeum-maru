//! 추첨과 선발 마감. 트랜잭션을 열고 강좌 단계 확인, 자리 보장·추첨 계산(`baeum-domain`), 기록
//! 남기기를 엮는다.
//!
//! 시드는 부른 쪽이 만들어 넘긴다(ADR 0016). 호스트 PC에서만 부르는지(BR-L6)와 역할은
//! 서버가 확인한다.

use baeum_domain::{LotterySeed, LotteryVersion, allocate};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::error::{CloseSelectionError, RunLotteryError};
use super::repository::{self, SelectionStage, cancel_active_run};
use crate::infra::db::{decode_error, timestamp_text};

/// 강좌의 접수됨 신청을 추첨하고 새 추첨 기록의 id를 돌려준다.
///
/// 이미 추첨한 강좌면 이전 기록에 취소 표시를 하고 그 신청들을 접수됨으로 되돌린 뒤 다시
/// 뽑는다(BR-L4). 읽고 확인한 뒤 쓰므로 `BEGIN IMMEDIATE`로 한다 (ADR 0005).
pub async fn run(
    pool: &SqlitePool,
    course_id: i64,
    staff_id: i64,
    seed: LotterySeed,
    now: OffsetDateTime,
) -> Result<i64, RunLotteryError> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let course = repository::course_row(&mut tx, course_id)
        .await?
        .ok_or(RunLotteryError::CourseNotFound)?;
    if course.stage() == SelectionStage::Closed {
        return Err(RunLotteryError::Closed);
    }
    let capacity = course.capacity()?;
    let now = timestamp_text(now);

    cancel_active_run(&mut tx, course_id, staff_id, &now).await?;

    let applications = repository::submitted_applications(&mut tx, course_id).await?;
    if applications.is_empty() {
        return Err(RunLotteryError::NoApplicants);
    }
    let mut guaranteed = Vec::new();
    let mut others = Vec::new();
    for (id, is_guaranteed) in applications {
        if is_guaranteed {
            guaranteed.push(id);
        } else {
            others.push(id);
        }
    }

    let version = LotteryVersion::CURRENT;
    // 정원을 넘으면 돌려주면서 tx가 사라져 위의 취소도 되돌려진다
    let allocation = allocate(&guaranteed, &others, capacity, seed, version).map_err(|error| {
        RunLotteryError::OverCapacity {
            guaranteed: error.guaranteed,
            capacity: error.capacity,
        }
    })?;
    let draw = allocation.draw();
    let run_id =
        repository::insert_run(&mut tx, course_id, staff_id, &now, seed, version, capacity).await?;

    for &application_id in allocation.guaranteed() {
        repository::insert_guaranteed_entry(&mut tx, run_id, application_id).await?;
        repository::set_drawn_status(&mut tx, application_id, true).await?;
    }
    for (index, &application_id) in draw.order().iter().enumerate() {
        // 정원이 200명까지라 순서도 i64에 들어간다
        let position = i64::try_from(index + 1).map_err(|_| decode_error("순서가 너무 큽니다"))?;
        repository::insert_entry(&mut tx, run_id, application_id, position).await?;
        let selected = index < draw.selected().len();
        repository::set_drawn_status(&mut tx, application_id, selected).await?;
    }

    tx.commit().await?;
    Ok(run_id)
}

/// 강좌의 선발을 마감한다(BR-L5). 마감 뒤에는 다시 추첨할 수 없다.
pub async fn close(
    pool: &SqlitePool,
    course_id: i64,
    staff_id: i64,
    now: OffsetDateTime,
) -> Result<(), CloseSelectionError> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let course = repository::course_row(&mut tx, course_id)
        .await?
        .ok_or(CloseSelectionError::CourseNotFound)?;
    match course.stage() {
        SelectionStage::NotDrawn => return Err(CloseSelectionError::NotDrawn),
        SelectionStage::Closed => return Err(CloseSelectionError::AlreadyClosed),
        SelectionStage::Drawn => {}
    }
    repository::insert_closure(&mut tx, course_id, staff_id, now).await?;
    tx.commit().await?;
    Ok(())
}
