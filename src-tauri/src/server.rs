//! 이 PC와 다른 PC의 화면이 붙는 HTTP 서버.
//!
//! ADR 0008: 요청은 HTTP + JSON.

mod version;

use axum::Router;
use axum::routing::get;

use crate::version::Version;

/// ADR 0004: 포트 47800 고정.
pub const PORT: u16 = 47800;

/// 모든 요청 주소를 묶은 라우터. `host`는 이 PC(호스트)의 버전이다.
pub fn router(host: Version) -> Router {
    Router::new()
        .route("/api/version", get(version::get_version))
        .with_state(host)
}
