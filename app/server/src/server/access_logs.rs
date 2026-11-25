//! `/api/access-logs`: 접속 기록 보기 (BR-K13). 관리자만 본다.

mod json;

use axum::Json;
use axum::extract::rejection::QueryRejection;
use axum::extract::{Query, State};
use baeum_domain::Action;
use sqlx::SqlitePool;

use super::auth::CurrentStaff;
use super::error::ApiError;
use super::paging::Paging;
use crate::db::access_logs;

use json::{AccessLogListResponse, AccessLogParams, AccessLogResponse};

/// 한 쪽에 보여 주는 기본 줄 수.
const DEFAULT_PAGE_SIZE: u32 = 50;

/// 접속 기록을 최근 것부터 한 쪽씩 보여 준다.
pub async fn list_access_logs(
    State(db): State<SqlitePool>,
    current: CurrentStaff,
    params: Result<Query<AccessLogParams>, QueryRejection>,
) -> Result<Json<AccessLogListResponse>, ApiError> {
    current.require(Action::ViewAccessLog)?;
    let Query(params) = params?;
    let Paging { page, page_size } =
        Paging::parse(params.page, params.page_size, DEFAULT_PAGE_SIZE)?;

    let found = access_logs::list(&db, page, page_size)
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

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use serde_json::json;

    use crate::server::test_support::{app, app_as, get, json_body, post};

    #[tokio::test]
    async fn admin_sees_newest_first() {
        let (_dir, _db, router) = app().await;
        // app()이 로그인을 한 줄 남겼다. 없는 직원으로 로그인 실패를 하나 더한다
        let login = json!({ "staffId": 999, "pin": "123456" }).to_string();
        post(router.clone(), "/api/login", &login).await;

        let response = get(router.clone(), "/api/access-logs?pageSize=1").await;

        assert_eq!(response.status(), StatusCode::OK);
        let body = json_body(response).await;
        assert_eq!(body["total"], 2);
        assert_eq!(body["page"], 1);
        assert_eq!(body["pageSize"], 1);
        let item = &body["items"][0];
        assert_eq!(item["event"], "login_failed");
        assert_eq!(item["staffId"], json!(null));
        assert_eq!(item["staffName"], json!(null));
        assert!(item["occurredAt"].as_str().unwrap().ends_with('Z'));

        let second = json_body(get(router, "/api/access-logs?page=2&pageSize=1").await).await;
        assert_eq!(second["items"][0]["event"], "login");
        assert_eq!(second["items"][0]["staffName"], "시험직원");
    }

    #[tokio::test]
    async fn only_admin_sees_access_logs() {
        for role in ["staff", "temp_staff", "read_only"] {
            let (_dir, _db, router) = app_as(role).await;

            let response = get(router, "/api/access-logs").await;

            assert_eq!(response.status(), StatusCode::FORBIDDEN, "{role}");
        }
    }

    #[tokio::test]
    async fn rejects_page_out_of_range() {
        let (_dir, _db, router) = app().await;

        let response = get(router, "/api/access-logs?page=0&pageSize=101").await;

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = json_body(response).await;
        assert_eq!(body["fields"][0]["field"], "page");
        assert_eq!(body["fields"][1]["field"], "pageSize");
    }
}
