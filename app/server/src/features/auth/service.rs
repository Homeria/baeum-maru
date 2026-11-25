//! 직원 PIN 확인 (BR-K5~K7, BR-K13)과 로그인·로그아웃·잠금 풀기·복구 코드 쓰기의 트랜잭션.
//! PIN 확인은 로그인, 잠금 풀기, 기기 승인, 실행 모드 바꾸기, 호스트 넘기기와 넘기기 취소가
//! 함께 쓴다.
//!
//! HTTP를 모른다. 서버는 [`PinRefusal`]을 `ApiError`로, Tauri 명령은 문장으로 바꾼다.

use baeum_domain::{FailedPin, PinRefusal, SCREEN_LOCK, is_account_locked, record_failed_pin};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::repository::{self, LoginAccount};
use crate::features::access_logs::repository::{self as access_logs, AccessEvent};
use crate::infra::db::timestamp_text;
use crate::infra::secret;

/// 맞으면 계정을 그대로 돌려준다. `device_id`는 요청한 기기, 호스트 PC 자신이면 `None`.
///
/// 계정을 쓸 수 없거나(BR-K5, BR-K6) 잠겼으면(BR-K7) PIN을 확인하지 않는다.
/// 실패는 모두 `failed_event`로 접속 기록에 남긴다 (BR-K13).
///
/// # Errors
///
/// 거절한 까닭. DB 문제는 [`PinRefusal::Internal`].
pub async fn check_pin(
    db: &SqlitePool,
    account: LoginAccount,
    pin: String,
    device_id: Option<i64>,
    failed_event: AccessEvent,
    now: OffsetDateTime,
) -> Result<LoginAccount, PinRefusal> {
    let refusal = if !account.is_usable() {
        Some(PinRefusal::AccountUnavailable)
    } else if is_account_locked(account.locked_until, now) {
        Some(PinRefusal::AccountLocked)
    } else {
        None
    };
    if let Some(refusal) = refusal {
        access_logs::record_on_device(db, Some(account.id), device_id, failed_event, now)
            .await
            .map_err(internal)?;
        return Err(refusal);
    }

    // 해시 확인은 일부러 느려서 따로 돌린다. 계정을 넘겼다가 돌려받는다
    let (account, pin_matches) = tokio::task::spawn_blocking(move || {
        let pin_matches = secret::verify(&pin, &account.pin_hash);
        (account, pin_matches)
    })
    .await
    .map_err(internal)?;
    if pin_matches {
        return Ok(account);
    }

    let failed = add_failed_pin(db, account.id, device_id, failed_event, now)
        .await
        .map_err(internal)?;
    Err(if failed.locked_until.is_some() {
        PinRefusal::AccountLocked
    } else {
        PinRefusal::WrongPin
    })
}

/// BR-K7: `staff_id`인 직원이 PIN을 틀렸다. 틀린 횟수를 세고(5번째면 잠그고)
/// 접속 기록에 `event`(로그인 실패나 잠금 풀기 실패)를 남긴 뒤, 바뀐 상태를 돌려준다.
///
/// 동시에 틀려도 횟수를 빠뜨리지 않게 읽기와 쓰기를 `BEGIN IMMEDIATE` 하나에서 한다.
pub async fn add_failed_pin(
    pool: &SqlitePool,
    staff_id: i64,
    device_id: Option<i64>,
    event: AccessEvent,
    now: OffsetDateTime,
) -> Result<FailedPin, sqlx::Error> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

    let failed_count = repository::failed_pin_count(&mut tx, staff_id).await?;
    let failed = record_failed_pin(failed_count, now);

    repository::set_failed_pin(&mut tx, staff_id, &failed).await?;
    access_logs::insert_on_device(
        &mut tx,
        Some(staff_id),
        device_id,
        event,
        &timestamp_text(now),
    )
    .await?;

    tx.commit().await?;
    Ok(failed)
}

/// BR-K8: PIN을 맞게 넣어 잠긴 세션을 연다.
/// BR-K7: 틀린 횟수도 0으로 돌린다 (로그인에서 맞게 넣은 것과 같다).
pub async fn unlock(
    pool: &SqlitePool,
    session_id: i64,
    staff_id: i64,
    now: OffsetDateTime,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    repository::clear_failed_pin(&mut tx, staff_id).await?;
    repository::set_last_seen(&mut tx, session_id, now).await?;
    tx.commit().await
}

/// `staff_id`인 직원의 로그인을 기록한다.
///
/// 한 트랜잭션에서 다음을 함께 한다.
/// - BR-K7: 틀린 횟수와 잠금을 푼다.
/// - ADR 0014: 그 직원의 잠긴(마지막 요청에서 10분 지난) 세션을 지워 정리한다.
/// - 새 세션을 저장하고 접속 기록에 로그인을 남긴다.
pub async fn start(
    pool: &SqlitePool,
    staff_id: i64,
    device_id: Option<i64>,
    token_hash: &[u8; 32],
    now: OffsetDateTime,
) -> Result<(), sqlx::Error> {
    let locked_before = timestamp_text(now - SCREEN_LOCK);
    let now = timestamp_text(now);
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

    repository::clear_failed_pin(&mut tx, staff_id).await?;
    repository::delete_locked_sessions(&mut tx, staff_id, &locked_before).await?;
    repository::insert_session(&mut tx, staff_id, token_hash, &now).await?;
    access_logs::insert_on_device(&mut tx, Some(staff_id), device_id, AccessEvent::Login, &now)
        .await?;

    tx.commit().await
}

/// 증표 해시가 `token_hash`인 세션을 지우고 접속 기록에 로그아웃을 남긴 뒤
/// 그 세션의 직원 id를 돌려준다. 그런 세션이 없으면 아무것도 하지 않고 `None`.
///
/// 잠긴 세션도 지운다. BR-K8: 다른 직원은 로그아웃한 뒤 로그인한다.
pub async fn end(
    pool: &SqlitePool,
    token_hash: &[u8; 32],
    now: OffsetDateTime,
) -> Result<Option<i64>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let Some(staff_id) = repository::delete_session(&mut tx, token_hash).await? else {
        return Ok(None);
    };
    access_logs::insert(
        &mut tx,
        Some(staff_id),
        AccessEvent::Logout,
        &timestamp_text(now),
    )
    .await?;

    tx.commit().await?;
    Ok(Some(staff_id))
}

/// BR-K12: 복구 코드를 써서 `staff_id`인 관리자의 PIN을 다시 정하고 코드를 새것으로 바꾼다.
///
/// 한 트랜잭션에서 다음을 함께 한다.
/// - 복구 코드 해시를 `used_code_hash`에서 `new_code_hash`로 바꾼다. 그사이 다른 요청이
///   먼저 썼으면 바뀌는 줄이 없어서 아무것도 하지 않고 `false`를 돌려준다.
/// - 관리자 PIN을 바꾸고 틀린 횟수와 잠금(BR-K7)을 푼다.
/// - 접속 기록에 복구 코드 사용을 남긴다 (BR-K13).
pub async fn use_code(
    pool: &SqlitePool,
    staff_id: i64,
    used_code_hash: &str,
    new_code_hash: &str,
    new_pin_hash: &str,
    now: OffsetDateTime,
) -> Result<bool, sqlx::Error> {
    let now = timestamp_text(now);
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

    if !repository::replace_code(&mut tx, used_code_hash, new_code_hash, &now).await? {
        return Ok(false);
    }
    repository::set_admin_pin(&mut tx, staff_id, new_pin_hash, &now).await?;
    access_logs::insert(&mut tx, Some(staff_id), AccessEvent::RecoveryUsed, &now).await?;

    tx.commit().await?;
    Ok(true)
}

fn internal(error: impl std::fmt::Display) -> PinRefusal {
    tracing::error!(%error, "PIN을 확인하지 못했습니다");
    PinRefusal::Internal
}
