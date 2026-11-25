//! 신청에 우선 배정 사유를 붙이고 떼기(BR-S12, BR-S13)와 변경 기록(BR-S14).
//!
//! 누가 붙일 수 있는지(관리자·직원)는 서버가 확인한다.

use std::collections::BTreeSet;

use sqlx::SqlitePool;
use time::OffsetDateTime;

use crate::features::applications::repository::priorities as repository;
use crate::features::lotteries::repository::cancel_active_run;
use crate::infra::db::timestamp_text;

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
    let Some(repository::PriorityTarget {
        member_id,
        course_id,
        status,
        closed,
    }) = repository::priority_target(&mut tx, application_id).await?
    else {
        return Err(SetPrioritiesError::NotFound);
    };
    if closed {
        return Err(SetPrioritiesError::Closed);
    }
    if !matches!(status.as_str(), "submitted" | "selected" | "waitlisted") {
        return Err(SetPrioritiesError::NotChangeable);
    }
    let wanted: BTreeSet<i64> = reason_ids.iter().copied().collect();
    let missing = repository::missing_reasons(&mut tx, &wanted).await?;
    if !missing.is_empty() {
        return Err(SetPrioritiesError::ReasonNotFound(missing));
    }

    let current = repository::current_reasons(&mut tx, application_id).await?;
    if current == wanted {
        return Ok(PrioritiesChanged {
            changed: false,
            lottery_cancelled: false,
        });
    }

    let now = timestamp_text(now);
    let lottery_cancelled = cancel_active_run(&mut tx, course_id, staff_id, &now).await?;
    let target = repository::ChangeTarget {
        application_id,
        member_id,
        course_id,
        staff_id,
        now: &now,
    };
    for &reason_id in current.difference(&wanted) {
        repository::detach(&mut tx, &target, reason_id).await?;
    }
    for &reason_id in wanted.difference(&current) {
        repository::attach(&mut tx, &target, reason_id).await?;
    }

    tx.commit().await?;
    Ok(PrioritiesChanged {
        changed: true,
        lottery_cancelled,
    })
}
