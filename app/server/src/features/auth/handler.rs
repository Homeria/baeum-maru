//! 로그인, 로그아웃, 화면 잠금 풀기, 복구 코드로 PIN 다시 정하기 (BR-K1, BR-K5~K8, BR-K12, BR-K13).

use std::convert::Infallible;
use std::time::Duration;

use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRef, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::{get, post};
use axum::{Json, Router};
use baeum_domain::{Pin, PinRefusal, RecoveryCode, Role};
use sqlx::SqlitePool;
use time::OffsetDateTime;
use tokio::sync::{broadcast, mpsc};
use tokio_stream::wrappers::ReceiverStream;

use super::dto::{
    LoginRequest, LoginResponse, LoginStaffListResponse, LoginStaffResponse, RecoveryRequest,
    RecoveryResponse, SignedInStaffResponse, UnlockRequest,
};
use super::middleware::{CurrentDevice, bearer_token, device_token};
use super::repository;
use super::service::check_pin;
use crate::features::access_logs::repository::{self as access_logs, AccessEvent};
use crate::http::changes::{BACKLOG, ChangeKind, ChangeNotices};
use crate::http::error::{ApiError, FieldError};
use crate::infra::clock::local_today;
use crate::infra::secret::{self, AdminSecrets, SessionToken};

/// 로그인 전에 받는 주소. 기기 확인 안, 로그인 확인 바깥에 들어간다.
///
/// 로그인 화면과 잠긴 화면은 로그인 확인을 거치지 않고 스스로 세션을 본다.
/// PIN을 잊은 관리자는 로그인하지 못하므로 복구도 여기서 받는다.
pub(crate) fn login_routes<S>() -> Router<S>
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

/// 로그인한 요청만 받는 주소: 변경 알림 연결 (ADR 0008).
pub(crate) fn routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    SqlitePool: FromRef<S>,
    ChangeNotices: FromRef<S>,
{
    Router::new().route("/api/changes", get(stream_changes))
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

/// 세션과 기기 승인이 아직 살아 있는지 다시 보는 간격.
const RECHECK_EVERY: Duration = Duration::from_secs(30);

/// `GET /api/changes`: 변경 알림을 SSE로 흘려보낸다.
///
/// 로그인 확인을 거친 뒤에만 열린다. 열린 뒤에도 세션이 잠기거나 끝났거나, 기기 승인이
/// 취소되면 끊는다. 이 확인은 세션의 마지막 요청 시각을 새로 적지 않는다(BR-K8).
///
/// # Errors
///
/// 증표가 머리글에 없으면(로그인 확인을 붙이지 않은 경우) [`ApiError::LoginRequired`] 또는
/// [`ApiError::DeviceNotApproved`].
async fn stream_changes(
    State(notices): State<ChangeNotices>,
    State(db): State<SqlitePool>,
    device: CurrentDevice,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let session = bearer_token(&headers)
        .ok_or(ApiError::LoginRequired)?
        .hash();
    let device = match device.id {
        // 호스트 PC 자신의 창은 기기 승인이 없다
        None => None,
        Some(_) => Some(
            device_token(&headers)
                .ok_or(ApiError::DeviceNotApproved)?
                .hash(),
        ),
    };
    let watch = Watch { session, device };
    let (events, stream) = mpsc::channel(BACKLOG);
    tokio::spawn(forward(db, watch, notices.subscribe(), events));
    Ok(Sse::new(ReceiverStream::new(stream)).keep_alive(KeepAlive::default()))
}

/// 열린 연결이 아직 쓸 수 있는지 볼 증표들의 해시.
pub(super) struct Watch {
    pub(super) session: [u8; 32],
    /// 다른 PC면 그 기기의 증표. 호스트 PC 자신이면 `None`.
    pub(super) device: Option<[u8; 32]>,
}

impl Watch {
    pub(super) async fn still_open(&self, db: &SqlitePool, now: OffsetDateTime) -> bool {
        match self.check(db, now).await {
            Ok(open) => open,
            Err(error) => {
                tracing::error!(%error, "변경 알림 연결을 확인하지 못했습니다");
                false
            }
        }
    }

    async fn check(&self, db: &SqlitePool, now: OffsetDateTime) -> Result<bool, sqlx::Error> {
        let Some(session) = repository::find(db, &self.session).await? else {
            return Ok(false);
        };
        if !session.account.is_usable() || baeum_domain::is_screen_locked(session.last_seen_at, now)
        {
            return Ok(false);
        }
        match &self.device {
            None => Ok(true),
            Some(device) => Ok(repository::find_approved(db, device).await?.is_some()),
        }
    }
}

/// 방송 채널의 알림을 이 연결로 옮긴다. 화면이 연결을 닫거나, 확인에 실패하거나,
/// 알림이 너무 밀리면 끝난다. 끝나면 연결이 닫히고, 화면은 다시 연결하며 전부 다시 받는다.
async fn forward(
    db: SqlitePool,
    watch: Watch,
    mut changes: broadcast::Receiver<ChangeKind>,
    events: mpsc::Sender<Result<Event, Infallible>>,
) {
    let mut recheck = tokio::time::interval(RECHECK_EVERY);
    // 첫 틱은 바로 오고, 방금 로그인 확인을 거쳤으니 건너뛴다
    recheck.tick().await;
    loop {
        tokio::select! {
            () = events.closed() => return,
            _ = recheck.tick() => {
                if !watch.still_open(&db, OffsetDateTime::now_utc()).await {
                    return;
                }
            }
            change = changes.recv() => {
                // 뒤처졌거나(Lagged) 서버가 닫혔다. 끊으면 화면이 전부 다시 받는다 (ADR 0008)
                let Ok(kind) = change else { return };
                if events.send(Ok(event(kind))).await.is_err() {
                    return;
                }
            }
        }
    }
}

fn event(kind: ChangeKind) -> Event {
    // 알림 이름만 담는다. 예: event: change, data: "application"
    Event::default()
        .event("change")
        .json_data(kind)
        .unwrap_or_else(|error| {
            tracing::error!(%error, "변경 알림을 만들지 못했습니다");
            Event::default().event("change")
        })
}
