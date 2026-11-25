//! 로그인, 로그아웃, 화면 잠금 풀기, 복구 코드로 PIN 다시 정하기 (BR-K1, BR-K5~K8, BR-K12, BR-K13).

use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRef, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use baeum_domain::{Pin, PinRefusal, RecoveryCode, Role};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::dto::{
    LoginRequest, LoginResponse, LoginStaffListResponse, LoginStaffResponse, RecoveryRequest,
    RecoveryResponse, SignedInStaffResponse, UnlockRequest,
};
use super::repository;
use super::service::check_pin;
use crate::features::access_logs::repository::{self as access_logs, AccessEvent};
use crate::http::auth::{CurrentDevice, bearer_token};
use crate::http::error::{ApiError, FieldError};
use crate::infra::clock::local_today;
use crate::infra::secret::{self, AdminSecrets, SessionToken};

/// 로그인 전에 받는 주소. 기기 확인 안, 로그인 확인 바깥에 들어간다.
///
/// 로그인 화면과 잠긴 화면은 로그인 확인을 거치지 않고 스스로 세션을 본다.
/// PIN을 잊은 관리자는 로그인하지 못하므로 복구도 여기서 받는다.
pub(crate) fn routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    SqlitePool: FromRef<S>,
{
    Router::new()
        .route("/api/login/staff", get(list_login_staff))
        .route("/api/login", post(login))
        .route("/api/unlock", post(unlock))
        .route("/api/logout", post(logout))
        .route("/api/recovery", post(recover))
}

/// PIN 확인 거절을 같은 문장의 응답으로 바꾼다.
impl From<PinRefusal> for ApiError {
    fn from(refusal: PinRefusal) -> Self {
        match refusal {
            PinRefusal::WrongPin => ApiError::WrongPin,
            PinRefusal::AccountLocked => ApiError::AccountLocked,
            PinRefusal::AccountUnavailable => ApiError::AccountUnavailable,
            PinRefusal::Internal => ApiError::Internal,
        }
    }
}

/// 로그인 화면에서 고를 직원을 이름 순서로 보여 준다.
/// 사용 중지했거나 유효기간이 지난 계정은 빠진다.
async fn list_login_staff(
    State(db): State<SqlitePool>,
) -> Result<Json<LoginStaffListResponse>, ApiError> {
    let today = local_today();
    let choices = repository::list_for_login(&db, today)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(LoginStaffListResponse {
        items: choices.into_iter().map(LoginStaffResponse::from).collect(),
    }))
}

/// 고른 직원의 PIN을 확인하고 세션 증표를 준다.
///
/// 실패는 모두 접속 기록에 남긴다 (BR-K13). 잠겼거나 쓸 수 없는 계정은 PIN을 확인하지 않는다.
async fn login(
    State(db): State<SqlitePool>,
    device: CurrentDevice,
    body: Result<Json<LoginRequest>, JsonRejection>,
) -> Result<Json<LoginResponse>, ApiError> {
    let Json(LoginRequest { staff_id, pin }) = body?;
    let now = OffsetDateTime::now_utc();

    let Some(account) = repository::get_login_account(&db, staff_id)
        .await
        .map_err(ApiError::internal)?
    else {
        access_logs::record_on_device(&db, None, device.id, AccessEvent::LoginFailed, now)
            .await
            .map_err(ApiError::internal)?;
        return Err(ApiError::StaffNotFound);
    };
    let account = check_pin(&db, account, pin, device.id, AccessEvent::LoginFailed, now).await?;

    let token = SessionToken::new().map_err(ApiError::internal)?;
    repository::start(&db, account.id, device.id, &token.hash(), now)
        .await
        .map_err(ApiError::internal)?;
    tracing::info!(staff_id = account.id, "로그인");

    Ok(Json(LoginResponse {
        token: token.to_hex(),
        staff: SignedInStaffResponse {
            id: account.id,
            name: account.name,
            role: account.role.into(),
        },
    }))
}

/// BR-K8: 잠긴 세션을 같은 직원의 PIN으로 연다. 성공하면 204(본문 없음).
///
/// 잠기지 않은 세션에 보내도 PIN이 맞으면 그대로 성공한다. 틀리면 BR-K7의 횟수에 들어간다.
async fn unlock(
    State(db): State<SqlitePool>,
    device: CurrentDevice,
    headers: HeaderMap,
    body: Result<Json<UnlockRequest>, JsonRejection>,
) -> Result<StatusCode, ApiError> {
    let token = bearer_token(&headers).ok_or(ApiError::LoginRequired)?;
    let Json(UnlockRequest { pin }) = body?;
    let now = OffsetDateTime::now_utc();

    let session = repository::find(&db, &token.hash())
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::LoginRequired)?;
    let account = check_pin(
        &db,
        session.account,
        pin,
        device.id,
        AccessEvent::UnlockFailed,
        now,
    )
    .await?;

    repository::unlock(&db, session.id, account.id, now)
        .await
        .map_err(ApiError::internal)?;
    tracing::info!(staff_id = account.id, "화면 잠금 풀기");
    Ok(StatusCode::NO_CONTENT)
}

/// 요청에 실린 세션을 끝낸다. 늘 204(본문 없음)를 돌려준다.
///
/// 세션이 이미 없어도(지워졌거나 증표가 없어도) 성공으로 본다. 앱은 어차피 증표를 버린다.
async fn logout(State(db): State<SqlitePool>, headers: HeaderMap) -> Result<StatusCode, ApiError> {
    if let Some(token) = bearer_token(&headers) {
        let ended = repository::end(&db, &token.hash(), OffsetDateTime::now_utc())
            .await
            .map_err(ApiError::internal)?;
        if let Some(staff_id) = ended {
            tracing::info!(staff_id, "로그아웃");
        }
    }
    Ok(StatusCode::NO_CONTENT)
}

/// `/api/recovery`: 복구 코드로 관리자 PIN 다시 정하기 (BR-K12).
/// PIN을 잊어 로그인하지 못한 관리자가 쓰므로 로그인 없이 받는다.
///
/// 복구 코드가 맞으면 고른 관리자의 PIN을 새로 정하고 새 복구 코드를 돌려준다.
///
/// 쓴 코드는 버린다. 계정 잠금(BR-K7)도 풀린다. 다른 역할의 직원은 403.
async fn recover(
    State(db): State<SqlitePool>,
    body: Result<Json<RecoveryRequest>, JsonRejection>,
) -> Result<Json<RecoveryResponse>, ApiError> {
    let Json(RecoveryRequest {
        staff_id,
        code,
        pin,
    }) = body?;
    let (code, pin) = parse_input(&code, &pin)?;

    let account = repository::get_login_account(&db, staff_id)
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::StaffNotFound)?;
    if account.role != Role::Admin {
        return Err(ApiError::RecoveryAdminOnly);
    }
    if !account.is_usable() {
        return Err(ApiError::AccountUnavailable);
    }
    let stored = repository::code_hash(&db)
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::WrongRecoveryCode)?;

    // 해시 확인 한 번과 새로 만들기 두 번이라 느리다. 따로 돌리고 stored는 돌려받는다
    let (stored, secrets) = tokio::task::spawn_blocking(move || {
        let secrets = if secret::verify(code.as_str(), &stored) {
            AdminSecrets::new(&pin).map(Some)
        } else {
            Ok(None)
        };
        (stored, secrets)
    })
    .await
    .map_err(ApiError::internal)?;
    let secrets = secrets
        .map_err(ApiError::internal)?
        .ok_or(ApiError::WrongRecoveryCode)?;

    let used = repository::use_code(
        &db,
        account.id,
        &stored,
        &secrets.recovery_code_hash,
        &secrets.pin_hash,
        OffsetDateTime::now_utc(),
    )
    .await
    .map_err(ApiError::internal)?;
    if !used {
        // 그사이 다른 화면에서 같은 코드를 먼저 썼다
        return Err(ApiError::WrongRecoveryCode);
    }
    tracing::info!(staff_id = account.id, "복구 코드로 관리자 PIN 다시 정함");

    Ok(Json(RecoveryResponse {
        recovery_code: secrets.recovery_code.to_string(),
    }))
}

/// 복구 코드 모양과 새 PIN(관리자 기준)을 함께 확인한다. 틀린 칸을 모두 모은다.
fn parse_input(code: &str, pin: &str) -> Result<(RecoveryCode, Pin), ApiError> {
    match (RecoveryCode::parse(code), Pin::parse(pin, Role::Admin)) {
        (Some(code), Ok(pin)) => Ok((code, pin)),
        (code, pin) => {
            let mut fields = Vec::new();
            if code.is_none() {
                fields.push(FieldError {
                    field: "code".to_owned(),
                    message: "복구 코드는 영문·숫자 16자입니다. 예: K7QF-M2XA-9TRE-HW4C".to_owned(),
                });
            }
            if let Err(error) = pin {
                fields.push(FieldError {
                    // Pin::parse는 PIN 칸 오류만 낸다
                    field: "pin".to_owned(),
                    message: error.to_string(),
                });
            }
            Err(ApiError::InvalidInput(fields))
        }
    }
}
