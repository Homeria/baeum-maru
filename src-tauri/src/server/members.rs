//! `/api/members`: 회원.

mod json;

use axum::Json;
use axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection};
use axum::extract::{Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use baeum_domain::{Action, MemberError, NewMember};
use chrono::{Local, Utc};
use sqlx::SqlitePool;

use super::auth::CurrentStaff;
use super::error::ApiError;
use super::paging::Paging;
use crate::db::members::{self, SaveMemberError};

use json::{
    CreateMemberResponse, MemberListItem, MemberListResponse, MemberRequest, MemberResponse,
    SearchMembersParams,
};

/// 한 쪽에 보여 주는 기본 인원.
const DEFAULT_PAGE_SIZE: u32 = 20;

/// 검색어로 회원을 찾는다. 이름순, 같으면 회원번호순.
// [docs/conventions.md]: 검색어는 로그에 남기지 않음
pub async fn search_members(
    State(db): State<SqlitePool>,
    params: Result<Query<SearchMembersParams>, QueryRejection>,
) -> Result<Json<MemberListResponse>, ApiError> {
    let Query(params) = params?;
    let Paging { page, page_size } =
        Paging::parse(params.page, params.page_size, DEFAULT_PAGE_SIZE)?;

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

/// 새 회원을 저장한다. 성공하면 201과 새 id를 돌려준다.
pub async fn create_member(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    body: Result<Json<MemberRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    staff.require(Action::EditMembers)?;
    let Json(request) = body?;
    let now = Utc::now();
    // 생년월일은 이 PC의 날짜로 비교 (한국에서 쓰므로 한국 날짜)
    let today = now.with_timezone(&Local).date_naive();

    let member = NewMember::parse(request.as_input(), today)
        .map_err(|errors| ApiError::invalid_input(errors, field_name))?;
    let id = members::insert(&db, &member, now).await?;
    tracing::info!(staff_id = staff.id, member_id = id, "회원 저장");

    let location = format!("/api/members/{id}");
    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, location)],
        Json(CreateMemberResponse { id }),
    ))
}

/// `id`인 회원의 칸을 모두 본문 값으로 바꾼다. 확인 규칙은 등록과 같다.
pub async fn update_member(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
    body: Result<Json<MemberRequest>, JsonRejection>,
) -> Result<Json<MemberResponse>, ApiError> {
    staff.require(Action::EditMembers)?;
    let Path(id) = id?;
    let Json(request) = body?;
    let now = Utc::now();
    let today = now.with_timezone(&Local).date_naive();

    let member = NewMember::parse(request.as_input(), today)
        .map_err(|errors| ApiError::invalid_input(errors, field_name))?;
    let record = members::update(&db, id, &member, now)
        .await?
        .ok_or(ApiError::MemberNotFound)?;
    tracing::info!(staff_id = staff.id, member_id = id, "회원 고침");
    Ok(Json(MemberResponse::from(record)))
}

/// `id`인 회원을 지운다. 성공하면 204(본문 없음).
///
/// BR-M5: 신청 기록이 있는 회원은 신청 기능에서 409로 막는다.
pub async fn delete_member(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
) -> Result<StatusCode, ApiError> {
    staff.require(Action::DeleteMembers)?;
    let Path(id) = id?;
    let deleted = members::delete(&db, id).await.map_err(ApiError::internal)?;
    if !deleted {
        return Err(ApiError::MemberNotFound);
    }
    tracing::info!(staff_id = staff.id, member_id = id, "회원 지움");
    Ok(StatusCode::NO_CONTENT)
}

/// 오류가 난 칸의 요청 JSON 이름.
fn field_name(error: &MemberError) -> &'static str {
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
    use serde_json::{Value, json};

    use super::*;
    use crate::server::test_support::{app, create, delete, get, json_body, patch, post};

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

        let response = post(router, "/api/members", &valid("00001")).await;

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

        let response = post(router, "/api/members", &body.to_string()).await;

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

        let response = post(router, "/api/members", &body.to_string()).await;

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
        post(router.clone(), "/api/members", &valid("00001")).await;

        let response = post(router, "/api/members", &valid("00001")).await;

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

        let response = post(router, "/api/members", &body.to_string()).await;

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

        let response = post(router, "/api/members", &body.to_string()).await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn searches_members() {
        let (_dir, _db, router) = app().await;
        post(router.clone(), "/api/members", &valid("00001")).await;
        post(router.clone(), "/api/members", &valid("00002")).await;

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
        post(router.clone(), "/api/members", &valid("00001")).await;

        // "회원"을 URL 인코딩
        let response = get(router, "/api/members?query=%ED%9A%8C%EC%9B%90").await;

        let body = json_body(response).await;
        assert_eq!(body["total"], 1);
    }

    async fn created_id(router: Router, member_no: &str) -> i64 {
        create(router, "/api/members", &valid(member_no)).await
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
