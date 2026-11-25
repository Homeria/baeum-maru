//! 선발 마감 뒤 포기(BR-C3), 대기자 연락 결과(BR-W2~W5), 잘못 처리 바로잡기(BR-W8).
//!
//! 바꿀 때마다 선발 결과 변경 기록을 남긴다(BR-W7). 누가 할 수 있는지는 서버가 확인한다.
//! 읽고 확인한 뒤 쓰므로 모두 `BEGIN IMMEDIATE`로 한다 (ADR 0005).

use baeum_domain::{Capacity, ContactResult, WithdrawnFrom, open_seats};
use sqlx::{SqliteConnection, SqlitePool};
use time::OffsetDateTime;

use crate::features::selection_results::repository::changes as repository;
use crate::infra::db::decode_error;

/// 선발 결과를 바꾸지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum ChangeSelectionError {
    #[error("신청을 찾지 못했습니다")]
    ApplicationNotFound,
    /// 포기와 승격은 선발 마감 뒤에만 한다(BR-C3).
    #[error("선발을 마감하지 않은 강좌입니다")]
    NotClosed,
    /// 이 일을 할 수 없는 상태다. 예: 대기자가 아닌 신청에 연락 결과
    #[error("지금 상태에서는 할 수 없습니다")]
    WrongStatus,
    /// 빈자리가 없어 당첨으로 바꿀 수 없다(BR-W3, BR-W8).
    #[error("빈자리가 없습니다")]
    NoOpenSeat,
    #[error("선발 결과를 바꾸지 못했습니다: {0}")]
    Db(#[from] sqlx::Error),
}

/// 당첨이나 대기인 신청을 포기로 바꾼다. 사유는 선택이다(BR-C3).
pub async fn withdraw(
    pool: &SqlitePool,
    application_id: i64,
    reason: Option<&str>,
    staff_id: i64,
    now: OffsetDateTime,
) -> Result<(), ChangeSelectionError> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let target = target(&mut tx, application_id).await?;
    let from = match target.status.as_str() {
        "selected" => "selected",
        "waitlisted" => "waitlisted",
        _ => return Err(ChangeSelectionError::WrongStatus),
    };
    let change = repository::Change {
        application_id,
        kind: "withdrawn",
        from,
        to: "withdrawn",
        reason,
    };
    repository::apply(&mut tx, &change, staff_id, now).await?;
    tx.commit().await?;
    Ok(())
}

/// 대기자에게 연락한 결과를 남긴다(BR-W2).
///
/// 수락은 빈자리가 있을 때만 당첨으로 바꾼다(BR-W3). 거절은 포기(BR-W4), 연락 안 됨은
/// 대기 그대로다(BR-W5). 연락할 차례가 아닌 대기자도 받는다.
pub async fn record_contact(
    pool: &SqlitePool,
    application_id: i64,
    result: ContactResult,
    staff_id: i64,
    now: OffsetDateTime,
) -> Result<(), ChangeSelectionError> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let target = target(&mut tx, application_id).await?;
    if target.status != "waitlisted" {
        return Err(ChangeSelectionError::WrongStatus);
    }
    let to = match result {
        ContactResult::Accepted => {
            if seats_left(&mut tx, &target).await? == 0 {
                return Err(ChangeSelectionError::NoOpenSeat);
            }
            "selected"
        }
        ContactResult::Declined => "withdrawn",
        ContactResult::Unreachable => "waitlisted",
    };
    let change = repository::Change {
        application_id,
        kind: result.code(),
        from: "waitlisted",
        to,
        reason: None,
    };
    repository::apply(&mut tx, &change, staff_id, now).await?;
    tx.commit().await?;
    Ok(())
}

/// 잘못 처리한 포기를 포기 전 상태로 되돌린다(BR-W8). 사유는 필수다.
///
/// 당첨이었으면 빈자리가 있을 때만 되돌린다. 대기였으면 원래 대기 순번으로 돌아간다
/// (대기 순번은 추첨 기록에서 나오므로 따로 할 일이 없다).
pub async fn correct_withdrawal(
    pool: &SqlitePool,
    application_id: i64,
    reason: &str,
    staff_id: i64,
    now: OffsetDateTime,
) -> Result<(), ChangeSelectionError> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let target = target(&mut tx, application_id).await?;
    if target.status != "withdrawn" {
        return Err(ChangeSelectionError::WrongStatus);
    }
    let from = repository::withdrawn_from_status(&mut tx, application_id).await?;
    let withdrawn_from = match from.as_deref() {
        Some("selected") => WithdrawnFrom::Selected,
        Some("waitlisted") => WithdrawnFrom::Waitlisted,
        _ => return Err(decode_error("포기 기록이 없습니다").into()),
    };
    if !withdrawn_from.can_restore(seats_left(&mut tx, &target).await?) {
        return Err(ChangeSelectionError::NoOpenSeat);
    }
    let change = repository::Change {
        application_id,
        kind: "corrected",
        from: "withdrawn",
        to: match withdrawn_from {
            WithdrawnFrom::Selected => "selected",
            WithdrawnFrom::Waitlisted => "waitlisted",
        },
        reason: Some(reason),
    };
    repository::apply(&mut tx, &change, staff_id, now).await?;
    tx.commit().await?;
    Ok(())
}

/// 신청을 찾고 선발을 마감한 강좌인지 확인한다.
async fn target(
    conn: &mut SqliteConnection,
    application_id: i64,
) -> Result<repository::Target, ChangeSelectionError> {
    let target = repository::target(conn, application_id)
        .await?
        .ok_or(ChangeSelectionError::ApplicationNotFound)?;
    if !target.closed {
        return Err(ChangeSelectionError::NotClosed);
    }
    Ok(target)
}

/// 강좌의 지금 빈자리(BR-W1).
async fn seats_left(
    conn: &mut SqliteConnection,
    target: &repository::Target,
) -> Result<usize, sqlx::Error> {
    let selected = repository::selected_count(conn, target.course_id).await?;
    let capacity =
        Capacity::new(target.capacity).map_err(|_| decode_error("정원이 범위 밖입니다"))?;
    Ok(open_seats(capacity, selected))
}
