//! `/api/rooms`: 강의실.

mod json;

use axum::Json;
use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use baeum_domain::{Action, NewRoom, RoomError};

use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::auth::CurrentStaff;
use super::error::ApiError;
use crate::db::rooms::{self, SaveRoomError};

use json::{CreateRoomResponse, RoomListResponse, RoomRequest, RoomResponse};

/// 강의실을 모두 보여 준다. 건물, 층, 이름 순서.
pub async fn list_rooms(State(db): State<SqlitePool>) -> Result<Json<RoomListResponse>, ApiError> {
    let records = rooms::list(&db).await.map_err(ApiError::internal)?;
    Ok(Json(RoomListResponse {
        items: records.into_iter().map(RoomResponse::from).collect(),
    }))
}

/// `id`인 강의실 하나를 보여 준다.
pub async fn get_room(
    State(db): State<SqlitePool>,
    id: Result<Path<i64>, PathRejection>,
) -> Result<Json<RoomResponse>, ApiError> {
    let Path(id) = id?;
    let record = rooms::get(&db, id)
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::RoomNotFound)?;
    Ok(Json(RoomResponse::from(record)))
}

/// 새 강의실을 저장한다. 성공하면 201과 새 id를 돌려준다.
pub async fn create_room(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    body: Result<Json<RoomRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    staff.require(Action::EditReferenceData)?;
    let Json(request) = body?;
    let room = NewRoom::parse(request.as_input())
        .map_err(|errors| ApiError::invalid_input(errors, field_name))?;
    let id = rooms::insert(&db, &room, OffsetDateTime::now_utc()).await?;
    tracing::info!(staff_id = staff.id, room_id = id, "강의실 저장");

    let location = format!("/api/rooms/{id}");
    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, location)],
        Json(CreateRoomResponse { id }),
    ))
}

/// `id`인 강의실의 칸을 모두 본문 값으로 바꾼다. 확인 규칙은 등록과 같다.
pub async fn update_room(
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
    let record = rooms::update(&db, id, &room, OffsetDateTime::now_utc())
        .await?
        .ok_or(ApiError::RoomNotFound)?;
    tracing::info!(staff_id = staff.id, room_id = id, "강의실 고침");
    Ok(Json(RoomResponse::from(record)))
}

/// `id`인 강의실을 지운다. 성공하면 204(본문 없음). BR-R2: 수업 시간이 쓰면 409.
pub async fn delete_room(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
) -> Result<StatusCode, ApiError> {
    staff.require(Action::EditReferenceData)?;
    let Path(id) = id?;
    let deleted = rooms::delete(&db, id)
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

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::Request;
    use serde_json::json;
    use tower::ServiceExt;

    use super::*;
    use crate::server::test_support::{
        app, create, create_course, create_room, create_subject, create_term, delete, get,
        json_body, patch, post,
    };

    fn room(building: &str, floor: Option<i32>, name: &str) -> String {
        json!({ "building": building, "floor": floor, "name": name }).to_string()
    }

    #[tokio::test]
    async fn creates_and_lists_room() {
        let (_dir, _db, router) = app().await;

        let response = post(
            router.clone(),
            "/api/rooms",
            &room("본관", Some(-1), "체력단련실"),
        )
        .await;

        assert_eq!(response.status(), StatusCode::CREATED);
        let id = json_body(response).await["id"].clone();
        let list = json_body(get(router, "/api/rooms").await).await;
        assert_eq!(
            list,
            json!({ "items": [
                { "id": id, "building": "본관", "floor": -1, "name": "체력단련실" }
            ]})
        );
    }

    #[tokio::test]
    async fn invalid_input_returns_every_field() {
        let (_dir, _db, router) = app().await;

        let response = post(router, "/api/rooms", &room(" ", Some(0), "")).await;

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = json_body(response).await;
        let fields: Vec<&str> = body["fields"]
            .as_array()
            .unwrap()
            .iter()
            .map(|field| field["field"].as_str().unwrap())
            .collect();
        assert_eq!(fields, ["name", "building", "floor"]);
    }

    #[tokio::test]
    async fn br_r1_duplicate_name_is_conflict() {
        let (_dir, _db, router) = app().await;
        post(
            router.clone(),
            "/api/rooms",
            &room("본관", Some(3), "다목적실"),
        )
        .await;

        let response = post(router, "/api/rooms", &room("본관", None, "다목적실")).await;

        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = json_body(response).await;
        assert_eq!(body["code"], "DUPLICATE_ROOM_NAME");
        assert_eq!(body["rule"], "BR-R1");
    }

    #[tokio::test]
    async fn empty_list_is_empty_items() {
        let (_dir, _db, router) = app().await;

        let body = json_body(get(router, "/api/rooms").await).await;

        assert_eq!(body, json!({ "items": [] }));
    }

    #[tokio::test]
    async fn rooms_need_matching_version() {
        let (_dir, _db, router) = app().await;
        let request = Request::get("/api/rooms").body(Body::empty()).unwrap();

        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn gets_one_room() {
        let (_dir, _db, router) = app().await;
        let id = create(
            router.clone(),
            "/api/rooms",
            &room("본관", Some(3), "다목적실"),
        )
        .await;

        let response = get(router, &format!("/api/rooms/{id}")).await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            json_body(response).await,
            json!({ "id": id, "building": "본관", "floor": 3, "name": "다목적실" })
        );
    }

    #[tokio::test]
    async fn unknown_room_is_not_found() {
        let (_dir, _db, router) = app().await;

        let response = get(router, "/api/rooms/999").await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(json_body(response).await["code"], "ROOM_NOT_FOUND");
    }

    #[tokio::test]
    async fn non_number_id_is_malformed() {
        let (_dir, _db, router) = app().await;

        let response = get(router, "/api/rooms/abc").await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(json_body(response).await["code"], "MALFORMED_REQUEST");
    }

    #[tokio::test]
    async fn updates_room() {
        let (_dir, _db, router) = app().await;
        let id = create(
            router.clone(),
            "/api/rooms",
            &room("본관", Some(3), "다목적실"),
        )
        .await;

        let uri = format!("/api/rooms/{id}");
        let response = patch(router, &uri, &room("별관", None, "요가실")).await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            json_body(response).await,
            json!({ "id": id, "building": "별관", "floor": null, "name": "요가실" })
        );
    }

    #[tokio::test]
    async fn update_checks_input_like_create() {
        let (_dir, _db, router) = app().await;
        let id = create(
            router.clone(),
            "/api/rooms",
            &room("본관", Some(3), "다목적실"),
        )
        .await;

        let uri = format!("/api/rooms/{id}");
        let response = patch(router, &uri, &room("본관", Some(6), "다목적실")).await;

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(json_body(response).await["fields"][0]["field"], "floor");
    }

    #[tokio::test]
    async fn br_r1_update_to_taken_name_is_conflict() {
        let (_dir, _db, router) = app().await;
        create(
            router.clone(),
            "/api/rooms",
            &room("본관", None, "다목적실"),
        )
        .await;
        let id = create(router.clone(), "/api/rooms", &room("본관", None, "요가실")).await;

        let uri = format!("/api/rooms/{id}");
        let response = patch(router, &uri, &room("본관", None, "다목적실")).await;

        assert_eq!(response.status(), StatusCode::CONFLICT);
        assert_eq!(json_body(response).await["rule"], "BR-R1");
    }

    #[tokio::test]
    async fn deletes_room() {
        let (_dir, _db, router) = app().await;
        let id = create(
            router.clone(),
            "/api/rooms",
            &room("본관", None, "다목적실"),
        )
        .await;
        let uri = format!("/api/rooms/{id}");

        let response = delete(router.clone(), &uri).await;

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let again = delete(router, &uri).await;
        assert_eq!(again.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn br_r2_room_used_by_schedule_is_conflict() {
        let (_dir, _db, router) = app().await;
        let room_id = create_room(router.clone()).await;
        let term_id = create_term(router.clone()).await;
        let subject_id = create_subject(router.clone(), "요가").await;
        create_course(router.clone(), term_id, subject_id, Some(room_id)).await;

        let response = delete(router, &format!("/api/rooms/{room_id}")).await;

        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = json_body(response).await;
        assert_eq!(body["code"], "ROOM_IN_USE");
        assert_eq!(body["rule"], "BR-R2");
    }
}
