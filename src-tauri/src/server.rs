//! 이 PC와 다른 PC의 화면이 붙는 HTTP 서버.
//!
//! ADR 0008: 요청은 HTTP + JSON.

mod version;

use std::net::{Ipv4Addr, SocketAddr};

use axum::Router;
use axum::http::{HeaderName, HeaderValue, Method};
use axum::routing::get;
use tower_http::cors::CorsLayer;

use crate::data_dir::DataDir;
use crate::db::{self, DbError};
use crate::version::Version;

/// ADR 0004: 포트 47800 고정.
pub const PORT: u16 = 47800;

/// 응답을 읽어도 되는 Tauri 창의 주소.
const APP_ORIGINS: [&str; 2] = [
    // Windows
    "http://tauri.localhost",
    // macOS, Linux
    "tauri://localhost",
];

/// 모든 요청 주소를 묶은 라우터. `host`는 이 PC(호스트)의 버전이다.
pub fn router(host: Version) -> Router {
    Router::new()
        .route("/api/version", get(version::get_version))
        .with_state(host)
        .layer(cors())
}

/// 창의 화면은 주소가 달라서(CORS) 서버가 허락해야 응답을 읽는다.
fn cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(APP_ORIGINS.map(HeaderValue::from_static))
        .allow_methods([Method::GET])
        .allow_headers([HeaderName::from_static(version::VERSION_HEADER)])
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
    // 서버가 도는 동안 DB 연결을 열어 둠
    let _db = db::open(data_dir.path()).await?;
    tracing::info!(portable = data_dir.is_portable(), "DB 열림");

    // [ADR 0001]: 다른 PC도 접속하므로 모든 주소에서 받음
    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, PORT));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, version = crate::version::CURRENT, "서버 시작");
    axum::serve(listener, router(host)).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, header};
    use tower::ServiceExt;

    use super::*;

    async fn allowed_origin(origin: &str) -> Option<HeaderValue> {
        let request = Request::get("/api/version")
            .header(header::ORIGIN, origin)
            .body(Body::empty())
            .unwrap();
        let response = router("0.1.0".parse().unwrap())
            .oneshot(request)
            .await
            .unwrap();
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
}
