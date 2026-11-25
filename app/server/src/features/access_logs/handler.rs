//! `/api/access-logs`: 접속 기록 보기 (BR-K13). 관리자만 본다.

use axum::Json;
use axum::Router;
use axum::extract::FromRef;
use axum::extract::rejection::QueryRejection;
use axum::extract::{Query, State};
use axum::routing::get;
use baeum_domain::Action;
use sqlx::SqlitePool;

use super::dto::{AccessLogListResponse, AccessLogParams, AccessLogResponse};
use super::repository;
use crate::http::auth::CurrentStaff;
use crate::http::changes::ChangeNotices;
use crate::http::error::ApiError;
use crate::http::paging::Paging;

/// 한 쪽에 보여 주는 기본 줄 수.
const DEFAULT_PAGE_SIZE: u32 = 50;

/// 접속 기록 요청 주소. 로그인한 요청만 받는 묶음에 들어간다 (ADR 0023).
pub(crate) fn routes<S>(_notices: &ChangeNotices) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    SqlitePool: FromRef<S>,
{
    Router::new().route("/api/access-logs", get(list_access_logs))
}

/// 접속 기록을 최근 것부터 한 쪽씩 보여 준다.
async fn list_access_logs(
    State(db): State<SqlitePool>,
    current: CurrentStaff,
    params: Result<Query<AccessLogParams>, QueryRejection>,
) -> Result<Json<AccessLogListResponse>, ApiError> {
    current.require(Action::ViewAccessLog)?;
    let Query(params) = params?;
    let Paging { page, page_size } =
        Paging::parse(params.page, params.page_size, DEFAULT_PAGE_SIZE)?;

    let found = repository::list(&db, page, page_size)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(AccessLogListResponse {
        items: found
            .items
            .into_iter()
            .map(AccessLogResponse::from)
            .collect(),
        total: found.total,
        page,
        page_size,
    }))
}
