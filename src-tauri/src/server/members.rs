//! `/api/members`: 회원.

use axum::Json;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use baeum_domain::{Gender, MemberError, MemberInput, NewMember};
use chrono::{Local, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use ts_rs::TS;

use super::error::{ApiError, FieldError};
use crate::db::members::{self, InsertMemberError};

/// 성별. JSON으로는 `"male"`, `"female"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "Gender")]
pub enum GenderJson {
    Male,
    Female,
}

impl From<GenderJson> for Gender {
    fn from(gender: GenderJson) -> Self {
        match gender {
            GenderJson::Male => Gender::Male,
            GenderJson::Female => Gender::Female,
        }
    }
}

/// `POST /api/members` 본문. 확인 전이라 글자 그대로다.
#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CreateMemberRequest {
    pub member_no: String,
    pub name: String,
    pub phone_number: String,
    /// `YYYY-MM-DD`. 선택.
    pub birth_date: Option<NaiveDate>,
    /// 선택.
    pub gender: Option<GenderJson>,
}

impl CreateMemberRequest {
    fn as_input(&self) -> MemberInput<'_> {
        MemberInput {
            member_no: &self.member_no,
            name: &self.name,
            phone_number: &self.phone_number,
            birth_date: self.birth_date,
            gender: self.gender.map(Gender::from),
        }
    }
}

/// `POST /api/members` 응답.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct CreateMemberResponse {
    /// 새 회원의 DB id.
    // [ts-rs]: i64는 bigint가 되는데 JSON 숫자는 number로 읽힘. id는 2^53을 넘지 않음
    #[ts(type = "number")]
    pub id: i64,
}

/// 새 회원을 저장한다. 성공하면 201과 새 id를 돌려준다.
pub async fn create_member(
    State(db): State<SqlitePool>,
    body: Result<Json<CreateMemberRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let Json(request) = body?;
    let now = Utc::now();
    // 생년월일은 이 PC의 날짜로 비교 (한국에서 쓰므로 한국 날짜)
    let today = now.with_timezone(&Local).date_naive();

    let member = NewMember::parse(request.as_input(), today).map_err(invalid_input)?;
    let id = members::insert(&db, &member, now).await?;
    tracing::info!(member_id = id, "회원 저장");

    let location = format!("/api/members/{id}");
    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, location)],
        Json(CreateMemberResponse { id }),
    ))
}

fn invalid_input(errors: Vec<MemberError>) -> ApiError {
    let fields = errors
        .into_iter()
        .map(|error| FieldError {
            field: field_name(error).to_owned(),
            message: error.to_string(),
        })
        .collect();
    ApiError::InvalidInput(fields)
}

/// 오류가 난 칸의 요청 JSON 이름.
fn field_name(error: MemberError) -> &'static str {
    match error {
        MemberError::InvalidMemberNo => "memberNo",
        MemberError::EmptyName | MemberError::NameTooLong => "name",
        MemberError::InvalidPhoneNumber => "phoneNumber",
        MemberError::BirthDateInFuture => "birthDate",
    }
}

impl From<InsertMemberError> for ApiError {
    fn from(error: InsertMemberError) -> Self {
        match error {
            InsertMemberError::DuplicateMemberNo => ApiError::DuplicateMemberNo,
            InsertMemberError::Db(error) => ApiError::internal(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::Router;
    use axum::body::Body;
    use axum::http::{Request, Response};
    use serde_json::{Value, json};
    use tower::ServiceExt;

    use super::*;
    use crate::server::{AppState, router};

    async fn app() -> (tempfile::TempDir, SqlitePool, Router) {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path()).await.unwrap();
        let router = router(AppState::new("0.1.0".parse().unwrap(), db.clone()));
        (dir, db, router)
    }

    async fn post(router: Router, body: &str) -> Response<Body> {
        let request = Request::post("/api/members")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_owned()))
            .unwrap();
        router.oneshot(request).await.unwrap()
    }

    async fn json_body(response: Response<Body>) -> Value {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    fn valid(member_no: &str) -> String {
        json!({
            "memberNo": member_no,
            "name": "회원1",
            "phoneNumber": "010-0000-0001",
            "birthDate": "1950-03-01",
            "gender": "female",
        })
        .to_string()
    }

    async fn count(db: &SqlitePool) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM members")
            .fetch_one(db)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn creates_member() {
        let (_dir, db, router) = app().await;

        let response = post(router, &valid("00001")).await;

        assert_eq!(response.status(), StatusCode::CREATED);
        // body를 읽으면 response가 사라지므로 헤더 글자를 먼저 따로 가져 둠
        let location = response.headers()[header::LOCATION]
            .to_str()
            .unwrap()
            .to_owned();
        let body = json_body(response).await;
        assert_eq!(location, format!("/api/members/{}", body["id"]));
        assert_eq!(count(&db).await, 1);
    }

    #[tokio::test]
    async fn optional_fields_can_be_null() {
        let (_dir, _db, router) = app().await;
        let body = json!({
            "memberNo": "00001",
            "name": "회원1",
            "phoneNumber": "01000000001",
            "birthDate": null,
            "gender": null,
        });

        let response = post(router, &body.to_string()).await;

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn invalid_input_returns_every_field() {
        let (_dir, db, router) = app().await;
        let body = json!({
            "memberNo": "번호",
            "name": " ",
            "phoneNumber": "12",
            "birthDate": "9999-01-01",
            "gender": null,
        });

        let response = post(router, &body.to_string()).await;

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = json_body(response).await;
        assert_eq!(body["code"], "INVALID_INPUT");
        let fields: Vec<&str> = body["fields"]
            .as_array()
            .unwrap()
            .iter()
            .map(|field| field["field"].as_str().unwrap())
            .collect();
        assert_eq!(fields, ["memberNo", "name", "phoneNumber", "birthDate"]);
        assert_eq!(count(&db).await, 0);
    }

    #[tokio::test]
    async fn br_m2_duplicate_member_no_is_conflict() {
        let (_dir, db, router) = app().await;
        post(router.clone(), &valid("00001")).await;

        let response = post(router, &valid("00001")).await;

        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = json_body(response).await;
        assert_eq!(body["code"], "DUPLICATE_MEMBER_NO");
        assert_eq!(body["rule"], "BR-M2");
        assert_eq!(count(&db).await, 1);
    }

    #[tokio::test]
    async fn malformed_body_does_not_echo_input() {
        let (_dir, _db, router) = app().await;
        let body = json!({
            "memberNo": "00001",
            "name": "회원1",
            "phoneNumber": "01000000001",
            "birthDate": "1950-13-01",
            "gender": null,
        });

        let response = post(router, &body.to_string()).await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = json_body(response).await;
        assert_eq!(body["code"], "MALFORMED_REQUEST");
        assert!(!body.to_string().contains("1950-13-01"));
    }

    #[tokio::test]
    async fn unknown_gender_is_malformed() {
        let (_dir, _db, router) = app().await;
        let body = json!({
            "memberNo": "00001",
            "name": "회원1",
            "phoneNumber": "01000000001",
            "birthDate": null,
            "gender": "other",
        });

        let response = post(router, &body.to_string()).await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
