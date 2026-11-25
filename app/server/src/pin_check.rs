//! 직원 PIN 확인 (BR-K5~K7, BR-K13). 로그인, 잠금 풀기, 기기 허락, 실행 모드 바꾸기,
//! 호스트 넘기기와 넘기기 취소가 함께 쓴다.
//!
//! HTTP를 모른다. 서버는 [`PinRefusal`]을 `ApiError`로, Tauri 명령은 문장으로 바꾼다.

use baeum_domain::is_account_locked;
use sqlx::SqlitePool;
use time::OffsetDateTime;

use crate::db::access_logs::{self, AccessEvent};
use crate::db::staff::{self, LoginAccount};
use crate::infra::secret;

/// PIN 확인을 거절했다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PinRefusal {
    /// PIN이 틀렸다.
    #[error("{}", self.message())]
    WrongPin,
    /// BR-K7: PIN을 5번 연속 틀려 계정이 잠겼다.
    #[error("{}", self.message())]
    AccountLocked,
    /// BR-K5, BR-K6: 사용 중지했거나 유효기간이 지난 계정이다.
    #[error("{}", self.message())]
    AccountUnavailable,
    /// DB를 읽거나 쓰지 못했다. 원인은 로그에 남겼다.
    #[error("{}", self.message())]
    Internal,
}

impl PinRefusal {
    /// 화면에 보여 줄 문장. 서버 응답과 Tauri 명령이 같은 문장을 쓴다.
    pub fn message(self) -> &'static str {
        match self {
            PinRefusal::WrongPin => "PIN이 맞지 않습니다. 5번 연속 틀리면 5분 동안 잠깁니다",
            PinRefusal::AccountLocked => {
                "PIN을 5번 연속 틀려 잠겼습니다. 5분이 지나면 다시 할 수 있습니다"
            }
            PinRefusal::AccountUnavailable => {
                "사용 중지했거나 유효기간이 지난 계정입니다. 관리자에게 물어 주세요"
            }
            PinRefusal::Internal => "PIN을 확인하지 못했습니다",
        }
    }
}

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

    let failed = staff::add_failed_pin(db, account.id, device_id, failed_event, now)
        .await
        .map_err(internal)?;
    Err(if failed.locked_until.is_some() {
        PinRefusal::AccountLocked
    } else {
        PinRefusal::WrongPin
    })
}

fn internal(error: impl std::fmt::Display) -> PinRefusal {
    tracing::error!(%error, "PIN을 확인하지 못했습니다");
    PinRefusal::Internal
}
