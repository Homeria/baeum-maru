//! `/api/members`: 회원.

use axum::Json;
use axum::Router;
use axum::extract::FromRef;
use axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection};
use axum::extract::{Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::get;
use baeum_domain::{Action, MemberError, NewMember};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::dto::{
    CreateMemberResponse, MemberListItem, MemberListResponse, MemberRequest, MemberResponse,
    SearchMembersParams,
};
use super::error::SaveMemberError;
use super::repository;
use crate::features::auth::middleware::CurrentStaff;
use crate::http::changes::{ChangeKind, ChangeNotices, notifies};
use crate::http::error::ApiError;
use crate::http::paging::Paging;
use crate::infra::clock::local_date;

/// 한 쪽에 보여 주는 기본 인원.
const DEFAULT_PAGE_SIZE: u32 = 20;

/// 검색어로 회원을 찾는다. 이름순, 같으면 회원번호순.
// [docs/conventions.md]: 검색어는 로그에 남기지 않음
/// 회원 요청 주소. 로그인한 요청만 받는 묶음에 들어간다 (ADR 0023).
pub(crate) fn routes<S>(notices: &ChangeNotices) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    SqlitePool: FromRef<S>,
{
    Router::new()
        .route(
            "/api/members",
            notifies(
                get(search_members).post(create_member),
                notices,
                ChangeKind::Member,
            ),
        )
        .route(
            "/api/members/{id}",
            notifies(
                get(get_member).patch(update_member).delete(delete_member),
                notices,
                ChangeKind::Member,
            ),
        )
}

async fn search_members(
    State(db): State<SqlitePool>,
    params: Result<Query<SearchMembersParams>, QueryRejection>,
) -> Result<Json<MemberListResponse>, ApiError> {
    let Query(params) = params?;
    let Paging { page, page_size } =
        Paging::parse(params.page, params.page_size, DEFAULT_PAGE_SIZE)?;

    let query = params.query.as_deref().unwrap_or("");
    let found = repository::search(&db, query, page, page_size)
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
async fn get_member(
    State(db): State<SqlitePool>,
    id: Result<Path<i64>, PathRejection>,
) -> Result<Json<MemberResponse>, ApiError> {
    let Path(id) = id?;
    let record = repository::get(&db, id)
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::MemberNotFound)?;
    Ok(Json(MemberResponse::from(record)))
}

/// 새 회원을 저장한다. 성공하면 201과 새 id를 돌려준다.
async fn create_member(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    body: Result<Json<MemberRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    staff.require(Action::EditMembers)?;
    let Json(request) = body?;
    let now = OffsetDateTime::now_utc();
    // 생년월일은 이 PC의 날짜로 비교 (한국에서 쓰므로 한국 날짜)
    let today = local_date(now);

    let member = NewMember::parse(request.as_input(), today)
        .map_err(|errors| ApiError::invalid_input(errors, field_name))?;
    let id = repository::insert(&db, &member, now).await?;
    tracing::info!(staff_id = staff.id, member_id = id, "회원 저장");

    let location = format!("/api/members/{id}");
    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, location)],
        Json(CreateMemberResponse { id }),
    ))
}

/// `id`인 회원의 칸을 모두 본문 값으로 바꾼다. 확인 규칙은 등록과 같다.
async fn update_member(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
    body: Result<Json<MemberRequest>, JsonRejection>,
) -> Result<Json<MemberResponse>, ApiError> {
    staff.require(Action::EditMembers)?;
    let Path(id) = id?;
    let Json(request) = body?;
    let now = OffsetDateTime::now_utc();
    let today = local_date(now);

    let member = NewMember::parse(request.as_input(), today)
        .map_err(|errors| ApiError::invalid_input(errors, field_name))?;
    let record = repository::update(&db, id, &member, now)
        .await?
        .ok_or(ApiError::MemberNotFound)?;
    tracing::info!(staff_id = staff.id, member_id = id, "회원 고침");
    Ok(Json(MemberResponse::from(record)))
}

/// `id`인 회원을 지운다. 성공하면 204(본문 없음).
///
/// BR-M5: 신청 기록이 있는 회원은 신청 기능에서 409로 막는다.
async fn delete_member(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
) -> Result<StatusCode, ApiError> {
    staff.require(Action::DeleteMembers)?;
    let Path(id) = id?;
    let deleted = repository::delete(&db, id)
        .await
        .map_err(|error| ApiError::from_delete(error, ApiError::MemberInUse))?;
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
