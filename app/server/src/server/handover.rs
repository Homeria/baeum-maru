//! `POST /api/handover`: 호스트 넘기기를 시작한다(BR-H1).
//!
//! 서버는 관리자 PIN을 확인하고, 접속 기록에 남기고, 실행 모드를 "넘기는 중"으로 바꿔 둘 뿐이다.
//! 화면이 앱을 다시 켜면 서버 없이 DB만 열어 넘기기 파일을 만든다(`crate::handover`).
//! 그래서 넘기기 파일을 만드는 동안 다른 PC가 저장하는 일이 없다.

use axum::Json;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use baeum_domain::Action;
use serde::Deserialize;
use sqlx::SqlitePool;
use time::OffsetDateTime;
use ts_rs::TS;

use crate::backup::Backups;
use crate::features::access_logs::repository::{self as access_logs, AccessEvent};
use crate::features::auth::repository as auth;
use crate::features::auth::service::check_pin;
use crate::http::auth::{Client, CurrentStaff};
use crate::http::error::ApiError;
use crate::run_mode::{self, RunMode};

/// `POST /api/handover` 본문. PIN이 들어 있어 `Debug`를 만들지 않는다.
#[derive(Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HandoverRequest {
    /// 로그인한 관리자의 PIN. 한 번 더 확인한다.
    pub pin: String,
}

/// 관리자 PIN이 맞으면 넘기기를 시작하고 204(본문 없음)를 돌려준다. 호스트 PC에서만.
///
/// 틀린 PIN은 로그인과 같이 BR-K7의 횟수에 들어가고 로그인 실패로 남는다.
pub async fn start_handover(
    State(db): State<SqlitePool>,
    State(backups): State<Backups>,
    staff: CurrentStaff,
    client: Client,
    body: Result<Json<HandoverRequest>, JsonRejection>,
) -> Result<StatusCode, ApiError> {
    staff.require(Action::HandOverHost)?;
    client.require_host()?;
    let Json(HandoverRequest { pin }) = body?;
    let now = OffsetDateTime::now_utc();

    let account = auth::get_login_account(&db, staff.id)
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::StaffNotFound)?;
    // 호스트 PC에서만 하므로 기기는 늘 호스트 자신이다
    let account = check_pin(&db, account, pin, None, AccessEvent::LoginFailed, now).await?;

    // 남기지 못하면 넘기지 않는다
    access_logs::record(&db, Some(account.id), AccessEvent::Handover, now)
        .await
        .map_err(ApiError::internal)?;
    run_mode::save(backups.data_dir(), RunMode::HandingOver).map_err(ApiError::internal)?;
    tracing::info!(staff_id = account.id, "호스트 넘기기 시작");
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use axum::Router;
    use serde_json::json;

    use super::*;
    use crate::features::access_logs::repository::list_for_test;
    use crate::server::test_support::{app, app_as, from_approved_pc, json_body, post};

    /// 로그인해 둔 첫 직원의 PIN을 123456으로 정한다.
    async fn set_pin(db: &SqlitePool) {
        sqlx::query("UPDATE staff SET pin_hash = ?")
            .bind(crate::infra::secret::hash("123456").unwrap())
            .execute(db)
            .await
            .unwrap();
    }

    async fn hand_over(router: Router, pin: &str) -> axum::response::Response {
        let body = json!({ "pin": pin }).to_string();
        post(router, "/api/handover", &body).await
    }

    #[tokio::test]
    async fn br_h1_admin_pin_starts_handover_and_is_logged() {
        let (dir, db, router) = app().await;
        set_pin(&db).await;

        let response = hand_over(router, "123456").await;

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert_eq!(run_mode::resolve(dir.path()), Some(RunMode::HandingOver));
        assert_eq!(
            list_for_test(&db).await.last().unwrap().1,
            "handover".to_owned()
        );
    }

    #[tokio::test]
    async fn wrong_pin_does_not_hand_over() {
        let (dir, db, router) = app().await;
        set_pin(&db).await;

        let response = hand_over(router, "000000").await;

        assert_eq!(json_body(response).await["code"], "WRONG_PIN");
        assert_ne!(run_mode::resolve(dir.path()), Some(RunMode::HandingOver));
        assert_eq!(
            list_for_test(&db).await.last().unwrap().1,
            "login_failed".to_owned()
        );
    }

    #[tokio::test]
    async fn only_admin_can_hand_over() {
        let (dir, db, router) = app_as("staff").await;
        set_pin(&db).await;

        let response = hand_over(router, "123456").await;

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        assert_ne!(run_mode::resolve(dir.path()), Some(RunMode::HandingOver));
    }

    #[tokio::test]
    async fn br_h1_only_on_host_pc() {
        let (dir, db, router) = app().await;
        set_pin(&db).await;
        let router = from_approved_pc(&db, router).await;

        let response = hand_over(router, "123456").await;

        assert_eq!(json_body(response).await["code"], "HOST_ONLY");
        assert_ne!(run_mode::resolve(dir.path()), Some(RunMode::HandingOver));
    }
}
