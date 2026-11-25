//! 기기 확인, 로그인 확인, 역할 확인 (BR-K1, BR-K3, BR-K5, BR-K6, BR-K8, BR-K14).
//!
//! ADR 0014: 요청마다 세션과 직원 계정을 함께 읽어서, 역할 변경·사용 중지·유효기간 만료가
//! 다음 요청부터 바로 적용된다.

use std::convert::Infallible;
use std::net::SocketAddr;

use axum::extract::{ConnectInfo, FromRequestParts, Request, State};
use axum::http::request::Parts;
use axum::http::{HeaderMap, header};
use axum::middleware::Next;
use axum::response::Response;
use baeum_domain::{Action, Role, is_screen_locked};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::repository;
use crate::http::error::ApiError;
use crate::infra::secret::{DeviceToken, SessionToken};

/// 지금 요청한 직원. 로그인 확인을 통과한 요청에서만 꺼낼 수 있다.
///
/// 처리 함수의 인자로 받으면 axum이 요청에서 꺼내 준다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrentStaff {
    pub id: i64,
    pub role: Role,
}

impl CurrentStaff {
    /// BR-K3: 이 직원의 역할로 `action`을 할 수 있어야 한다.
    ///
    /// # Errors
    ///
    /// 역할에 없는 일이면 [`ApiError::NotAllowed`].
    pub fn require(self, action: Action) -> Result<(), ApiError> {
        if self.role.can(action) {
            Ok(())
        } else {
            Err(ApiError::NotAllowed)
        }
    }
}

impl<S: Send + Sync> FromRequestParts<S> for CurrentStaff {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // require_session이 넣어 둔다. 없으면 로그인 확인을 붙이지 않은 주소에서 쓴 것이다
        parts
            .extensions
            .get::<CurrentStaff>()
            .copied()
            .ok_or(ApiError::LoginRequired)
    }
}

/// 요청을 보낸 PC가 호스트 자신인지. 추첨처럼 호스트 PC에서만 하는 일(BR-L6)에 쓴다.
///
/// 처리 함수의 인자로 받으면 axum이 요청의 연결 주소로 정해 준다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Client {
    pub on_host: bool,
}

impl Client {
    /// BR-L6: 호스트 PC에서 온 요청이어야 한다.
    ///
    /// # Errors
    ///
    /// 다른 PC에서 왔으면 [`ApiError::HostOnly`].
    pub fn require_host(self) -> Result<(), ApiError> {
        if self.on_host {
            Ok(())
        } else {
            Err(ApiError::HostOnly)
        }
    }
}

impl<S: Send + Sync> FromRequestParts<S> for Client {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // 서버가 연결마다 상대 주소를 넣어 둔다(serve의 into_make_service_with_connect_info).
        // 주소를 모르면 호스트가 아니라고 본다
        let on_host = ConnectInfo::<SocketAddr>::from_request_parts(parts, state)
            .await
            .is_ok_and(|ConnectInfo(addr)| addr.ip().is_loopback());
        Ok(Client { on_host })
    }
}

/// 기기 증표를 싣는 머리글 (BR-K14).
pub const DEVICE_HEADER: &str = "x-baeum-device";

/// 요청을 보낸 기기. 기기 확인을 통과한 요청에서만 꺼낼 수 있다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrentDevice {
    /// 승인한 기기의 id. 호스트 PC 자신이면 `None`.
    pub id: Option<i64>,
}

impl<S: Send + Sync> FromRequestParts<S> for CurrentDevice {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // require_device가 넣어 둔다. 없으면 기기 확인을 붙이지 않은 주소에서 쓴 것이다
        parts
            .extensions
            .get::<CurrentDevice>()
            .copied()
            .ok_or(ApiError::DeviceNotApproved)
    }
}

/// BR-K14: 승인한 기기의 요청만 다음으로 넘긴다. 호스트 PC 자신의 창은 승인 없이 쓴다.
///
/// # Errors
///
/// 다른 PC에서 왔는데 기기 증표가 없거나, 승인하지 않았거나 승인을 취소한 증표면
/// [`ApiError::DeviceNotApproved`].
pub async fn require_device(
    State(db): State<SqlitePool>,
    client: Client,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let id = if client.on_host {
        None
    } else {
        let token = device_token(request.headers()).ok_or(ApiError::DeviceNotApproved)?;
        let found = repository::find_approved(&db, &token.hash())
            .await
            .map_err(ApiError::internal)?;
        Some(found.ok_or(ApiError::DeviceNotApproved)?)
    };
    request.extensions_mut().insert(CurrentDevice { id });
    Ok(next.run(request).await)
}

/// 기기 증표 머리글의 증표. 없거나 모양이 틀리면 `None`.
pub fn device_token(headers: &HeaderMap) -> Option<DeviceToken> {
    DeviceToken::parse(headers.get(DEVICE_HEADER)?.to_str().ok()?)
}

/// 화면이 스스로 한 요청에 붙이는 머리글. 직원이 화면을 만지지 않는 동안의 요청이다(BR-K8).
pub const QUIET_HEADER: &str = "x-baeum-quiet";

/// 로그인한 요청만 다음으로 넘기고, 세션의 마지막 요청 시각을 새로 적는다.
///
/// [`QUIET_HEADER`]가 붙은 요청은 시각을 새로 적지 않는다. 변경 알림 때문에 다시 받는 요청이
/// 자동 잠금(BR-K8)을 미루지 않게.
///
/// # Errors
///
/// - 증표가 없거나 맞는 세션이 없으면 [`ApiError::LoginRequired`]
/// - 계정을 사용 중지했거나 유효기간이 지났으면 [`ApiError::AccountUnavailable`]
/// - 마지막 요청에서 10분이 지났으면 [`ApiError::SessionLocked`]
pub async fn require_session(
    State(db): State<SqlitePool>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let token = bearer_token(request.headers()).ok_or(ApiError::LoginRequired)?;
    let now = OffsetDateTime::now_utc();
    let session = repository::find(&db, &token.hash())
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::LoginRequired)?;
    if !session.account.is_usable() {
        return Err(ApiError::AccountUnavailable);
    }
    if is_screen_locked(session.last_seen_at, now) {
        return Err(ApiError::SessionLocked);
    }

    if !request.headers().contains_key(QUIET_HEADER) {
        repository::touch(&db, session.id, now)
            .await
            .map_err(ApiError::internal)?;
    }
    request.extensions_mut().insert(CurrentStaff {
        id: session.account.id,
        role: session.account.role,
    });
    Ok(next.run(request).await)
}

/// `Authorization: Bearer <증표>` 머리글의 증표. 없거나 모양이 틀리면 `None`.
pub fn bearer_token(headers: &HeaderMap) -> Option<SessionToken> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    SessionToken::parse(value.strip_prefix("Bearer ")?)
}
