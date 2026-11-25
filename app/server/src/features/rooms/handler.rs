//! 강의실 요청 처리: `/api/rooms`.

use axum::extract::FromRef;
use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use baeum_domain::{Action, NewRoom, RoomError};

use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::dto::{CreateRoomResponse, RoomListResponse, RoomRequest, RoomResponse};
use super::repository::{self, SaveRoomError};
use crate::http::auth::CurrentStaff;
use crate::http::changes::{ChangeKind, ChangeNotices, notifies};
use crate::http::error::ApiError;

/// 강의실 요청 주소. 로그인한 요청만 받는 묶음에 들어가고, 바꾸면 변경 알림을 보낸다 (ADR 0008).
///
/// 상태 `S`는 무엇이든 DB 연결 묶음(`SqlitePool`)을 꺼낼 수 있으면 된다. 그래서 이 기능은 라우터의
/// 상태 타입(`AppState`)을 몰라도 된다.
pub(crate) fn routes<S>(notices: &ChangeNotices) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    SqlitePool: FromRef<S>,
{
    Router::new()
        .route(
            "/api/rooms",
            notifies(get(list_rooms).post(create_room), notices, ChangeKind::Room),
        )
        .route(
            "/api/rooms/{id}",
            notifies(
                get(get_room).patch(update_room).delete(delete_room),
                notices,
                ChangeKind::Room,
            ),
        )
}

/// 강의실을 모두 보여 준다. 건물, 층, 이름 순서.
async fn list_rooms(State(db): State<SqlitePool>) -> Result<Json<RoomListResponse>, ApiError> {
    let records = repository::list(&db).await.map_err(ApiError::internal)?;
    Ok(Json(RoomListResponse {
        items: records.into_iter().map(RoomResponse::from).collect(),
    }))
}

/// `id`인 강의실 하나를 보여 준다.
async fn get_room(
    State(db): State<SqlitePool>,
    id: Result<Path<i64>, PathRejection>,
) -> Result<Json<RoomResponse>, ApiError> {
    let Path(id) = id?;
    let record = repository::get(&db, id)
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::RoomNotFound)?;
    Ok(Json(RoomResponse::from(record)))
}

/// 새 강의실을 저장한다. 성공하면 201과 새 id를 돌려준다.
async fn create_room(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    body: Result<Json<RoomRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    staff.require(Action::EditReferenceData)?;
    let Json(request) = body?;
    let room = NewRoom::parse(request.as_input())
        .map_err(|errors| ApiError::invalid_input(errors, field_name))?;
    let id = repository::insert(&db, &room, OffsetDateTime::now_utc()).await?;
    tracing::info!(staff_id = staff.id, room_id = id, "강의실 저장");

    let location = format!("/api/rooms/{id}");
    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, location)],
        Json(CreateRoomResponse { id }),
    ))
}

/// `id`인 강의실의 칸을 모두 본문 값으로 바꾼다. 확인 규칙은 등록과 같다.
async fn update_room(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
    body: Result<Json<RoomRequest>, JsonRejection>,
) -> Result<Json<RoomResponse>, ApiError> {
    staff.require(Action::EditReferenceData)?;
    let Path(id) = id?;
    let Json(request) = body?;
    let room = NewRoom::parse(request.as_input())
        .map_err(|errors| ApiError::invalid_input(errors, field_name))?;
    let record = repository::update(&db, id, &room, OffsetDateTime::now_utc())
        .await?
        .ok_or(ApiError::RoomNotFound)?;
    tracing::info!(staff_id = staff.id, room_id = id, "강의실 고침");
    Ok(Json(RoomResponse::from(record)))
}

/// `id`인 강의실을 지운다. 성공하면 204(본문 없음). BR-R2: 수업 시간이 쓰면 409.
async fn delete_room(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
) -> Result<StatusCode, ApiError> {
    staff.require(Action::EditReferenceData)?;
    let Path(id) = id?;
    let deleted = repository::delete(&db, id)
        .await
        .map_err(|error| ApiError::from_delete(error, ApiError::RoomInUse))?;
    if !deleted {
        return Err(ApiError::RoomNotFound);
    }
    tracing::info!(staff_id = staff.id, room_id = id, "강의실 지움");
    Ok(StatusCode::NO_CONTENT)
}

/// 오류가 난 칸의 요청 JSON 이름.
fn field_name(error: &RoomError) -> &'static str {
    match error {
        RoomError::EmptyName | RoomError::NameTooLong => "name",
        RoomError::EmptyBuilding | RoomError::BuildingTooLong => "building",
        RoomError::InvalidFloor => "floor",
    }
}

impl From<SaveRoomError> for ApiError {
    fn from(error: SaveRoomError) -> Self {
        match error {
            SaveRoomError::DuplicateName => ApiError::DuplicateRoomName,
            SaveRoomError::Db(error) => ApiError::internal(error),
        }
    }
}
