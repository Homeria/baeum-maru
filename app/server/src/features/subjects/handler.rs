//! `/api/subjects`: 과목.

use axum::Json;
use axum::Router;
use axum::extract::FromRef;
use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::get;
use baeum_domain::{Action, NewSubject, SubjectError};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::dto::{CreateSubjectResponse, SubjectListResponse, SubjectRequest, SubjectResponse};
use super::repository::{self, SaveSubjectError};
use crate::http::auth::CurrentStaff;
use crate::http::changes::{ChangeKind, ChangeNotices, notifies};
use crate::http::error::ApiError;

/// 과목 요청 주소. 로그인한 요청만 받는 묶음에 들어간다 (ADR 0023).
pub(crate) fn routes<S>(notices: &ChangeNotices) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    SqlitePool: FromRef<S>,
{
    Router::new()
        .route(
            "/api/subjects",
            notifies(
                get(list_subjects).post(create_subject),
                notices,
                ChangeKind::Subject,
            ),
        )
        .route(
            "/api/subjects/{id}",
            notifies(
                get(get_subject)
                    .patch(update_subject)
                    .delete(delete_subject),
                notices,
                ChangeKind::Subject,
            ),
        )
}

/// 과목을 모두 보여 준다. 분류, 이름 순서.
async fn list_subjects(
    State(db): State<SqlitePool>,
) -> Result<Json<SubjectListResponse>, ApiError> {
    let records = repository::list(&db).await.map_err(ApiError::internal)?;
    Ok(Json(SubjectListResponse {
        items: records.into_iter().map(SubjectResponse::from).collect(),
    }))
}

/// `id`인 과목 하나를 보여 준다.
async fn get_subject(
    State(db): State<SqlitePool>,
    id: Result<Path<i64>, PathRejection>,
) -> Result<Json<SubjectResponse>, ApiError> {
    let Path(id) = id?;
    let record = repository::get(&db, id)
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::SubjectNotFound)?;
    Ok(Json(SubjectResponse::from(record)))
}

/// 새 과목을 저장한다. 성공하면 201과 새 id를 돌려준다.
async fn create_subject(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    body: Result<Json<SubjectRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    staff.require(Action::EditReferenceData)?;
    let Json(request) = body?;
    let subject = NewSubject::parse(request.as_input())
        .map_err(|errors| ApiError::invalid_input(errors, field_name))?;
    let id = repository::insert(&db, &subject, OffsetDateTime::now_utc()).await?;
    tracing::info!(staff_id = staff.id, subject_id = id, "과목 저장");

    let location = format!("/api/subjects/{id}");
    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, location)],
        Json(CreateSubjectResponse { id }),
    ))
}

/// `id`인 과목의 칸을 모두 본문 값으로 바꾼다. 확인 규칙은 등록과 같다.
async fn update_subject(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
    body: Result<Json<SubjectRequest>, JsonRejection>,
) -> Result<Json<SubjectResponse>, ApiError> {
    staff.require(Action::EditReferenceData)?;
    let Path(id) = id?;
    let Json(request) = body?;
    let subject = NewSubject::parse(request.as_input())
        .map_err(|errors| ApiError::invalid_input(errors, field_name))?;
    let record = repository::update(&db, id, &subject, OffsetDateTime::now_utc())
        .await?
        .ok_or(ApiError::SubjectNotFound)?;
    tracing::info!(staff_id = staff.id, subject_id = id, "과목 고침");
    Ok(Json(SubjectResponse::from(record)))
}

/// `id`인 과목을 지운다. 성공하면 204(본문 없음). BR-J2: 강좌가 쓰면 409.
async fn delete_subject(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
) -> Result<StatusCode, ApiError> {
    staff.require(Action::EditReferenceData)?;
    let Path(id) = id?;
    let deleted = repository::delete(&db, id)
        .await
        .map_err(|error| ApiError::from_delete(error, ApiError::SubjectInUse))?;
    if !deleted {
        return Err(ApiError::SubjectNotFound);
    }
    tracing::info!(staff_id = staff.id, subject_id = id, "과목 지움");
    Ok(StatusCode::NO_CONTENT)
}

/// 오류가 난 칸의 요청 JSON 이름.
fn field_name(error: &SubjectError) -> &'static str {
    match error {
        SubjectError::EmptyName | SubjectError::NameTooLong => "name",
        SubjectError::CategoryTooLong => "category",
    }
}

impl From<SaveSubjectError> for ApiError {
    fn from(error: SaveSubjectError) -> Self {
        match error {
            SaveSubjectError::DuplicateName => ApiError::DuplicateSubjectName,
            SaveSubjectError::Db(error) => ApiError::internal(error),
        }
    }
}
