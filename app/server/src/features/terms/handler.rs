//! `/api/terms`: 학기.

use axum::Json;
use axum::Router;
use axum::extract::FromRef;
use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::get;
use baeum_domain::{Action, NewTerm, TermError};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::dto::{CreateTermResponse, TermListResponse, TermRequest, TermResponse};
use super::repository::{self, SaveTermError};
use crate::features::auth::middleware::CurrentStaff;
use crate::http::changes::{ChangeKind, ChangeNotices, notifies};
use crate::http::error::ApiError;

/// 학기 요청 주소. 로그인한 요청만 받는 묶음에 들어간다 (ADR 0023).
pub(crate) fn routes<S>(notices: &ChangeNotices) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    SqlitePool: FromRef<S>,
{
    Router::new()
        .route(
            "/api/terms",
            notifies(get(list_terms).post(create_term), notices, ChangeKind::Term),
        )
        .route(
            "/api/terms/{id}",
            notifies(
                get(get_term).patch(update_term).delete(delete_term),
                notices,
                ChangeKind::Term,
            ),
        )
}

/// 학기를 모두 보여 준다. 시작일이 늦은 학기부터.
async fn list_terms(State(db): State<SqlitePool>) -> Result<Json<TermListResponse>, ApiError> {
    let records = repository::list(&db).await.map_err(ApiError::internal)?;
    Ok(Json(TermListResponse {
        items: records.into_iter().map(TermResponse::from).collect(),
    }))
}

/// `id`인 학기 하나를 보여 준다.
async fn get_term(
    State(db): State<SqlitePool>,
    id: Result<Path<i64>, PathRejection>,
) -> Result<Json<TermResponse>, ApiError> {
    let Path(id) = id?;
    let record = repository::get(&db, id)
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::TermNotFound)?;
    Ok(Json(TermResponse::from(record)))
}

/// 새 학기를 저장한다. 성공하면 201과 새 id를 돌려준다.
async fn create_term(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    body: Result<Json<TermRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    staff.require(Action::EditCourses)?;
    let Json(request) = body?;
    let term = NewTerm::parse(request.as_input())
        .map_err(|errors| ApiError::invalid_input(errors, field_name))?;
    let id = repository::insert(&db, &term, OffsetDateTime::now_utc()).await?;
    tracing::info!(staff_id = staff.id, term_id = id, "학기 저장");

    let location = format!("/api/terms/{id}");
    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, location)],
        Json(CreateTermResponse { id }),
    ))
}

/// `id`인 학기의 칸을 모두 본문 값으로 바꾼다. 확인 규칙은 등록과 같다.
async fn update_term(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
    body: Result<Json<TermRequest>, JsonRejection>,
) -> Result<Json<TermResponse>, ApiError> {
    staff.require(Action::EditCourses)?;
    let Path(id) = id?;
    let Json(request) = body?;
    let term = NewTerm::parse(request.as_input())
        .map_err(|errors| ApiError::invalid_input(errors, field_name))?;
    let record = repository::update(&db, id, &term, OffsetDateTime::now_utc())
        .await?
        .ok_or(ApiError::TermNotFound)?;
    tracing::info!(staff_id = staff.id, term_id = id, "학기 고침");
    Ok(Json(TermResponse::from(record)))
}

/// `id`인 학기를 지운다. 성공하면 204(본문 없음). BR-T5: 강좌가 있으면 409.
async fn delete_term(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
) -> Result<StatusCode, ApiError> {
    staff.require(Action::EditCourses)?;
    let Path(id) = id?;
    let deleted = repository::delete(&db, id)
        .await
        .map_err(|error| ApiError::from_delete(error, ApiError::TermInUse))?;
    if !deleted {
        return Err(ApiError::TermNotFound);
    }
    tracing::info!(staff_id = staff.id, term_id = id, "학기 지움");
    Ok(StatusCode::NO_CONTENT)
}

/// 오류가 난 칸의 요청 JSON 이름.
fn field_name(error: &TermError) -> &'static str {
    match error {
        TermError::EmptyName | TermError::NameTooLong => "name",
        TermError::MissingStart => "startsOn",
        TermError::MissingEnd | TermError::EndBeforeStart => "endsOn",
    }
}

impl From<SaveTermError> for ApiError {
    fn from(error: SaveTermError) -> Self {
        match error {
            SaveTermError::DuplicateName => ApiError::DuplicateTermName,
            SaveTermError::Db(error) => ApiError::internal(error),
        }
    }
}
