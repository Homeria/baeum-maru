//! 변경 알림 (ADR 0008): 데이터를 바꾸는 요청이 저장되면 연결된 화면들에 바뀐 종류만 알린다.
//!
//! - 알림에는 종류만 담는다. 어느 회원·강좌인지, 누가 바꿨는지는 담지 않는다.
//! - 어느 주소가 무엇을 바꾸는지는 주소를 등록할 때 [`notifies`]로 붙인다.
//! - 화면은 `GET /api/changes`로 연결을 열어 두고 알림을 받는다. 연결은 세션과 기기를 다시 확인해야 해서
//!   `features::auth::handler`가 연다.

use axum::extract::{Request, State};
use axum::http::Method;
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::routing::MethodRouter;
use serde::Serialize;
use tokio::sync::broadcast;
use ts_rs::TS;

/// 바뀐 데이터의 종류. 이름은 용어집의 영어 이름을 소문자와 밑줄로 쓴다(ADR 0008 구현 메모).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ChangeKind {
    Member,
    Room,
    PriorityReason,
    Subject,
    Term,
    Course,
    /// 신청을 넣거나 지우거나 우선 사유를 바꿨다.
    Application,
    /// 추첨, 선발 마감, 포기, 연락 결과처럼 선발과 그 결과가 바뀌었다.
    Selection,
    Staff,
    Device,
    Backup,
}

/// 보내고 아직 받아 가지 않은 알림을 연결마다 이만큼 쌓아 둔다. 넘치면 그 연결을 끊는다.
pub(crate) const BACKLOG: usize = 64;

/// 알림을 보내는 곳. 복사하면 같은 채널을 함께 가리킨다.
#[derive(Debug, Clone)]
pub struct ChangeNotices {
    sender: broadcast::Sender<ChangeKind>,
}

impl ChangeNotices {
    pub fn new() -> Self {
        ChangeNotices {
            sender: broadcast::Sender::new(BACKLOG),
        }
    }

    /// 연결된 화면들에 알린다. 받는 화면이 없으면 아무 일도 없다.
    pub fn send(&self, kind: ChangeKind) {
        // 받는 쪽이 하나도 없을 때만 실패한다. 그때는 알릴 화면이 없다는 뜻이라 버린다
        if self.sender.send(kind).is_err() {
            tracing::trace!(?kind, "변경 알림을 받을 화면이 없습니다");
        }
    }

    pub(crate) fn subscribe(&self) -> broadcast::Receiver<ChangeKind> {
        self.sender.subscribe()
    }
}

impl Default for ChangeNotices {
    fn default() -> Self {
        Self::new()
    }
}

/// 주소의 처리 함수들에 변경 알림을 붙인다. 읽기(GET)가 아닌 요청이 성공하면 `kind`를 알린다.
pub fn notifies<S>(
    route: MethodRouter<S>,
    notices: &ChangeNotices,
    kind: ChangeKind,
) -> MethodRouter<S>
where
    S: Clone + Send + Sync + 'static,
{
    let target = Target {
        notices: notices.clone(),
        kind,
    };
    route.route_layer(middleware::from_fn_with_state(target, notify))
}

/// [`notifies`]가 미들웨어에 넘기는 것.
#[derive(Clone)]
struct Target {
    notices: ChangeNotices,
    kind: ChangeKind,
}

async fn notify(State(target): State<Target>, request: Request, next: Next) -> Response {
    let changes = request.method() != Method::GET;
    let response = next.run(request).await;
    // 실패한 요청은 아무것도 저장하지 않았다
    if changes && response.status().is_success() {
        target.notices.send(target.kind);
    }
    response
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use axum::body::Body;
    use axum::http::{Response, StatusCode};
    use tokio_stream::StreamExt;

    use crate::server::test_support::{app, create_room, create_subject, create_term, get, post};

    /// 응답 흐름에서 다음 알림 이름을 꺼낸다. 확인 신호(주석 줄)는 건너뛴다.
    async fn next_change(body: &mut axum::body::BodyDataStream) -> Option<String> {
        loop {
            let chunk = tokio::time::timeout(Duration::from_secs(2), body.next())
                .await
                .ok()??
                .ok()?;
            let text = String::from_utf8(chunk.to_vec()).ok()?;
            if let Some(data) = text.lines().find_map(|line| line.strip_prefix("data: ")) {
                return Some(data.to_owned());
            }
        }
    }

    async fn open(router: axum::Router) -> Response<Body> {
        let response = get(router, "/api/changes").await;
        assert_eq!(response.status(), StatusCode::OK);
        response
    }

    #[tokio::test]
    async fn saved_change_is_announced_by_kind() {
        let (_dir, _db, router) = app().await;
        let mut stream = open(router.clone()).await.into_body().into_data_stream();

        create_room(router).await;

        assert_eq!(next_change(&mut stream).await.as_deref(), Some(r#""room""#));
    }

    #[tokio::test]
    async fn failed_change_is_not_announced() {
        let (_dir, _db, router) = app().await;
        let mut stream = open(router.clone()).await.into_body().into_data_stream();

        let refused = post(router.clone(), "/api/rooms", r#"{"name":""}"#).await;
        create_subject(router, "서예").await;

        assert!(!refused.status().is_success());
        // 실패한 강의실 저장은 건너뛰고 다음 과목 알림이 먼저 온다
        assert_eq!(
            next_change(&mut stream).await.as_deref(),
            Some(r#""subject""#)
        );
    }

    #[tokio::test]
    async fn reading_is_not_announced() {
        let (_dir, _db, router) = app().await;
        let mut stream = open(router.clone()).await.into_body().into_data_stream();

        get(router.clone(), "/api/rooms").await;
        create_term(router).await;

        assert_eq!(next_change(&mut stream).await.as_deref(), Some(r#""term""#));
    }
}
