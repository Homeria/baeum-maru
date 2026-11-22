//! 요청 처리 테스트에서 함께 쓰는 도우미. 테스트할 때만 컴파일된다.
//!
//! 요청은 모두 맞는 버전 머리글을 붙여 보낸다. 버전 확인 자체를 시험할 때는
//! `Request`를 직접 만든다.
//!
//! [`app`]과 [`app_as`]의 라우터는 로그인한 직원의 세션 증표를 요청마다 붙인다.
//! 요청에 `Authorization` 머리글이 이미 있으면 그것을 쓴다.

use std::net::{Ipv4Addr, SocketAddr};

use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::extract::connect_info::MockConnectInfo;
use axum::http::request::Builder;
use axum::http::{HeaderValue, Request, Response, StatusCode, header};
use axum::{Extension, Router};

use serde_json::{Value, json};
use sqlx::SqlitePool;
use time::OffsetDateTime;
use tower::ServiceExt;
use tower_http::set_header::SetRequestHeaderLayer;

use super::version::VERSION_HEADER;
use super::{AppState, router};
use crate::db::sessions;
use crate::db::test_support::insert_staff;
use crate::secret::SessionToken;

/// 임시 DB로 라우터를 만들고 관리자로 로그인해 둔다. `TempDir`가 살아 있는 동안 DB가 남는다.
pub async fn app() -> (tempfile::TempDir, SqlitePool, Router) {
    app_as("admin").await
}

/// [`app`]과 같지만 `role`(DB의 역할 값) 직원으로 로그인해 둔다.
///
/// 직원과 세션을 DB에 바로 넣어서 PIN 해시를 만들지 않는다. 접속 기록에 로그인이 한 줄 남는다.
pub async fn app_as(role: &str) -> (tempfile::TempDir, SqlitePool, Router) {
    let (dir, db, router) = app_without_staff().await;
    let valid_until = (role == "temp_staff").then_some("2999-12-31");
    let staff_id = insert_staff(&db, "시험직원", role, valid_until)
        .await
        .unwrap();
    let token = SessionToken::new().unwrap();
    sessions::start(&db, staff_id, &token.hash(), OffsetDateTime::now_utc())
        .await
        .unwrap();
    let bearer = HeaderValue::from_str(&format!("Bearer {}", token.to_hex())).unwrap();
    let router = router.layer(SetRequestHeaderLayer::if_not_present(
        header::AUTHORIZATION,
        bearer,
    ));
    (dir, db, router)
}

/// 직원이 하나도 없는 새 DB로 라우터를 만든다. 로그인하지 않은 상태다.
///
/// 요청은 호스트 PC 자신(127.0.0.1)에서 온 것으로 본다. 다른 PC는 [`from_other_pc`].
pub async fn app_without_staff() -> (tempfile::TempDir, SqlitePool, Router) {
    let (dir, db) = crate::db::open_temp().await;
    let router = router(AppState::new(
        crate::version::CURRENT.parse().unwrap(),
        db.clone(),
    ))
    .layer(MockConnectInfo(SocketAddr::from((
        Ipv4Addr::LOCALHOST,
        50000,
    ))));
    (dir, db, router)
}

/// 요청이 같은 네트워크의 다른 PC에서 온 것으로 바꾼다.
///
/// 진짜 연결 주소(`ConnectInfo`)를 넣으면 [`MockConnectInfo`]보다 먼저 쓰인다.
pub fn from_other_pc(router: Router) -> Router {
    let other_pc = SocketAddr::from((Ipv4Addr::new(192, 168, 0, 20), 50000));
    router.layer(Extension(ConnectInfo(other_pc)))
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

pub async fn put(router: Router, uri: &str, body: &str) -> Response<Body> {
    send(router, json(Request::put(uri)), Body::from(body.to_owned())).await
}

pub async fn delete(router: Router, uri: &str) -> Response<Body> {
    send(router, Request::delete(uri), Body::empty()).await
}

/// 본문을 붙인 DELETE. 예: 지우는 사유
pub async fn delete_with_body(router: Router, uri: &str, body: &str) -> Response<Body> {
    send(
        router,
        json(Request::delete(uri)),
        Body::from(body.to_owned()),
    )
    .await
}

/// `uri`에 `body`를 보내 새로 만들고 새 id를 돌려준다. 201이 아니면 테스트가 실패한다.
pub async fn create(router: Router, uri: &str, body: &str) -> i64 {
    let response = post(router, uri, body).await;
    assert_eq!(response.status(), StatusCode::CREATED, "{uri}");
    json_body(response).await["id"].as_i64().unwrap()
}

/// 예시 학기: 2027년 1학기, 2027-03-02 ~ 2027-06-30.
pub async fn create_term(router: Router) -> i64 {
    let body = json!({ "name": "2027년 1학기", "startsOn": "2027-03-02", "endsOn": "2027-06-30" });
    create(router, "/api/terms", &body.to_string()).await
}

/// 예시 과목. 분류는 없다.
pub async fn create_subject(router: Router, name: &str) -> i64 {
    let body = json!({ "name": name, "category": null });
    create(router, "/api/subjects", &body.to_string()).await
}

/// 예시 강의실: 본관 2층 201호.
pub async fn create_room(router: Router) -> i64 {
    let body = json!({ "name": "201호", "building": "본관", "floor": 2 });
    create(router, "/api/rooms", &body.to_string()).await
}

/// 예시 강좌: 수준·반 없음, 정원 20명, 월요일 10:00~11:00 한 줄.
pub async fn create_course(
    router: Router,
    term_id: i64,
    subject_id: i64,
    room_id: Option<i64>,
) -> i64 {
    let body = json!({
        "subjectId": subject_id,
        "level": null,
        "section": null,
        "capacity": 20,
        "schedules": [{ "weekday": 1, "start": "10:00", "end": "11:00", "roomId": room_id }],
    });
    create(
        router,
        &format!("/api/terms/{term_id}/courses"),
        &body.to_string(),
    )
    .await
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
