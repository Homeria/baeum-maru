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
use baeum_domain::{Action, Role, is_screen_locked, is_valid_on};

use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::error::ApiError;
use crate::clock::local_today;
use crate::db::staff::LoginAccount;
use crate::db::{devices, sessions};
use crate::secret::{DeviceToken, SessionToken};

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
        let found = devices::find_approved(&db, &token.hash())
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

/// 로그인한 요청만 다음으로 넘기고, 세션의 마지막 요청 시각을 새로 적는다.
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
    let session = sessions::find(&db, &token.hash())
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::LoginRequired)?;
    if !is_usable(&session.account) {
        return Err(ApiError::AccountUnavailable);
    }
    if is_screen_locked(session.last_seen_at, now) {
        return Err(ApiError::SessionLocked);
    }

    sessions::touch(&db, session.id, now)
        .await
        .map_err(ApiError::internal)?;
    request.extensions_mut().insert(CurrentStaff {
        id: session.account.id,
        role: session.account.role,
    });
    Ok(next.run(request).await)
}

/// BR-K5, BR-K6: 사용 중지하지 않았고 유효기간이 남은 계정인지.
/// 유효기간은 기관의 달력 날짜라 이 PC의 날짜로 본다.
pub fn is_usable(account: &LoginAccount) -> bool {
    !account.disabled && is_valid_on(account.valid_until, local_today())
}

/// `Authorization: Bearer <증표>` 머리글의 증표. 없거나 모양이 틀리면 `None`.
pub fn bearer_token(headers: &HeaderMap) -> Option<SessionToken> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    SessionToken::parse(value.strip_prefix("Bearer ")?)
}

#[cfg(test)]
mod tests {
    use axum::Router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use serde_json::{Value, json};
    use tower::ServiceExt;

    use super::*;
    use crate::server::test_support::{
        app, app_as, app_without_staff, delete, get, json_body, patch, post,
    };
    use crate::server::version::VERSION_HEADER;

    /// 로그인이 필요한 주소마다 하나씩. id는 없는 값이어도 된다 (로그인 확인이 먼저다).
    const SIGNED_IN_ROUTES: [(&str, &str); 18] = [
        ("GET", "/api/members"),
        ("POST", "/api/members"),
        ("GET", "/api/members/1"),
        ("PATCH", "/api/members/1"),
        ("DELETE", "/api/members/1"),
        ("GET", "/api/rooms"),
        ("POST", "/api/rooms"),
        ("DELETE", "/api/rooms/1"),
        ("GET", "/api/subjects"),
        ("PATCH", "/api/subjects/1"),
        ("GET", "/api/terms"),
        ("POST", "/api/terms"),
        ("GET", "/api/terms/1/courses"),
        ("POST", "/api/terms/1/courses"),
        ("GET", "/api/courses/1"),
        ("GET", "/api/staff"),
        ("POST", "/api/staff/1/pin"),
        ("GET", "/api/access-logs"),
    ];

    async fn send(router: Router, method: &str, uri: &str, token: Option<&str>) -> Value {
        let mut request = Request::builder()
            .method(method)
            .uri(uri)
            .header(VERSION_HEADER, crate::version::CURRENT)
            .header(header::CONTENT_TYPE, "application/json");
        if let Some(token) = token {
            request = request.header(header::AUTHORIZATION, token);
        }
        let response = router
            .oneshot(request.body(Body::from("{}")).unwrap())
            .await
            .unwrap();
        let status = response.status().as_u16();
        json!({ "status": status, "code": json_body(response).await["code"] })
    }

    #[tokio::test]
    async fn every_data_route_requires_login() {
        let (_dir, _db, router) = app_without_staff().await;
        let wrong_token = format!("Bearer {}", "0".repeat(64));

        for (method, uri) in SIGNED_IN_ROUTES {
            for token in [None, Some("Bearer 틀린모양"), Some(wrong_token.as_str())] {
                let result = send(router.clone(), method, uri, token).await;

                assert_eq!(
                    result,
                    json!({ "status": 401, "code": "LOGIN_REQUIRED" }),
                    "{method} {uri} {token:?}"
                );
            }
        }
    }

    async fn set_last_seen(db: &SqlitePool, minutes_ago: i64) {
        let last_seen = OffsetDateTime::now_utc() - time::Duration::minutes(minutes_ago);
        sqlx::query("UPDATE sessions SET last_seen_at = ?")
            .bind(crate::db::timestamp_text(last_seen))
            .execute(db)
            .await
            .unwrap();
    }

    async fn last_seen(db: &SqlitePool) -> String {
        sqlx::query_scalar("SELECT last_seen_at FROM sessions")
            .fetch_one(db)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn request_keeps_session_open() {
        let (_dir, db, router) = app().await;
        set_last_seen(&db, 9).await;
        let before = last_seen(&db).await;

        let response = get(router, "/api/rooms").await;

        assert_eq!(response.status(), StatusCode::OK);
        assert!(last_seen(&db).await > before);
    }

    #[tokio::test]
    async fn session_locks_after_ten_quiet_minutes() {
        let (_dir, db, router) = app().await;
        set_last_seen(&db, 10).await;
        let before = last_seen(&db).await;

        let response = get(router, "/api/rooms").await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let body = json_body(response).await;
        assert_eq!(body["code"], "SESSION_LOCKED");
        assert_eq!(body["rule"], "BR-K8");
        // 잠긴 요청은 시각을 늘리지 않는다
        assert_eq!(last_seen(&db).await, before);
    }

    #[tokio::test]
    async fn account_changes_apply_from_next_request() {
        let (_dir, db, router) = app().await;
        let room = json!({ "name": "201호", "building": null, "floor": null }).to_string();

        sqlx::query("UPDATE staff SET role = 'read_only'")
            .execute(&db)
            .await
            .unwrap();
        let response = post(router.clone(), "/api/rooms", &room).await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        assert_eq!(json_body(response).await["code"], "NOT_ALLOWED");

        sqlx::query("UPDATE staff SET disabled = 1")
            .execute(&db)
            .await
            .unwrap();
        let response = get(router, "/api/rooms").await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        assert_eq!(json_body(response).await["code"], "ACCOUNT_UNAVAILABLE");
    }

    #[tokio::test]
    async fn expired_account_is_blocked_from_next_request() {
        let (_dir, db, router) = app_as("temp_staff").await;
        sqlx::query("UPDATE staff SET valid_until = '2020-12-31'")
            .execute(&db)
            .await
            .unwrap();

        let response = get(router, "/api/members").await;

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        assert_eq!(json_body(response).await["code"], "ACCOUNT_UNAVAILABLE");
    }

    /// 없는 id에 빈 본문으로 바꾸기를 보내 본다. 역할 확인을 지나면 본문이나 대상에서
    /// 걸려 400·404가 되고, 역할이 허락하지 않으면 403이다.
    async fn allowed(router: Router, method: &str, uri: &str) -> bool {
        let body = "{}";
        let response = match method {
            "PATCH" => patch(router, uri, body).await,
            "DELETE" => delete(router, uri).await,
            _ => unreachable!("{method}"),
        };
        match response.status() {
            StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND => true,
            StatusCode::FORBIDDEN => false,
            other => panic!("{method} {uri}: {other}"),
        }
    }

    #[tokio::test]
    async fn roles_follow_the_role_table() {
        // (역할, 회원 고치기, 회원 지우기, 강좌 바꾸기, 기초 정보 바꾸기)
        let table = [
            ("admin", true, true, true, true),
            ("staff", true, true, true, true),
            ("temp_staff", true, false, false, false),
            ("read_only", false, false, false, false),
        ];
        for (role, edit_members, delete_members, edit_courses, edit_reference) in table {
            let (_dir, _db, router) = app_as(role).await;

            assert_eq!(
                allowed(router.clone(), "PATCH", "/api/members/999").await,
                edit_members,
                "{role}"
            );
            assert_eq!(
                allowed(router.clone(), "DELETE", "/api/members/999").await,
                delete_members,
                "{role}"
            );
            assert_eq!(
                allowed(router.clone(), "DELETE", "/api/courses/999").await,
                edit_courses,
                "{role}"
            );
            assert_eq!(
                allowed(router.clone(), "DELETE", "/api/terms/999").await,
                edit_courses,
                "{role}"
            );
            assert_eq!(
                allowed(router.clone(), "DELETE", "/api/subjects/999").await,
                edit_reference,
                "{role}"
            );
            assert_eq!(
                allowed(router.clone(), "DELETE", "/api/rooms/999").await,
                edit_reference,
                "{role}"
            );
            // 모두 보기는 모든 역할
            let response = get(router, "/api/members").await;
            assert_eq!(response.status(), StatusCode::OK, "{role}");
        }
    }
}
