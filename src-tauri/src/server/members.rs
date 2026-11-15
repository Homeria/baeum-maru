//! `/api/members`: 회원.

use axum::Json;
use axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection};
use axum::extract::{Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use baeum_domain::{Gender, MemberError, MemberInput, NewMember, Personal};
use chrono::{Local, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use ts_rs::TS;

use super::error::{ApiError, FieldError};
use crate::db::members::{self, MemberRecord, SaveMemberError};
use crate::db::timestamp_text;

/// 성별. JSON으로는 `"male"`, `"female"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
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

impl From<Gender> for GenderJson {
    fn from(gender: Gender) -> Self {
        match gender {
            Gender::Male => GenderJson::Male,
            Gender::Female => GenderJson::Female,
        }
    }
}

/// `POST /api/members`, `PATCH /api/members/{id}` 본문. 확인 전이라 글자 그대로다.
///
/// 고칠 때도 다섯 칸을 모두 보낸다. 비운 선택 칸은 `null`이다.
///
/// 회원 정보가 그대로 들어 있어 `Debug`를 붙이지 않는다 (로그에 찍히지 않게).
#[derive(Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MemberRequest {
    pub member_no: String,
    pub name: String,
    pub phone_number: String,
    /// `YYYY-MM-DD`. 선택.
    pub birth_date: Option<NaiveDate>,
    /// 선택.
    pub gender: Option<GenderJson>,
}

impl MemberRequest {
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
    body: Result<Json<MemberRequest>, JsonRejection>,
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

/// 회원 한 명. `GET`·`PATCH /api/members/{id}` 응답.
///
/// 회원 정보가 그대로 들어 있어 `Debug`를 붙이지 않는다.
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MemberResponse {
    #[ts(type = "number")]
    pub id: i64,
    pub member_no: String,
    pub name: String,
    /// 숫자만.
    pub phone_number: String,
    pub birth_date: Option<NaiveDate>,
    pub gender: Option<GenderJson>,
    /// UTC, 밀리초까지, 끝에 Z. 화면에서 한국 시각으로 바꾼다.
    pub created_at: String,
    /// `created_at`과 같은 모양.
    pub updated_at: String,
}

impl From<MemberRecord> for MemberResponse {
    fn from(record: MemberRecord) -> Self {
        MemberResponse {
            id: record.id,
            member_no: record.member_no.into_inner(),
            name: record.name.into_inner(),
            phone_number: record.phone_number.into_inner(),
            birth_date: record.birth_date.map(Personal::into_inner),
            gender: record
                .gender
                .map(|gender| GenderJson::from(gender.into_inner())),
            created_at: timestamp_text(record.created_at),
            updated_at: timestamp_text(record.updated_at),
        }
    }
}

/// `id`인 회원 한 명을 보여 준다.
pub async fn get_member(
    State(db): State<SqlitePool>,
    id: Result<Path<i64>, PathRejection>,
) -> Result<Json<MemberResponse>, ApiError> {
    let Path(id) = id?;
    let record = members::get(&db, id)
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::MemberNotFound)?;
    Ok(Json(MemberResponse::from(record)))
}

/// `id`인 회원의 칸을 모두 본문 값으로 바꾼다. 확인 규칙은 등록과 같다.
pub async fn update_member(
    State(db): State<SqlitePool>,
    id: Result<Path<i64>, PathRejection>,
    body: Result<Json<MemberRequest>, JsonRejection>,
) -> Result<Json<MemberResponse>, ApiError> {
    let Path(id) = id?;
    let Json(request) = body?;
    let now = Utc::now();
    let today = now.with_timezone(&Local).date_naive();

    let member = NewMember::parse(request.as_input(), today).map_err(invalid_input)?;
    let record = members::update(&db, id, &member, now)
        .await?
        .ok_or(ApiError::MemberNotFound)?;
    tracing::info!(member_id = id, "회원 고침");
    Ok(Json(MemberResponse::from(record)))
}

/// `id`인 회원을 지운다. 성공하면 204(본문 없음).
///
/// BR-M5: 신청 기록이 있는 회원은 신청 기능에서 409로 막는다.
pub async fn delete_member(
    State(db): State<SqlitePool>,
    id: Result<Path<i64>, PathRejection>,
) -> Result<StatusCode, ApiError> {
    let Path(id) = id?;
    let deleted = members::delete(&db, id).await.map_err(ApiError::internal)?;
    if !deleted {
        return Err(ApiError::MemberNotFound);
    }
    tracing::info!(member_id = id, "회원 지움");
    Ok(StatusCode::NO_CONTENT)
}

/// 한 쪽에 보여 주는 기본 인원.
const DEFAULT_PAGE_SIZE: u32 = 20;
/// 한 쪽에 보여 주는 최대 인원.
const MAX_PAGE_SIZE: u32 = 100;

/// `GET /api/members`의 `?` 뒤 값.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchMembersParams {
    /// 이름, 회원번호, 연락처의 일부. 없으면 모두.
    query: Option<String>,
    /// 1부터. 없으면 1.
    page: Option<u32>,
    /// 1~100. 없으면 20.
    page_size: Option<u32>,
}

/// 목록의 회원 한 명.
///
/// 회원 정보가 그대로 들어 있어 `Debug`를 붙이지 않는다.
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MemberListItem {
    #[ts(type = "number")]
    pub id: i64,
    pub member_no: String,
    pub name: String,
    /// 숫자만. 하이픈은 화면에서 넣는다.
    pub phone_number: String,
    pub birth_date: Option<NaiveDate>,
    pub gender: Option<GenderJson>,
}

impl From<MemberRecord> for MemberListItem {
    fn from(record: MemberRecord) -> Self {
        MemberListItem {
            id: record.id,
            member_no: record.member_no.into_inner(),
            name: record.name.into_inner(),
            phone_number: record.phone_number.into_inner(),
            birth_date: record.birth_date.map(Personal::into_inner),
            gender: record
                .gender
                .map(|gender| GenderJson::from(gender.into_inner())),
        }
    }
}

/// `GET /api/members` 응답.
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MemberListResponse {
    pub items: Vec<MemberListItem>,
    /// 조건에 맞는 전체 인원.
    #[ts(type = "number")]
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
}

/// 검색어로 회원을 찾는다. 이름순, 같으면 회원번호순.
// [docs/conventions.md]: 검색어는 로그에 남기지 않음
pub async fn search_members(
    State(db): State<SqlitePool>,
    params: Result<Query<SearchMembersParams>, QueryRejection>,
) -> Result<Json<MemberListResponse>, ApiError> {
    let Query(params) = params?;
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(DEFAULT_PAGE_SIZE);

    let mut fields = Vec::new();
    if page < 1 {
        fields.push(FieldError {
            field: "page".to_owned(),
            message: "쪽 번호는 1부터입니다".to_owned(),
        });
    }
    if !(1..=MAX_PAGE_SIZE).contains(&page_size) {
        fields.push(FieldError {
            field: "pageSize".to_owned(),
            message: format!("한 쪽 인원은 1~{MAX_PAGE_SIZE}명입니다"),
        });
    }
    if !fields.is_empty() {
        return Err(ApiError::InvalidInput(fields));
    }

    let query = params.query.as_deref().unwrap_or("");
    let found = members::search(&db, query, page, page_size)
        .await
        .map_err(ApiError::internal)?;

    Ok(Json(MemberListResponse {
        items: found.items.into_iter().map(MemberListItem::from).collect(),
        total: found.total,
        page,
        page_size,
    }))
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

impl From<SaveMemberError> for ApiError {
    fn from(error: SaveMemberError) -> Self {
        match error {
            SaveMemberError::DuplicateMemberNo => ApiError::DuplicateMemberNo,
            SaveMemberError::Db(error) => ApiError::internal(error),
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
    use crate::server::version::VERSION_HEADER;
    use crate::server::{AppState, router};

    async fn app() -> (tempfile::TempDir, SqlitePool, Router) {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path()).await.unwrap();
        let router = router(AppState::new(
            crate::version::CURRENT.parse().unwrap(),
            db.clone(),
        ));
        (dir, db, router)
    }

    async fn post(router: Router, body: &str) -> Response<Body> {
        let request = Request::post("/api/members")
            .header(header::CONTENT_TYPE, "application/json")
            .header(VERSION_HEADER, crate::version::CURRENT)
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

    async fn get(router: Router, uri: &str) -> Response<Body> {
        let request = Request::get(uri)
            .header(VERSION_HEADER, crate::version::CURRENT)
            .body(Body::empty())
            .unwrap();
        router.oneshot(request).await.unwrap()
    }

    #[tokio::test]
    async fn searches_members() {
        let (_dir, _db, router) = app().await;
        post(router.clone(), &valid("00001")).await;
        post(router.clone(), &valid("00002")).await;

        let response = get(router, "/api/members?query=0000-0001&pageSize=10").await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        let body = json_body(response).await;
        assert_eq!(body["total"], 2);
        assert_eq!(body["page"], 1);
        assert_eq!(body["pageSize"], 10);
        let first = &body["items"][0];
        assert_eq!(first["memberNo"], "00001");
        assert_eq!(first["name"], "회원1");
        assert_eq!(first["phoneNumber"], "01000000001");
        assert_eq!(first["birthDate"], "1950-03-01");
        assert_eq!(first["gender"], "female");
    }

    #[tokio::test]
    async fn search_without_params_uses_defaults() {
        let (_dir, _db, router) = app().await;

        let response = get(router, "/api/members").await;

        assert_eq!(response.status(), StatusCode::OK);
        let body = json_body(response).await;
        assert_eq!(body["items"], json!([]));
        assert_eq!(body["page"], 1);
        assert_eq!(body["pageSize"], 20);
    }

    #[tokio::test]
    async fn search_rejects_page_out_of_range() {
        let (_dir, _db, router) = app().await;

        let response = get(router, "/api/members?page=0&pageSize=101").await;

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = json_body(response).await;
        assert_eq!(body["fields"][0]["field"], "page");
        assert_eq!(body["fields"][1]["field"], "pageSize");
    }

    #[tokio::test]
    async fn search_with_broken_page_is_malformed() {
        let (_dir, _db, router) = app().await;

        let response = get(router, "/api/members?page=abc").await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn korean_query_in_url() {
        let (_dir, _db, router) = app().await;
        post(router.clone(), &valid("00001")).await;

        // "회원"을 URL 인코딩
        let response = get(router, "/api/members?query=%ED%9A%8C%EC%9B%90").await;

        let body = json_body(response).await;
        assert_eq!(body["total"], 1);
    }

    async fn patch(router: Router, uri: &str, body: &str) -> Response<Body> {
        let request = Request::patch(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .header(VERSION_HEADER, crate::version::CURRENT)
            .body(Body::from(body.to_owned()))
            .unwrap();
        router.oneshot(request).await.unwrap()
    }

    async fn created_id(router: Router, member_no: &str) -> i64 {
        let body = json_body(post(router, &valid(member_no)).await).await;
        body["id"].as_i64().unwrap()
    }

    #[tokio::test]
    async fn gets_one_member() {
        let (_dir, _db, router) = app().await;
        let id = created_id(router.clone(), "00001").await;

        let response = get(router, &format!("/api/members/{id}")).await;

        assert_eq!(response.status(), StatusCode::OK);
        let body = json_body(response).await;
        assert_eq!(body["id"], id);
        assert_eq!(body["memberNo"], "00001");
        assert_eq!(body["gender"], "female");
        let created_at = body["createdAt"].as_str().unwrap();
        assert!(created_at.ends_with('Z'), "{created_at}");
        assert_eq!(created_at.len(), "2026-11-15T01:02:03.000Z".len());
        assert_eq!(body["updatedAt"], body["createdAt"]);
    }

    #[tokio::test]
    async fn unknown_member_is_not_found() {
        let (_dir, _db, router) = app().await;

        let response = get(router, "/api/members/999").await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(json_body(response).await["code"], "MEMBER_NOT_FOUND");
    }

    #[tokio::test]
    async fn non_number_id_is_malformed() {
        let (_dir, _db, router) = app().await;

        let response = get(router, "/api/members/abc").await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn updates_member() {
        let (_dir, _db, router) = app().await;
        let id = created_id(router.clone(), "00001").await;
        let changed = json!({
            "memberNo": "00009",
            "name": "회원2",
            "phoneNumber": "032-000-0000",
            "birthDate": null,
            "gender": "male",
        });

        let response = patch(router, &format!("/api/members/{id}"), &changed.to_string()).await;

        assert_eq!(response.status(), StatusCode::OK);
        let body = json_body(response).await;
        assert_eq!(body["memberNo"], "00009");
        assert_eq!(body["name"], "회원2");
        assert_eq!(body["phoneNumber"], "0320000000");
        assert_eq!(body["birthDate"], Value::Null);
        assert_eq!(body["gender"], "male");
    }

    #[tokio::test]
    async fn update_checks_input_like_create() {
        let (_dir, _db, router) = app().await;
        let id = created_id(router.clone(), "00001").await;
        let changed = json!({
            "memberNo": "00001",
            "name": "",
            "phoneNumber": "01000000001",
            "birthDate": null,
            "gender": null,
        });

        let response = patch(router, &format!("/api/members/{id}"), &changed.to_string()).await;

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(json_body(response).await["fields"][0]["field"], "name");
    }

    #[tokio::test]
    async fn br_m2_update_to_taken_member_no_is_conflict() {
        let (_dir, _db, router) = app().await;
        created_id(router.clone(), "00001").await;
        let id = created_id(router.clone(), "00002").await;

        let response = patch(router, &format!("/api/members/{id}"), &valid("00001")).await;

        assert_eq!(response.status(), StatusCode::CONFLICT);
        assert_eq!(json_body(response).await["rule"], "BR-M2");
    }

    #[tokio::test]
    async fn update_unknown_member_is_not_found() {
        let (_dir, _db, router) = app().await;

        let response = patch(router, "/api/members/999", &valid("00001")).await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    async fn delete(router: Router, uri: &str) -> Response<Body> {
        let request = Request::delete(uri)
            .header(VERSION_HEADER, crate::version::CURRENT)
            .body(Body::empty())
            .unwrap();
        router.oneshot(request).await.unwrap()
    }

    #[tokio::test]
    async fn br_m5_deletes_member() {
        let (_dir, db, router) = app().await;
        let id = created_id(router.clone(), "00001").await;

        let response = delete(router.clone(), &format!("/api/members/{id}")).await;

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert_eq!(count(&db).await, 0);
        let again = get(router, &format!("/api/members/{id}")).await;
        assert_eq!(again.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delete_unknown_member_is_not_found() {
        let (_dir, _db, router) = app().await;

        let response = delete(router, "/api/members/999").await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
