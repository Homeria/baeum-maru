//! 요청 처리 테스트에서 함께 쓰는 도우미. 테스트할 때만 컴파일된다.
//!
//! 요청은 모두 맞는 버전 머리글을 붙여 보낸다. 버전 확인 자체를 시험할 때는
//! `Request`를 직접 만든다.

use axum::Router;
use axum::body::Body;
use axum::http::request::Builder;
use axum::http::{Request, Response, header};
use serde_json::Value;
use sqlx::SqlitePool;
use tower::ServiceExt;

use super::version::VERSION_HEADER;
use super::{AppState, router};

/// 임시 DB로 라우터를 만든다. `TempDir`가 살아 있는 동안 DB가 남는다.
pub async fn app() -> (tempfile::TempDir, SqlitePool, Router) {
    let (dir, db) = crate::db::open_temp().await;
    let router = router(AppState::new(
        crate::version::CURRENT.parse().unwrap(),
        db.clone(),
    ));
    (dir, db, router)
}

pub async fn get(router: Router, uri: &str) -> Response<Body> {
    send(router, Request::get(uri), Body::empty()).await
}

pub async fn post(router: Router, uri: &str, body: &str) -> Response<Body> {
    send(
        router,
        json(Request::post(uri)),
        Body::from(body.to_owned()),
    )
    .await
}

pub async fn patch(router: Router, uri: &str, body: &str) -> Response<Body> {
    send(
        router,
        json(Request::patch(uri)),
        Body::from(body.to_owned()),
    )
    .await
}

pub async fn delete(router: Router, uri: &str) -> Response<Body> {
    send(router, Request::delete(uri), Body::empty()).await
}

/// 응답 본문을 JSON으로 읽는다.
pub async fn json_body(response: Response<Body>) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn json(request: Builder) -> Builder {
    request.header(header::CONTENT_TYPE, "application/json")
}

async fn send(router: Router, request: Builder, body: Body) -> Response<Body> {
    let request = request
        .header(VERSION_HEADER, crate::version::CURRENT)
        .body(body)
        .unwrap();
    router.oneshot(request).await.unwrap()
}
