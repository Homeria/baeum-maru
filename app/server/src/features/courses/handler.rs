//! 강좌: 목록·등록은 `/api/terms/{termId}/courses`, 보기·고치기·지우기는 `/api/courses/{id}`.

use axum::Json;
use axum::Router;
use axum::extract::FromRef;
use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::get;
use baeum_domain::{Action, CourseError, NewCourse};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::dto::{CourseListResponse, CourseRequest, CourseResponse, CreateCourseResponse};
use super::repository::{self, MissingReference, SaveCourseError};
use crate::features::terms::repository as terms;
use crate::http::auth::CurrentStaff;
use crate::http::changes::{ChangeKind, ChangeNotices, notifies};
use crate::http::error::ApiError;

/// 강좌 요청 주소. 로그인한 요청만 받는 묶음에 들어간다 (ADR 0023).
pub(crate) fn routes<S>(notices: &ChangeNotices) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    SqlitePool: FromRef<S>,
{
    Router::new()
        .route(
            "/api/terms/{term_id}/courses",
            notifies(
                get(list_courses).post(create_course),
                notices,
                ChangeKind::Course,
            ),
        )
        .route(
            "/api/courses/{id}",
            notifies(
                get(get_course).patch(update_course).delete(delete_course),
                notices,
                ChangeKind::Course,
            ),
        )
}

/// 학기의 강좌를 모두 보여 준다. 과목 이름, 등록 순서. 학기가 없으면 404.
async fn list_courses(
    State(db): State<SqlitePool>,
    term_id: Result<Path<i64>, PathRejection>,
) -> Result<Json<CourseListResponse>, ApiError> {
    let Path(term_id) = term_id?;
    terms::get(&db, term_id)
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::TermNotFound)?;
    let records = repository::list(&db, term_id)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(CourseListResponse {
        items: records.into_iter().map(CourseResponse::from).collect(),
    }))
}

/// `id`인 강좌 하나를 보여 준다.
async fn get_course(
    State(db): State<SqlitePool>,
    id: Result<Path<i64>, PathRejection>,
) -> Result<Json<CourseResponse>, ApiError> {
    let Path(id) = id?;
    let record = repository::get(&db, id)
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::CourseNotFound)?;
    Ok(Json(CourseResponse::from(record)))
}

/// 학기에 새 강좌를 저장한다. 성공하면 201과 새 id를 돌려준다.
async fn create_course(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    term_id: Result<Path<i64>, PathRejection>,
    body: Result<Json<CourseRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    staff.require(Action::EditCourses)?;
    let Path(term_id) = term_id?;
    let Json(request) = body?;
    let course = parse(&request)?;
    let id = repository::insert(&db, term_id, &course, OffsetDateTime::now_utc()).await?;
    tracing::info!(staff_id = staff.id, term_id, course_id = id, "강좌 저장");

    let location = format!("/api/courses/{id}");
    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, location)],
        Json(CreateCourseResponse { id }),
    ))
}

/// `id`인 강좌의 칸과 수업 시간을 모두 본문 값으로 바꾼다. 확인 규칙은 등록과 같다.
async fn update_course(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
    body: Result<Json<CourseRequest>, JsonRejection>,
) -> Result<Json<CourseResponse>, ApiError> {
    staff.require(Action::EditCourses)?;
    let Path(id) = id?;
    let Json(request) = body?;
    let course = parse(&request)?;
    let record = repository::update(&db, id, &course, OffsetDateTime::now_utc())
        .await?
        .ok_or(ApiError::CourseNotFound)?;
    tracing::info!(staff_id = staff.id, course_id = id, "강좌 고침");
    Ok(Json(CourseResponse::from(record)))
}

/// `id`인 강좌와 그 수업 시간을 지운다. 성공하면 204(본문 없음).
async fn delete_course(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
) -> Result<StatusCode, ApiError> {
    staff.require(Action::EditCourses)?;
    let Path(id) = id?;
    let deleted = repository::delete(&db, id)
        .await
        .map_err(|error| ApiError::from_delete(error, ApiError::CourseInUse))?;
    if !deleted {
        return Err(ApiError::CourseNotFound);
    }
    tracing::info!(staff_id = staff.id, course_id = id, "강좌 지움");
    Ok(StatusCode::NO_CONTENT)
}

/// 본문을 업무 규칙으로 확인한다. 요일 번호가 틀리면 400, 칸이 틀리면 422.
fn parse(request: &CourseRequest) -> Result<NewCourse, ApiError> {
    let input = request.as_input().ok_or(ApiError::MalformedRequest)?;
    NewCourse::parse(input).map_err(|errors| ApiError::invalid_input(errors, field_name))
}

/// 오류가 난 칸의 요청 JSON 이름. 수업 시간 줄의 오류는 모두 `schedules`이고 문장에 줄 번호가 있다.
fn field_name(error: &CourseError) -> &'static str {
    match error {
        CourseError::MissingSubject => "subjectId",
        CourseError::LevelTooLong => "level",
        CourseError::SectionTooLong => "section",
        CourseError::MissingCapacity | CourseError::CapacityOutOfRange => "capacity",
        CourseError::NoSchedule
        | CourseError::InvalidTime { .. }
        | CourseError::EndNotAfterStart { .. }
        | CourseError::SchedulesOverlap { .. } => "schedules",
    }
}

/// 없는 과목·강의실이 어느 칸의 것인지. [`field_name`]과 같은 이름을 쓴다.
fn reference_field_name(missing: &MissingReference) -> &'static str {
    match missing {
        MissingReference::Subject => "subjectId",
        MissingReference::Room { .. } => "schedules",
    }
}

impl From<SaveCourseError> for ApiError {
    fn from(error: SaveCourseError) -> Self {
        match error {
            SaveCourseError::TermNotFound => ApiError::TermNotFound,
            SaveCourseError::MissingReferences(missing) => {
                ApiError::invalid_input(missing, reference_field_name)
            }
            SaveCourseError::Duplicate => ApiError::DuplicateCourse,
            SaveCourseError::Db(error) => ApiError::internal(error),
        }
    }
}
