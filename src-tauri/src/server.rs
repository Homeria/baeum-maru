//! 이 PC와 다른 PC의 화면이 붙는 HTTP 서버.
//!
//! ADR 0008: 요청은 HTTP + JSON.

mod access_logs;
mod applications;
mod auth;
mod courses;
mod error;
mod login;
mod lotteries;
mod members;
mod paging;
mod priority_reasons;
mod recovery;
mod rooms;
mod selection_results;
mod setup;
mod staff;
mod subjects;
mod terms;
#[cfg(test)]
mod test_support;
mod version;

use std::net::{Ipv4Addr, SocketAddr};

use axum::extract::FromRef;
use axum::http::{HeaderName, HeaderValue, Method, header};
use axum::routing::{delete, get, patch, post, put};
use axum::{Router, middleware};
use sqlx::SqlitePool;
use tower_http::cors::CorsLayer;
use tower_http::set_header::SetResponseHeaderLayer;

use crate::data_dir::DataDir;
use crate::db::{self, DbError};
use crate::version::Version;

/// ADR 0004: 포트 47800 고정.
pub const PORT: u16 = 47800;

/// 요청을 받는 주소. 지금은 이 PC 안(127.0.0.1)에서만 받는다.
///
/// ADR 0001은 다른 PC도 접속하게 하지만, 아직 다른 PC의 앱이 붙는 기능(ADR 0004)과
/// 로그인·기기 승인(ADR 0003)이 없다. 그 전에 모든 주소에서 받으면 같은 네트워크의
/// 누구든 요청할 수 있어서, 기기 승인을 만들 때 모든 주소(`Ipv4Addr::UNSPECIFIED`)로 연다.
const LISTEN_ADDR: Ipv4Addr = Ipv4Addr::LOCALHOST;

/// 응답을 읽어도 되는 Tauri 창의 주소.
const APP_ORIGINS: [&str; 2] = [
    // Windows
    "http://tauri.localhost",
    // macOS, Linux
    "tauri://localhost",
];

/// 요청 처리 함수들이 함께 쓰는 것.
#[derive(Debug, Clone)]
pub struct AppState {
    /// 이 PC(호스트)의 버전.
    host: Version,
    db: SqlitePool,
}

impl AppState {
    pub fn new(host: Version, db: SqlitePool) -> Self {
        AppState { host, db }
    }
}

// 처리 함수가 State<Version>, State<SqlitePool>로 필요한 것만 꺼내게 함
impl FromRef<AppState> for Version {
    fn from_ref(state: &AppState) -> Self {
        state.host
    }
}

impl FromRef<AppState> for SqlitePool {
    fn from_ref(state: &AppState) -> Self {
        // 연결 묶음을 함께 가리키는 손잡이만 복사됨
        state.db.clone()
    }
}

/// 모든 요청 주소를 묶은 라우터.
pub fn router(state: AppState) -> Router {
    // BR-K1: 로그인한 요청만 받는다
    let signed_in = Router::new()
        .route(
            "/api/members",
            get(members::search_members).post(members::create_member),
        )
        .route(
            "/api/members/{id}",
            get(members::get_member)
                .patch(members::update_member)
                .delete(members::delete_member),
        )
        .route(
            "/api/rooms",
            get(rooms::list_rooms).post(rooms::create_room),
        )
        .route(
            "/api/rooms/{id}",
            get(rooms::get_room)
                .patch(rooms::update_room)
                .delete(rooms::delete_room),
        )
        .route(
            "/api/priority-reasons",
            get(priority_reasons::list_priority_reasons)
                .post(priority_reasons::create_priority_reason),
        )
        .route(
            "/api/priority-reasons/{id}",
            patch(priority_reasons::update_priority_reason)
                .delete(priority_reasons::delete_priority_reason),
        )
        .route(
            "/api/subjects",
            get(subjects::list_subjects).post(subjects::create_subject),
        )
        .route(
            "/api/subjects/{id}",
            get(subjects::get_subject)
                .patch(subjects::update_subject)
                .delete(subjects::delete_subject),
        )
        .route(
            "/api/terms",
            get(terms::list_terms).post(terms::create_term),
        )
        .route(
            "/api/terms/{id}",
            get(terms::get_term)
                .patch(terms::update_term)
                .delete(terms::delete_term),
        )
        .route(
            "/api/terms/{term_id}/courses",
            get(courses::list_courses).post(courses::create_course),
        )
        .route(
            "/api/terms/{term_id}/applications",
            get(applications::list_applications).post(applications::submit_applications),
        )
        .route(
            "/api/terms/{term_id}/application-counts",
            get(applications::list_application_counts),
        )
        .route(
            "/api/terms/{term_id}/selections",
            get(lotteries::list_selections),
        )
        .route(
            "/api/courses/{id}/lottery",
            get(lotteries::get_lottery).post(lotteries::run_lottery),
        )
        .route(
            "/api/courses/{id}/selection-closure",
            post(lotteries::close_selection),
        )
        .route(
            "/api/terms/{term_id}/selection-results",
            get(selection_results::list_closed_courses),
        )
        .route(
            "/api/courses/{id}/selection-result",
            get(selection_results::get_course_result),
        )
        .route(
            "/api/applications/{id}/withdrawal",
            post(selection_results::withdraw),
        )
        .route(
            "/api/applications/{id}/contacts",
            post(selection_results::record_contact),
        )
        .route(
            "/api/applications/{id}/withdrawal-correction",
            post(selection_results::correct_withdrawal),
        )
        .route(
            "/api/applications/{id}",
            delete(applications::delete_application),
        )
        .route(
            "/api/applications/{id}/priority-reasons",
            put(applications::set_priority_reasons),
        )
        .route(
            "/api/courses/{id}",
            get(courses::get_course)
                .patch(courses::update_course)
                .delete(courses::delete_course),
        )
        .route(
            "/api/staff",
            get(staff::list_staff).post(staff::create_staff),
        )
        .route(
            "/api/staff/{id}",
            get(staff::get_staff).patch(staff::update_staff),
        )
        .route("/api/staff/{id}/pin", post(staff::reset_staff_pin))
        .route("/api/access-logs", get(access_logs::list_access_logs))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_session,
        ));

    // 버전 확인은 로그인 확인보다 먼저 한다 (route_layer는 나중에 붙인 것이 바깥)
    let checked = Router::new()
        // 처음 설정은 직원이 없을 때 하므로 로그인 없이 받는다
        .route(
            "/api/setup",
            get(setup::get_setup_status).post(setup::setup),
        )
        // 로그인 화면과 잠긴 화면은 로그인 확인을 거치지 않고 스스로 세션을 본다
        .route("/api/login/staff", get(login::list_login_staff))
        .route("/api/login", post(login::login))
        .route("/api/unlock", post(login::unlock))
        .route("/api/logout", post(login::logout))
        // PIN을 잊은 관리자는 로그인하지 못하므로 그대로 받는다
        .route("/api/recovery", post(recovery::recover))
        .merge(signed_in)
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            version::require_compatible,
        ));

    Router::new()
        .route("/api/version", get(version::get_version))
        .merge(checked)
        .with_state(state)
        .layer(no_store())
        .layer(cors())
}

/// 응답에 회원 정보가 담기므로 창(WebView)이 디스크에 캐시하지 않게 한다.
fn no_store() -> SetResponseHeaderLayer<HeaderValue> {
    SetResponseHeaderLayer::overriding(header::CACHE_CONTROL, HeaderValue::from_static("no-store"))
}

/// 창의 화면은 주소가 달라서(CORS) 서버가 허락해야 응답을 읽는다.
fn cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(APP_ORIGINS.map(HeaderValue::from_static))
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([
            header::CONTENT_TYPE,
            // 세션 증표 (ADR 0014)
            header::AUTHORIZATION,
            HeaderName::from_static(version::VERSION_HEADER),
        ])
}

/// 서버를 시작하지 못했다.
#[derive(Debug, thiserror::Error)]
enum ServeError {
    #[error(transparent)]
    Db(#[from] DbError),
    #[error("포트 {PORT}을 열지 못했습니다: {0}")]
    Listen(#[from] std::io::Error),
}

/// DB를 열고 서버를 띄워 끝날 때까지 돈다. 실패하면 로그만 남긴다.
pub async fn serve(host: Version, data_dir: DataDir) {
    if let Err(error) = listen(host, &data_dir).await {
        tracing::error!(%error, "서버를 시작하지 못했습니다");
    }
}

async fn listen(host: Version, data_dir: &DataDir) -> Result<(), ServeError> {
    let db = db::open(data_dir.path()).await?;
    tracing::info!(portable = data_dir.is_portable(), "DB 열림");

    let addr = SocketAddr::from((LISTEN_ADDR, PORT));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, version = crate::version::CURRENT, "서버 시작");
    // 처리 함수가 요청을 보낸 PC의 주소를 알게 한다 (BR-L6, auth::Client)
    let app = router(AppState::new(host, db)).into_make_service_with_connect_info::<SocketAddr>();
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, Response, StatusCode};
    use tower::ServiceExt;

    use super::*;

    #[test]
    fn listens_only_inside_this_pc_until_device_approval() {
        assert!(LISTEN_ADDR.is_loopback());
    }

    async fn send(request: Request<Body>) -> Response<Body> {
        let dir = tempfile::tempdir().unwrap();
        let db = db::open(dir.path()).await.unwrap();
        router(AppState::new("0.1.0".parse().unwrap(), db))
            .oneshot(request)
            .await
            .unwrap()
    }

    async fn allowed_origin(origin: &str) -> Option<HeaderValue> {
        let request = Request::get("/api/version")
            .header(header::ORIGIN, origin)
            .body(Body::empty())
            .unwrap();
        let response = send(request).await;
        response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .cloned()
    }

    #[tokio::test]
    async fn cors_allows_app_window() {
        for origin in APP_ORIGINS {
            assert_eq!(allowed_origin(origin).await.unwrap(), origin);
        }
    }

    #[tokio::test]
    async fn cors_rejects_other_pages() {
        assert_eq!(allowed_origin("http://example.com").await, None);
    }

    #[tokio::test]
    async fn cors_allows_json_writes_from_app_window() {
        for method in ["POST", "PATCH", "DELETE"] {
            let request = Request::options("/api/members/1")
                .header(header::ORIGIN, APP_ORIGINS[0])
                .header(header::ACCESS_CONTROL_REQUEST_METHOD, method)
                .header(
                    header::ACCESS_CONTROL_REQUEST_HEADERS,
                    "content-type, authorization",
                )
                .body(Body::empty())
                .unwrap();

            let response = send(request).await;

            let headers = response.headers();
            let methods = headers[header::ACCESS_CONTROL_ALLOW_METHODS]
                .to_str()
                .unwrap();
            assert!(methods.contains(method), "{methods}");
            let allowed = headers[header::ACCESS_CONTROL_ALLOW_HEADERS]
                .to_str()
                .unwrap();
            assert!(allowed.contains("content-type"), "{allowed}");
            assert!(allowed.contains("authorization"), "{allowed}");
        }
    }

    async fn post_member(version: Option<&str>) -> Response<Body> {
        let mut request =
            Request::post("/api/members").header(header::CONTENT_TYPE, "application/json");
        if let Some(version) = version {
            request = request.header(version::VERSION_HEADER, version);
        }
        let body = r#"{"memberNo":"00001","name":"회원1","phoneNumber":"01000000001"}"#;
        send(request.body(Body::from(body)).unwrap()).await
    }

    #[tokio::test]
    async fn rejects_request_without_version() {
        let response = post_member(None).await;

        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn rejects_request_from_other_minor_version() {
        let response = post_member(Some("0.2.0")).await;

        assert_eq!(response.status(), StatusCode::CONFLICT);
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["code"], "VERSION_MISMATCH");
    }

    #[tokio::test]
    async fn accepts_request_when_only_patch_differs() {
        let response = post_member(Some("0.1.9")).await;

        // 버전 확인을 지나 로그인 확인에서 멈춘다 (로그인하지 않은 요청)
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["code"], "LOGIN_REQUIRED");
    }

    #[tokio::test]
    async fn responses_are_not_cached() {
        let request = Request::get("/api/version").body(Body::empty()).unwrap();

        let response = send(request).await;

        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    }
}
