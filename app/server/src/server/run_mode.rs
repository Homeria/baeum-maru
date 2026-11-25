//! `/api/run-mode/permit`: 실행 모드 바꾸기를 관리자 PIN으로 허락받는다 (BR-K17, ADR 0021).
//!
//! 바꾸는 일 자체는 그 PC의 앱이 한다. 호스트는 관리자 PIN을 확인하고 접속 기록에 남길 뿐이다.
//! 접속 모드 PC는 직원 계정이 없어서 호스트에 묻는다.

use axum::Json;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use baeum_domain::Role;
use serde::Deserialize;
use sqlx::SqlitePool;
use time::OffsetDateTime;
use ts_rs::TS;

use crate::features::access_logs::repository::{self as access_logs, AccessEvent};
use crate::features::auth::repository as auth;
use crate::features::auth::service::check_pin;
use crate::http::auth::CurrentDevice;
use crate::http::error::ApiError;

/// `POST /api/run-mode/permit` 본문. PIN이 들어 있어 `Debug`를 만들지 않는다.
#[derive(Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RunModePermitRequest {
    /// 로그인 화면의 직원 목록에서 고른 관리자.
    #[ts(type = "number")]
    pub staff_id: i64,
    pub pin: String,
}

/// 고른 관리자의 PIN이 맞으면 204(본문 없음)를 돌려주고 접속 기록에 남긴다.
///
/// 틀린 PIN은 로그인과 같이 BR-K7의 횟수에 들어가고 로그인 실패로 남는다.
pub async fn permit(
    State(db): State<SqlitePool>,
    device: CurrentDevice,
    body: Result<Json<RunModePermitRequest>, JsonRejection>,
) -> Result<StatusCode, ApiError> {
    let Json(RunModePermitRequest { staff_id, pin }) = body?;
    let now = OffsetDateTime::now_utc();

    let account = auth::get_login_account(&db, staff_id)
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::StaffNotFound)?;
    if account.role != Role::Admin {
        return Err(ApiError::NotAllowed);
    }
    let account = check_pin(&db, account, pin, device.id, AccessEvent::LoginFailed, now).await?;

    access_logs::record_on_device(
        &db,
        Some(account.id),
        device.id,
        AccessEvent::RunModeChanged,
        now,
    )
    .await
    .map_err(ApiError::internal)?;
    tracing::info!(staff_id = account.id, "실행 모드 바꾸기 허락");
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use axum::Router;
    use serde_json::json;

    use super::*;
    use crate::db::test_support::insert_staff;
    use crate::features::access_logs::repository::list_for_test;
    use crate::server::test_support::{app_without_staff, get, json_body, post};

    /// 처음 설정으로 관리자(PIN 123456)를 만들고 id를 돌려준다.
    async fn setup_admin(router: Router) -> i64 {
        let setup = json!({ "name": "관리자", "pin": "123456" }).to_string();
        post(router.clone(), "/api/setup", &setup).await;
        let list = json_body(get(router, "/api/login/staff").await).await;
        list["items"][0]["id"].as_i64().unwrap()
    }

    fn body(staff_id: i64, pin: &str) -> String {
        json!({ "staffId": staff_id, "pin": pin }).to_string()
    }

    #[tokio::test]
    async fn br_k17_admin_pin_permits_and_is_logged() {
        let (_dir, db, router) = app_without_staff().await;
        let admin = setup_admin(router.clone()).await;

        let response = post(router, "/api/run-mode/permit", &body(admin, "123456")).await;

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert_eq!(
            list_for_test(&db).await.last().unwrap(),
            &(Some(admin), "run_mode_changed".to_owned())
        );
    }

    #[tokio::test]
    async fn wrong_pin_counts_as_failed_login() {
        let (_dir, db, router) = app_without_staff().await;
        let admin = setup_admin(router.clone()).await;

        let response = post(router, "/api/run-mode/permit", &body(admin, "000000")).await;

        assert_eq!(json_body(response).await["code"], "WRONG_PIN");
        assert_eq!(
            list_for_test(&db).await.last().unwrap(),
            &(Some(admin), "login_failed".to_owned())
        );
    }

    #[tokio::test]
    async fn only_admin_can_permit() {
        let (_dir, db, router) = app_without_staff().await;
        setup_admin(router.clone()).await;
        let staff_id = insert_staff(&db, "직원1", "staff", None).await.unwrap();

        let response = post(router, "/api/run-mode/permit", &body(staff_id, "1234")).await;

        assert_eq!(json_body(response).await["code"], "NOT_ALLOWED");
    }

    #[tokio::test]
    async fn unapproved_pc_cannot_ask() {
        let (_dir, _db, router) = app_without_staff().await;
        let admin = setup_admin(router.clone()).await;
        let other = crate::server::test_support::from_other_pc(router);

        let response = post(other, "/api/run-mode/permit", &body(admin, "123456")).await;

        assert_eq!(json_body(response).await["code"], "DEVICE_NOT_APPROVED");
    }
}
