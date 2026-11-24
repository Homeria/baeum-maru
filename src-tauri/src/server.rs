//! 이 PC와 다른 PC의 화면이 붙는 HTTP 서버.
//!
//! ADR 0008: 요청은 HTTP + JSON, 변경 알림은 SSE.

mod access_logs;
mod applications;
mod auth;
pub(crate) mod backups;
mod changes;
mod courses;
mod devices;
mod error;
mod login;
mod lotteries;
mod members;
mod paging;
mod priority_reasons;
mod recovery;
mod rooms;
mod rosters;
mod run_mode;
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

use self::changes::{ChangeKind, ChangeNotices, notifies};
use crate::backup::Backups;
use crate::version::Version;

/// ADR 0004: 포트 47800 고정.
pub const PORT: u16 = 47800;

/// 요청을 받는 주소. 같은 네트워크의 다른 PC도 붙도록 모든 주소에서 받는다(ADR 0001).
///
/// 같은 Wi-Fi의 누구든 요청할 수 있으므로, 승인한 기기가 아니면 승인 요청과 버전 확인 말고는
/// 받지 않는다(BR-K14, [`auth::require_device`]).
const LISTEN_ADDR: Ipv4Addr = Ipv4Addr::UNSPECIFIED;

/// 응답을 읽어도 되는 Tauri 창의 주소.
const APP_ORIGINS: [&str; 2] = [
    // Windows
    "http://tauri.localhost",
    // macOS, Linux
    "tauri://localhost",
];

/// 개발 모드(`just dev`)의 창 주소. 이 PC의 서버로는 Vite가 넘겨줘서 필요 없고, 다른 PC의
/// 호스트에 접속해 볼 때만 쓰인다. 출시 빌드에는 들어가지 않는다.
const DEV_ORIGIN: Option<&str> = if cfg!(debug_assertions) {
    Some("http://localhost:5173")
} else {
    None
};

/// 요청 처리 함수들이 함께 쓰는 것.
#[derive(Debug, Clone)]
pub struct AppState {
    /// 이 PC(호스트)의 버전.
    host: Version,
    db: SqlitePool,
    backups: Backups,
    changes: ChangeNotices,
}

impl AppState {
    pub fn new(host: Version, db: SqlitePool, backups: Backups) -> Self {
        AppState {
            host,
            db,
            backups,
            changes: ChangeNotices::new(),
        }
    }
}

impl FromRef<AppState> for ChangeNotices {
    fn from_ref(state: &AppState) -> Self {
        // 같은 방송 채널을 함께 가리키는 손잡이만 복사됨
        state.changes.clone()
    }
}

impl FromRef<AppState> for Backups {
    fn from_ref(state: &AppState) -> Self {
        state.backups.clone()
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
    let notices = state.changes.clone();
    // BR-K1: 로그인한 요청만 받는다. 데이터를 바꾸는 주소에는 변경 알림을 붙인다 (ADR 0008)
    let signed_in = Router::new()
        .route("/api/changes", get(changes::stream_changes))
        .route(
            "/api/members",
            notifies(
                get(members::search_members).post(members::create_member),
                &notices,
                ChangeKind::Member,
            ),
        )
        .route(
            "/api/members/{id}",
            notifies(
                get(members::get_member)
                    .patch(members::update_member)
                    .delete(members::delete_member),
                &notices,
                ChangeKind::Member,
            ),
        )
        .route(
            "/api/rooms",
            notifies(
                get(rooms::list_rooms).post(rooms::create_room),
                &notices,
                ChangeKind::Room,
            ),
        )
        .route(
            "/api/rooms/{id}",
            notifies(
                get(rooms::get_room)
                    .patch(rooms::update_room)
                    .delete(rooms::delete_room),
                &notices,
                ChangeKind::Room,
            ),
        )
        .route(
            "/api/priority-reasons",
            notifies(
                get(priority_reasons::list_priority_reasons)
                    .post(priority_reasons::create_priority_reason),
                &notices,
                ChangeKind::PriorityReason,
            ),
        )
        .route(
            "/api/priority-reasons/{id}",
            notifies(
                patch(priority_reasons::update_priority_reason)
                    .delete(priority_reasons::delete_priority_reason),
                &notices,
                ChangeKind::PriorityReason,
            ),
        )
        .route(
            "/api/subjects",
            notifies(
                get(subjects::list_subjects).post(subjects::create_subject),
                &notices,
                ChangeKind::Subject,
            ),
        )
        .route(
            "/api/subjects/{id}",
            notifies(
                get(subjects::get_subject)
                    .patch(subjects::update_subject)
                    .delete(subjects::delete_subject),
                &notices,
                ChangeKind::Subject,
            ),
        )
        .route(
            "/api/terms",
            notifies(
                get(terms::list_terms).post(terms::create_term),
                &notices,
                ChangeKind::Term,
            ),
        )
        .route(
            "/api/terms/{id}",
            notifies(
                get(terms::get_term)
                    .patch(terms::update_term)
                    .delete(terms::delete_term),
                &notices,
                ChangeKind::Term,
            ),
        )
        .route(
            "/api/terms/{term_id}/courses",
            notifies(
                get(courses::list_courses).post(courses::create_course),
                &notices,
                ChangeKind::Course,
            ),
        )
        .route(
            "/api/terms/{term_id}/applications",
            notifies(
                get(applications::list_applications).post(applications::submit_applications),
                &notices,
                ChangeKind::Application,
            ),
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
            notifies(
                get(lotteries::get_lottery).post(lotteries::run_lottery),
                &notices,
                ChangeKind::Selection,
            ),
        )
        .route(
            "/api/courses/{id}/selection-closure",
            notifies(
                post(lotteries::close_selection),
                &notices,
                ChangeKind::Selection,
            ),
        )
        .route(
            "/api/terms/{term_id}/roster-exports",
            post(rosters::export_roster),
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
            notifies(
                post(selection_results::withdraw),
                &notices,
                ChangeKind::Selection,
            ),
        )
        .route(
            "/api/applications/{id}/contacts",
            notifies(
                post(selection_results::record_contact),
                &notices,
                ChangeKind::Selection,
            ),
        )
        .route(
            "/api/applications/{id}/withdrawal-correction",
            notifies(
                post(selection_results::correct_withdrawal),
                &notices,
                ChangeKind::Selection,
            ),
        )
        .route(
            "/api/applications/{id}",
            notifies(
                delete(applications::delete_application),
                &notices,
                ChangeKind::Application,
            ),
        )
        .route(
            "/api/applications/{id}/priority-reasons",
            notifies(
                put(applications::set_priority_reasons),
                &notices,
                ChangeKind::Application,
            ),
        )
        .route(
            "/api/courses/{id}",
            notifies(
                get(courses::get_course)
                    .patch(courses::update_course)
                    .delete(courses::delete_course),
                &notices,
                ChangeKind::Course,
            ),
        )
        .route(
            "/api/staff",
            notifies(
                get(staff::list_staff).post(staff::create_staff),
                &notices,
                ChangeKind::Staff,
            ),
        )
        .route(
            "/api/staff/{id}",
            notifies(
                get(staff::get_staff).patch(staff::update_staff),
                &notices,
                ChangeKind::Staff,
            ),
        )
        .route(
            "/api/staff/{id}/pin",
            notifies(post(staff::reset_staff_pin), &notices, ChangeKind::Staff),
        )
        .route("/api/access-logs", get(access_logs::list_access_logs))
        .route(
            "/api/backups",
            notifies(
                get(backups::list_backups).post(backups::create_backup),
                &notices,
                ChangeKind::Backup,
            ),
        )
        .route(
            "/api/backups/second-location",
            notifies(
                put(backups::set_second_location),
                &notices,
                ChangeKind::Backup,
            ),
        )
        .route(
            "/api/backups/restore-preview",
            post(backups::preview_restore),
        )
        .route("/api/backups/restore", post(backups::restore))
        .route("/api/device-requests", get(devices::list_device_requests))
        .route(
            "/api/device-requests/{id}/approve",
            notifies(post(devices::approve_device), &notices, ChangeKind::Device),
        )
        .route(
            "/api/device-requests/{id}/reject",
            notifies(post(devices::reject_device), &notices, ChangeKind::Device),
        )
        .route("/api/devices", get(devices::list_devices))
        .route(
            "/api/devices/{id}",
            notifies(delete(devices::revoke_device), &notices, ChangeKind::Device),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_session,
        ));

    // BR-K14: 승인한 기기만 받는다. 기기 확인은 로그인 확인보다 먼저 한다
    // (route_layer는 나중에 붙인 것이 바깥)
    let approved = Router::new()
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
        // BR-K17: 로그인 전 화면에서 실행 모드를 바꿀 때 관리자 PIN을 확인한다
        .route("/api/run-mode/permit", post(run_mode::permit))
        .merge(signed_in)
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_device,
        ));

    // 버전 확인은 기기 확인보다 먼저 한다
    let checked = Router::new()
        // BR-K15: 승인받지 않은 기기가 요청하고 결과를 본다
        .route("/api/approval/request", post(devices::request_approval))
        .route("/api/approval/status", get(devices::approval_status))
        .merge(approved)
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
        .allow_origin(
            APP_ORIGINS
                .into_iter()
                .chain(DEV_ORIGIN)
                .map(HeaderValue::from_static)
                .collect::<Vec<_>>(),
        )
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([
            header::CONTENT_TYPE,
            // 세션 증표 (ADR 0014)
            header::AUTHORIZATION,
            HeaderName::from_static(version::VERSION_HEADER),
            // 기기 증표 (BR-K14)
            HeaderName::from_static(auth::DEVICE_HEADER),
            // 화면이 스스로 한 요청 (BR-K8)
            HeaderName::from_static(auth::QUIET_HEADER),
        ])
}

/// 서버를 띄워 끝날 때까지 돈다. 포트를 열지 못하면 로그만 남긴다.
pub async fn serve(host: Version, db: SqlitePool, backups: Backups) {
    if let Err(error) = listen(host, db, backups).await {
        tracing::error!(%error, "서버를 시작하지 못했습니다 (포트 {PORT})");
    }
}

async fn listen(host: Version, db: SqlitePool, backups: Backups) -> Result<(), std::io::Error> {
    let addr = SocketAddr::from((LISTEN_ADDR, PORT));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, version = crate::version::CURRENT, "서버 시작");
    // 처리 함수가 요청을 보낸 PC의 주소를 알게 한다 (BR-L6, auth::Client)
    let app = router(AppState::new(host, db, backups))
        .into_make_service_with_connect_info::<SocketAddr>();
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::extract::connect_info::MockConnectInfo;
    use axum::http::{Request, Response, StatusCode};
    use tower::ServiceExt;

    use super::*;

    #[test]
    fn listens_for_other_pcs() {
        assert!(LISTEN_ADDR.is_unspecified());
    }

    async fn send(request: Request<Body>) -> Response<Body> {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path()).await.unwrap();
        let backups = Backups::new(db.clone(), dir.path());
        // 호스트 PC 자신의 창에서 보낸 것으로 본다
        router(AppState::new("0.1.0".parse().unwrap(), db, backups))
            .layer(MockConnectInfo(SocketAddr::from((
                Ipv4Addr::LOCALHOST,
                50000,
            ))))
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
