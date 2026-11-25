//! `/api/staff`: 직원 관리 (BR-K2~K6, BR-K9, BR-K10). 관리자만 쓴다.
//!
//! 직원은 지우지 않고 사용 중지한다 (BR-K6). 그래서 지우기 주소가 없다.

mod json;

use axum::Json;
use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use baeum_domain::{Action, NewStaff, Pin, StaffError, StaffProfile};

use sqlx::SqlitePool;
use time::OffsetDateTime;

use crate::db::staff::{self, SaveStaffError};
use crate::http::auth::CurrentStaff;
use crate::http::error::ApiError;
use crate::infra::secret;

use crate::infra::clock::local_today;
pub use json::RoleCode;
use json::{
    CreateStaffRequest, CreateStaffResponse, ResetPinRequest, StaffListResponse, StaffResponse,
    UpdateStaffRequest,
};

/// 직원을 모두 보여 준다. 사용 중인 직원 먼저, 이름 순서.
pub async fn list_staff(
    State(db): State<SqlitePool>,
    current: CurrentStaff,
) -> Result<Json<StaffListResponse>, ApiError> {
    current.require(Action::ManageStaff)?;
    let records = staff::list(&db).await.map_err(ApiError::internal)?;
    let now = OffsetDateTime::now_utc();
    Ok(Json(StaffListResponse {
        items: records
            .into_iter()
            .map(|record| StaffResponse::new(record, now))
            .collect(),
    }))
}

/// `id`인 직원 하나를 보여 준다.
pub async fn get_staff(
    State(db): State<SqlitePool>,
    current: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
) -> Result<Json<StaffResponse>, ApiError> {
    current.require(Action::ManageStaff)?;
    let Path(id) = id?;
    let record = staff::get(&db, id)
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::StaffNotFound)?;
    Ok(Json(StaffResponse::new(record, OffsetDateTime::now_utc())))
}

/// 새 직원을 저장한다. 성공하면 201과 새 id를 돌려준다.
pub async fn create_staff(
    State(db): State<SqlitePool>,
    current: CurrentStaff,
    body: Result<Json<CreateStaffRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    current.require(Action::ManageStaff)?;
    let Json(request) = body?;
    let new_staff = NewStaff::parse(request.as_input(), &request.pin)
        .map_err(|errors| ApiError::invalid_input(errors, field_name))?;

    // 해시는 일부러 느려서 따로 돌린다. 새 직원을 넘겼다가 돌려받는다
    let (new_staff, pin_hash) = tokio::task::spawn_blocking(move || {
        let pin_hash = secret::hash(new_staff.pin().as_str());
        (new_staff, pin_hash)
    })
    .await
    .map_err(ApiError::internal)?;
    let pin_hash = pin_hash.map_err(ApiError::internal)?;

    let id = staff::insert(&db, &new_staff, &pin_hash, OffsetDateTime::now_utc()).await?;
    tracing::info!(staff_id = current.id, target_staff_id = id, "직원 저장");

    let location = format!("/api/staff/{id}");
    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, location)],
        Json(CreateStaffResponse { id }),
    ))
}

/// `id`인 직원의 이름·역할·유효기간·사용 중지를 본문 값으로 바꾼다.
///
/// BR-K9: 사용 중인 관리자가 하나도 남지 않게 되면 409.
pub async fn update_staff(
    State(db): State<SqlitePool>,
    current: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
    body: Result<Json<UpdateStaffRequest>, JsonRejection>,
) -> Result<Json<StaffResponse>, ApiError> {
    current.require(Action::ManageStaff)?;
    let Path(id) = id?;
    let Json(request) = body?;
    let profile = StaffProfile::parse(request.as_input())
        .map_err(|errors| ApiError::invalid_input(errors, field_name))?;

    let today = local_today();
    let now = OffsetDateTime::now_utc();
    let record = staff::update(&db, id, &profile, request.disabled, today, now)
        .await?
        .ok_or(ApiError::StaffNotFound)?;
    tracing::info!(staff_id = current.id, target_staff_id = id, "직원 고침");
    Ok(Json(StaffResponse::new(record, now)))
}

/// BR-K10: `id`인 직원의 PIN을 새로 정한다. 계정 잠금도 풀린다. 성공하면 204(본문 없음).
///
/// PIN 길이는 그 직원의 역할로 확인한다 (BR-K4: 관리자는 6자리 이상).
pub async fn reset_staff_pin(
    State(db): State<SqlitePool>,
    current: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
    body: Result<Json<ResetPinRequest>, JsonRejection>,
) -> Result<StatusCode, ApiError> {
    current.require(Action::ManageStaff)?;
    let Path(id) = id?;
    let Json(request) = body?;
    let target = staff::get(&db, id)
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::StaffNotFound)?;
    let pin = Pin::parse(&request.pin, target.role)
        .map_err(|error| ApiError::invalid_input(vec![error], field_name))?;

    let pin_hash = tokio::task::spawn_blocking(move || secret::hash(pin.as_str()))
        .await
        .map_err(ApiError::internal)?
        .map_err(ApiError::internal)?;
    let changed = staff::reset_pin(&db, id, &pin_hash, OffsetDateTime::now_utc())
        .await
        .map_err(ApiError::internal)?;
    if !changed {
        return Err(ApiError::StaffNotFound);
    }
    tracing::info!(
        staff_id = current.id,
        target_staff_id = id,
        "직원 PIN 새로 정함"
    );
    Ok(StatusCode::NO_CONTENT)
}

/// 오류가 난 칸의 요청 JSON 이름. 처음 설정도 함께 쓴다.
pub(super) fn field_name(error: &StaffError) -> &'static str {
    match error {
        StaffError::EmptyName | StaffError::NameTooLong => "name",
        StaffError::InvalidPin | StaffError::AdminPinTooShort => "pin",
        StaffError::MissingValidUntil => "validUntil",
    }
}

impl From<SaveStaffError> for ApiError {
    fn from(error: SaveStaffError) -> Self {
        match error {
            SaveStaffError::DuplicateName => ApiError::DuplicateStaffName,
            SaveStaffError::LastAdmin => ApiError::LastAdmin,
            SaveStaffError::Db(error) => ApiError::internal(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::Router;
    use serde_json::{Value, json};

    use super::*;
    use crate::server::test_support::{app, app_as, create, get, json_body, patch, post};

    fn new_staff(name: &str, role: &str, valid_until: Option<&str>, pin: &str) -> String {
        json!({ "name": name, "role": role, "validUntil": valid_until, "pin": pin }).to_string()
    }

    fn update(name: &str, role: &str, valid_until: Option<&str>, disabled: bool) -> String {
        json!({ "name": name, "role": role, "validUntil": valid_until, "disabled": disabled })
            .to_string()
    }

    async fn list(router: Router) -> Value {
        json_body(get(router, "/api/staff").await).await["items"].clone()
    }

    /// `app()`이 로그인해 둔 관리자의 id.
    async fn admin_id(router: Router) -> i64 {
        list(router).await[0]["id"].as_i64().unwrap()
    }

    #[tokio::test]
    async fn only_admin_manages_staff() {
        for role in ["staff", "temp_staff", "read_only"] {
            let (_dir, _db, router) = app_as(role).await;

            let responses = [
                get(router.clone(), "/api/staff").await,
                get(router.clone(), "/api/staff/1").await,
                post(router.clone(), "/api/staff", "{}").await,
                patch(router.clone(), "/api/staff/1", "{}").await,
                post(router.clone(), "/api/staff/1/pin", "{}").await,
            ];

            for response in responses {
                assert_eq!(response.status(), StatusCode::FORBIDDEN, "{role}");
                assert_eq!(json_body(response).await["code"], "NOT_ALLOWED");
            }
        }
    }

    #[tokio::test]
    async fn creates_and_lists_staff() {
        let (_dir, _db, router) = app().await;

        let id = create(
            router.clone(),
            "/api/staff",
            &new_staff("가 임시", "temp_staff", Some("2027-02-28"), "1234"),
        )
        .await;

        let items = list(router.clone()).await;
        assert_eq!(
            items[0],
            json!({
                "id": id,
                "name": "가 임시",
                "role": "temp_staff",
                "validUntil": "2027-02-28",
                "disabled": false,
                "locked": false,
            })
        );
        assert_eq!(items[1]["name"], "시험직원");
        let response = get(router, &format!("/api/staff/{id}")).await;
        assert_eq!(json_body(response).await["name"], "가 임시");
    }

    #[tokio::test]
    async fn rejects_invalid_new_staff() {
        let (_dir, _db, router) = app().await;

        let response = post(
            router.clone(),
            "/api/staff",
            &new_staff("", "temp_staff", None, "12"),
        )
        .await;

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = json_body(response).await;
        let fields: Vec<&str> = body["fields"]
            .as_array()
            .unwrap()
            .iter()
            .map(|field| field["field"].as_str().unwrap())
            .collect();
        assert_eq!(fields, ["name", "pin", "validUntil"]);
        assert!(!body.to_string().contains("\"12\""));

        let unknown_role = new_staff("직원", "boss", None, "1234");
        let response = post(router, "/api/staff", &unknown_role).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_duplicate_name() {
        let (_dir, _db, router) = app().await;

        let response = post(
            router,
            "/api/staff",
            &new_staff("시험 직원", "staff", None, "1234"),
        )
        .await;

        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = json_body(response).await;
        assert_eq!(body["code"], "DUPLICATE_STAFF_NAME");
        assert_eq!(body["rule"], "BR-K2");
    }

    #[tokio::test]
    async fn updates_role_and_disables_staff() {
        let (_dir, _db, router) = app().await;
        let id = create(
            router.clone(),
            "/api/staff",
            &new_staff("직원1", "staff", None, "1234"),
        )
        .await;

        let response = patch(
            router,
            &format!("/api/staff/{id}"),
            &update("직원 1", "read_only", None, true),
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        let body = json_body(response).await;
        assert_eq!(body["name"], "직원 1");
        assert_eq!(body["role"], "read_only");
        assert_eq!(body["disabled"], true);
    }

    #[tokio::test]
    async fn last_admin_stays_admin() {
        let (_dir, _db, router) = app().await;
        let admin_id = admin_id(router.clone()).await;
        let uri = format!("/api/staff/{admin_id}");

        for body in [
            update("시험직원", "staff", None, false),
            update("시험직원", "admin", None, true),
        ] {
            let response = patch(router.clone(), &uri, &body).await;

            assert_eq!(response.status(), StatusCode::CONFLICT);
            let body = json_body(response).await;
            assert_eq!(body["code"], "LAST_ADMIN");
            assert_eq!(body["rule"], "BR-K9");
        }
    }

    #[tokio::test]
    async fn missing_staff_is_not_found() {
        let (_dir, _db, router) = app().await;

        let responses = [
            get(router.clone(), "/api/staff/999").await,
            patch(
                router.clone(),
                "/api/staff/999",
                &update("없음", "staff", None, false),
            )
            .await,
            post(router, "/api/staff/999/pin", r#"{"pin":"1234"}"#).await,
        ];

        for response in responses {
            assert_eq!(response.status(), StatusCode::NOT_FOUND);
            assert_eq!(json_body(response).await["code"], "STAFF_NOT_FOUND");
        }
    }

    #[tokio::test]
    async fn new_pin_works_for_login_and_unlocks_account() {
        let (_dir, db, router) = app().await;
        let id = create(
            router.clone(),
            "/api/staff",
            &new_staff("직원1", "staff", None, "1234"),
        )
        .await;
        sqlx::query("UPDATE staff SET locked_until = '2999-01-01T00:00:00.000Z' WHERE id = ?")
            .bind(id)
            .execute(&db)
            .await
            .unwrap();
        let staff_uri = format!("/api/staff/{id}");
        assert_eq!(
            json_body(get(router.clone(), &staff_uri).await).await["locked"],
            true
        );

        let response = post(
            router.clone(),
            &format!("{staff_uri}/pin"),
            r#"{"pin":"5678"}"#,
        )
        .await;

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert_eq!(
            json_body(get(router.clone(), &staff_uri).await).await["locked"],
            false
        );
        let login = json!({ "staffId": id, "pin": "5678" }).to_string();
        let response = post(router, "/api/login", &login).await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn admin_pin_needs_six_digits() {
        let (_dir, _db, router) = app().await;
        let admin_id = admin_id(router.clone()).await;

        let response = post(
            router,
            &format!("/api/staff/{admin_id}/pin"),
            r#"{"pin":"1234"}"#,
        )
        .await;

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = json_body(response).await;
        assert_eq!(body["fields"][0]["field"], "pin");
        assert_eq!(
            body["fields"][0]["message"],
            "관리자 PIN은 6자리 이상입니다"
        );
    }
}
