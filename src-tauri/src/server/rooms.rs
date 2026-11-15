//! `/api/rooms`: 강의실.

mod json;

use axum::Json;
use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use baeum_domain::{NewRoom, RoomError};
use chrono::Utc;
use sqlx::SqlitePool;

use super::error::{ApiError, FieldError};
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
    body: Result<Json<RoomRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let Json(request) = body?;
    let room = NewRoom::parse(request.as_input()).map_err(invalid_input)?;
    let id = rooms::insert(&db, &room, Utc::now()).await?;
    tracing::info!(room_id = id, "강의실 저장");

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
    id: Result<Path<i64>, PathRejection>,
    body: Result<Json<RoomRequest>, JsonRejection>,
) -> Result<Json<RoomResponse>, ApiError> {
    let Path(id) = id?;
    let Json(request) = body?;
    let room = NewRoom::parse(request.as_input()).map_err(invalid_input)?;
    let record = rooms::update(&db, id, &room, Utc::now())
        .await?
        .ok_or(ApiError::RoomNotFound)?;
    tracing::info!(room_id = id, "강의실 고침");
    Ok(Json(RoomResponse::from(record)))
}

/// `id`인 강의실을 지운다. 성공하면 204(본문 없음).
pub async fn delete_room(
    State(db): State<SqlitePool>,
    id: Result<Path<i64>, PathRejection>,
) -> Result<StatusCode, ApiError> {
    let Path(id) = id?;
    let deleted = rooms::delete(&db, id).await.map_err(ApiError::internal)?;
    if !deleted {
        return Err(ApiError::RoomNotFound);
    }
    tracing::info!(room_id = id, "강의실 지움");
    Ok(StatusCode::NO_CONTENT)
}

fn invalid_input(errors: Vec<RoomError>) -> ApiError {
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
fn field_name(error: RoomError) -> &'static str {
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
    use axum::Router;
    use axum::body::Body;
    use axum::http::{Request, Response};
    use serde_json::{Value, json};
    use tower::ServiceExt;

    use super::*;
    use crate::server::version::VERSION_HEADER;
    use crate::server::{AppState, router};

    async fn app() -> (tempfile::TempDir, Router) {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path()).await.unwrap();
        let router = router(AppState::new(crate::version::CURRENT.parse().unwrap(), db));
        (dir, router)
    }

    async fn post(router: Router, body: Value) -> Response<Body> {
        let request = Request::post("/api/rooms")
            .header(header::CONTENT_TYPE, "application/json")
            .header(VERSION_HEADER, crate::version::CURRENT)
            .body(Body::from(body.to_string()))
            .unwrap();
        router.oneshot(request).await.unwrap()
    }

    async fn get(router: Router, uri: &str) -> Response<Body> {
        let request = Request::get(uri)
            .header(VERSION_HEADER, crate::version::CURRENT)
            .body(Body::empty())
            .unwrap();
        router.oneshot(request).await.unwrap()
    }

    async fn json_body(response: Response<Body>) -> Value {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    fn room(building: &str, floor: Option<i32>, name: &str) -> Value {
        json!({ "building": building, "floor": floor, "name": name })
    }

    #[tokio::test]
    async fn creates_and_lists_room() {
        let (_dir, router) = app().await;

        let response = post(router.clone(), room("본관", Some(-1), "체력단련실")).await;

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
        let (_dir, router) = app().await;

        let response = post(router, room(" ", Some(0), "")).await;

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
        let (_dir, router) = app().await;
        post(router.clone(), room("본관", Some(3), "다목적실")).await;

        let response = post(router, room("본관", None, "다목적실")).await;

        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = json_body(response).await;
        assert_eq!(body["code"], "DUPLICATE_ROOM_NAME");
        assert_eq!(body["rule"], "BR-R1");
    }

    #[tokio::test]
    async fn empty_list_is_empty_items() {
        let (_dir, router) = app().await;

        let body = json_body(get(router, "/api/rooms").await).await;

        assert_eq!(body, json!({ "items": [] }));
    }

    #[tokio::test]
    async fn rooms_need_matching_version() {
        let (_dir, router) = app().await;
        let request = Request::get("/api/rooms").body(Body::empty()).unwrap();

        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    async fn send(router: Router, method: &str, uri: &str, body: Option<Value>) -> Response<Body> {
        let mut request = Request::builder()
            .method(method)
            .uri(uri)
            .header(VERSION_HEADER, crate::version::CURRENT);
        let body = match body {
            Some(body) => {
                request = request.header(header::CONTENT_TYPE, "application/json");
                Body::from(body.to_string())
            }
            None => Body::empty(),
        };
        router.oneshot(request.body(body).unwrap()).await.unwrap()
    }

    async fn created_id(router: Router, body: Value) -> i64 {
        json_body(post(router, body).await).await["id"]
            .as_i64()
            .unwrap()
    }

    #[tokio::test]
    async fn gets_one_room() {
        let (_dir, router) = app().await;
        let id = created_id(router.clone(), room("본관", Some(3), "다목적실")).await;

        let response = get(router, &format!("/api/rooms/{id}")).await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            json_body(response).await,
            json!({ "id": id, "building": "본관", "floor": 3, "name": "다목적실" })
        );
    }

    #[tokio::test]
    async fn unknown_room_is_not_found() {
        let (_dir, router) = app().await;

        let response = get(router, "/api/rooms/999").await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(json_body(response).await["code"], "ROOM_NOT_FOUND");
    }

    #[tokio::test]
    async fn non_number_id_is_malformed() {
        let (_dir, router) = app().await;

        let response = get(router, "/api/rooms/abc").await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(json_body(response).await["code"], "MALFORMED_REQUEST");
    }

    #[tokio::test]
    async fn updates_room() {
        let (_dir, router) = app().await;
        let id = created_id(router.clone(), room("본관", Some(3), "다목적실")).await;

        let uri = format!("/api/rooms/{id}");
        let response = send(router, "PATCH", &uri, Some(room("별관", None, "요가실"))).await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            json_body(response).await,
            json!({ "id": id, "building": "별관", "floor": null, "name": "요가실" })
        );
    }

    #[tokio::test]
    async fn update_checks_input_like_create() {
        let (_dir, router) = app().await;
        let id = created_id(router.clone(), room("본관", Some(3), "다목적실")).await;

        let uri = format!("/api/rooms/{id}");
        let response = send(
            router,
            "PATCH",
            &uri,
            Some(room("본관", Some(6), "다목적실")),
        )
        .await;

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(json_body(response).await["fields"][0]["field"], "floor");
    }

    #[tokio::test]
    async fn br_r1_update_to_taken_name_is_conflict() {
        let (_dir, router) = app().await;
        created_id(router.clone(), room("본관", None, "다목적실")).await;
        let id = created_id(router.clone(), room("본관", None, "요가실")).await;

        let uri = format!("/api/rooms/{id}");
        let response = send(router, "PATCH", &uri, Some(room("본관", None, "다목적실"))).await;

        assert_eq!(response.status(), StatusCode::CONFLICT);
        assert_eq!(json_body(response).await["rule"], "BR-R1");
    }

    #[tokio::test]
    async fn deletes_room() {
        let (_dir, router) = app().await;
        let id = created_id(router.clone(), room("본관", None, "다목적실")).await;
        let uri = format!("/api/rooms/{id}");

        let response = send(router.clone(), "DELETE", &uri, None).await;

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let again = send(router, "DELETE", &uri, None).await;
        assert_eq!(again.status(), StatusCode::NOT_FOUND);
    }
}
