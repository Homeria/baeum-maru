//! 변경 알림 (ADR 0008): 데이터를 바꾸는 요청이 저장되면 연결된 화면들에 바뀐 종류만 알린다.
//!
//! - 알림에는 종류만 담는다. 어느 회원·강좌인지, 누가 바꿨는지는 담지 않는다.
//! - 어느 주소가 무엇을 바꾸는지는 주소를 등록할 때 [`notifies`]로 붙인다.
//! - 화면은 `GET /api/changes`로 연결을 열어 두고 알림을 받는다([`stream_changes`]).

use std::convert::Infallible;
use std::time::Duration;

use axum::extract::{Request, State};
use axum::http::{HeaderMap, Method};
use axum::middleware::{self, Next};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::MethodRouter;
use serde::Serialize;
use sqlx::SqlitePool;
use time::OffsetDateTime;
use tokio::sync::{broadcast, mpsc};
use tokio_stream::wrappers::ReceiverStream;
use ts_rs::TS;

use super::auth::{CurrentDevice, bearer_token, device_token, is_usable};
use super::error::ApiError;
use crate::db::{devices, sessions};

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
const BACKLOG: usize = 64;

/// 세션과 기기 승인이 아직 살아 있는지 다시 보는 간격.
const RECHECK_EVERY: Duration = Duration::from_secs(30);

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

    fn subscribe(&self) -> broadcast::Receiver<ChangeKind> {
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

/// `GET /api/changes`: 변경 알림을 SSE로 흘려보낸다.
///
/// 로그인 확인을 거친 뒤에만 열린다. 열린 뒤에도 세션이 잠기거나 끝났거나, 기기 승인이
/// 취소되면 끊는다. 이 확인은 세션의 마지막 요청 시각을 새로 적지 않는다(BR-K8).
///
/// # Errors
///
/// 증표가 머리글에 없으면(로그인 확인을 붙이지 않은 경우) [`ApiError::LoginRequired`] 또는
/// [`ApiError::DeviceNotApproved`].
pub async fn stream_changes(
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
struct Watch {
    session: [u8; 32],
    /// 다른 PC면 그 기기의 증표. 호스트 PC 자신이면 `None`.
    device: Option<[u8; 32]>,
}

impl Watch {
    async fn still_open(&self, db: &SqlitePool, now: OffsetDateTime) -> bool {
        match self.check(db, now).await {
            Ok(open) => open,
            Err(error) => {
                tracing::error!(%error, "변경 알림 연결을 확인하지 못했습니다");
                false
            }
        }
    }

    async fn check(&self, db: &SqlitePool, now: OffsetDateTime) -> Result<bool, sqlx::Error> {
        let Some(session) = sessions::find(db, &self.session).await? else {
            return Ok(false);
        };
        if !is_usable(&session.account) || baeum_domain::is_screen_locked(session.last_seen_at, now)
        {
            return Ok(false);
        }
        match &self.device {
            None => Ok(true),
            Some(device) => Ok(devices::find_approved(db, device).await?.is_some()),
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

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Response, StatusCode};
    use tokio_stream::StreamExt;

    use super::*;
    use crate::server::test_support::{
        app, app_without_staff, create_room, create_subject, create_term, get, post,
    };

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

    #[tokio::test]
    async fn needs_login() {
        let (_dir, _db, router) = app_without_staff().await;

        let response = get(router, "/api/changes").await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    /// 관리자 세션 하나와 승인한 기기 하나를 DB에 바로 넣고, 둘을 지켜보는 [`Watch`]를 만든다.
    async fn watch_on_approved_pc(db: &SqlitePool) -> Watch {
        let staff_id = crate::db::test_support::insert_staff(db, "지켜볼직원", "admin", None)
            .await
            .unwrap();
        let session = crate::secret::SessionToken::new().unwrap();
        sessions::start(
            db,
            staff_id,
            None,
            &session.hash(),
            OffsetDateTime::now_utc(),
        )
        .await
        .unwrap();
        let device = crate::secret::DeviceToken::new().unwrap();
        sqlx::query(
            "INSERT INTO devices (name, token_hash, approved_at, approved_by)
             VALUES ('시험 PC', ?, '2026-11-24T00:00:00.000Z', ?)",
        )
        .bind(device.hash().as_slice())
        .bind(staff_id)
        .execute(db)
        .await
        .unwrap();
        Watch {
            session: session.hash(),
            device: Some(device.hash()),
        }
    }

    #[tokio::test]
    async fn br_k16_revoked_device_ends_the_connection() {
        let (_dir, db, _router) = app_without_staff().await;
        let watch = watch_on_approved_pc(&db).await;
        let now = OffsetDateTime::now_utc();
        assert!(watch.still_open(&db, now).await);

        sqlx::query("UPDATE devices SET revoked_at = '2026-11-24T00:00:00.000Z'")
            .execute(&db)
            .await
            .unwrap();

        assert!(!watch.still_open(&db, now).await);
    }

    #[tokio::test]
    async fn br_k8_locked_session_ends_the_connection() {
        let (_dir, db, _router) = app_without_staff().await;
        let watch = watch_on_approved_pc(&db).await;

        let later = OffsetDateTime::now_utc() + time::Duration::minutes(10);

        assert!(!watch.still_open(&db, later).await);
    }

    #[tokio::test]
    async fn closed_session_ends_the_check() {
        let (_dir, db, _router) = app().await;
        let watch = Watch {
            session: [0; 32],
            device: None,
        };

        assert!(!watch.still_open(&db, OffsetDateTime::now_utc()).await);
    }
}
