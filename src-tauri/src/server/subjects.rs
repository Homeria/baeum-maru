//! `/api/subjects`: 과목.

mod json;

use axum::Json;
use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use baeum_domain::{Action, NewSubject, SubjectError};

use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::auth::CurrentStaff;
use super::error::ApiError;
use crate::db::subjects::{self, SaveSubjectError};

use json::{CreateSubjectResponse, SubjectListResponse, SubjectRequest, SubjectResponse};

/// 과목을 모두 보여 준다. 분류, 이름 순서.
pub async fn list_subjects(
    State(db): State<SqlitePool>,
) -> Result<Json<SubjectListResponse>, ApiError> {
    let records = subjects::list(&db).await.map_err(ApiError::internal)?;
    Ok(Json(SubjectListResponse {
        items: records.into_iter().map(SubjectResponse::from).collect(),
    }))
}

/// `id`인 과목 하나를 보여 준다.
pub async fn get_subject(
    State(db): State<SqlitePool>,
    id: Result<Path<i64>, PathRejection>,
) -> Result<Json<SubjectResponse>, ApiError> {
    let Path(id) = id?;
    let record = subjects::get(&db, id)
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::SubjectNotFound)?;
    Ok(Json(SubjectResponse::from(record)))
}

/// 새 과목을 저장한다. 성공하면 201과 새 id를 돌려준다.
pub async fn create_subject(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    body: Result<Json<SubjectRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    staff.require(Action::EditReferenceData)?;
    let Json(request) = body?;
    let subject = NewSubject::parse(request.as_input())
        .map_err(|errors| ApiError::invalid_input(errors, field_name))?;
    let id = subjects::insert(&db, &subject, OffsetDateTime::now_utc()).await?;
    tracing::info!(staff_id = staff.id, subject_id = id, "과목 저장");

    let location = format!("/api/subjects/{id}");
    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, location)],
        Json(CreateSubjectResponse { id }),
    ))
}

/// `id`인 과목의 칸을 모두 본문 값으로 바꾼다. 확인 규칙은 등록과 같다.
pub async fn update_subject(
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
    let record = subjects::update(&db, id, &subject, OffsetDateTime::now_utc())
        .await?
        .ok_or(ApiError::SubjectNotFound)?;
    tracing::info!(staff_id = staff.id, subject_id = id, "과목 고침");
    Ok(Json(SubjectResponse::from(record)))
}

/// `id`인 과목을 지운다. 성공하면 204(본문 없음). BR-J2: 강좌가 쓰면 409.
pub async fn delete_subject(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
) -> Result<StatusCode, ApiError> {
    staff.require(Action::EditReferenceData)?;
    let Path(id) = id?;
    let deleted = subjects::delete(&db, id)
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

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::Request;
    use serde_json::json;
    use tower::ServiceExt;

    use super::*;
    use crate::server::test_support::{
        app, create, create_course, create_subject, create_term, delete, get, json_body, patch,
        post,
    };

    fn subject(name: &str, category: Option<&str>) -> String {
        json!({ "name": name, "category": category }).to_string()
    }

    #[tokio::test]
    async fn creates_and_lists_subject() {
        let (_dir, _db, router) = app().await;

        let response = post(
            router.clone(),
            "/api/subjects",
            &subject("한글 교실", Some("평생교육")),
        )
        .await;

        assert_eq!(response.status(), StatusCode::CREATED);
        let location = response.headers()[header::LOCATION]
            .to_str()
            .unwrap()
            .to_owned();
        let id = json_body(response).await["id"].clone();
        assert_eq!(location, format!("/api/subjects/{id}"));
        let list = json_body(get(router, "/api/subjects").await).await;
        assert_eq!(
            list,
            json!({ "items": [
                { "id": id, "name": "한글 교실", "category": "평생교육" }
            ]})
        );
    }

    #[tokio::test]
    async fn invalid_input_returns_every_field() {
        let (_dir, _db, router) = app().await;
        let long = "가".repeat(21);

        let response = post(router, "/api/subjects", &subject(" ", Some(&long))).await;

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = json_body(response).await;
        let fields: Vec<&str> = body["fields"]
            .as_array()
            .unwrap()
            .iter()
            .map(|field| field["field"].as_str().unwrap())
            .collect();
        assert_eq!(fields, ["name", "category"]);
    }

    #[tokio::test]
    async fn br_j1_name_differing_only_in_spaces_is_conflict() {
        let (_dir, _db, router) = app().await;
        post(router.clone(), "/api/subjects", &subject("한글교실", None)).await;

        let response = post(router, "/api/subjects", &subject("한글 교실", None)).await;

        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = json_body(response).await;
        assert_eq!(body["code"], "DUPLICATE_SUBJECT_NAME");
        assert_eq!(body["rule"], "BR-J1");
    }

    #[tokio::test]
    async fn category_can_be_left_out() {
        let (_dir, _db, router) = app().await;

        let response = post(router.clone(), "/api/subjects", r#"{ "name": "요가" }"#).await;

        assert_eq!(response.status(), StatusCode::CREATED);
        let list = json_body(get(router, "/api/subjects").await).await;
        assert_eq!(list["items"][0]["category"], json!(null));
    }

    #[tokio::test]
    async fn gets_one_subject() {
        let (_dir, _db, router) = app().await;
        let id = create(
            router.clone(),
            "/api/subjects",
            &subject("요가", Some("취미여가")),
        )
        .await;

        let response = get(router, &format!("/api/subjects/{id}")).await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            json_body(response).await,
            json!({ "id": id, "name": "요가", "category": "취미여가" })
        );
    }

    #[tokio::test]
    async fn unknown_subject_is_not_found() {
        let (_dir, _db, router) = app().await;

        let response = get(router, "/api/subjects/999").await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(json_body(response).await["code"], "SUBJECT_NOT_FOUND");
    }

    #[tokio::test]
    async fn non_number_id_is_malformed() {
        let (_dir, _db, router) = app().await;

        let response = get(router, "/api/subjects/abc").await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn updates_subject() {
        let (_dir, _db, router) = app().await;
        let id = create(
            router.clone(),
            "/api/subjects",
            &subject("요가", Some("취미여가")),
        )
        .await;

        let uri = format!("/api/subjects/{id}");
        let response = patch(router, &uri, &subject("필라테스", None)).await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            json_body(response).await,
            json!({ "id": id, "name": "필라테스", "category": null })
        );
    }

    #[tokio::test]
    async fn br_j1_update_to_taken_name_is_conflict() {
        let (_dir, _db, router) = app().await;
        create(router.clone(), "/api/subjects", &subject("한글교실", None)).await;
        let id = create(router.clone(), "/api/subjects", &subject("요가", None)).await;

        let uri = format!("/api/subjects/{id}");
        let response = patch(router, &uri, &subject("한글 교실", None)).await;

        assert_eq!(response.status(), StatusCode::CONFLICT);
        assert_eq!(json_body(response).await["rule"], "BR-J1");
    }

    #[tokio::test]
    async fn deletes_subject() {
        let (_dir, _db, router) = app().await;
        let id = create(router.clone(), "/api/subjects", &subject("요가", None)).await;
        let uri = format!("/api/subjects/{id}");

        let response = delete(router.clone(), &uri).await;

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert_eq!(delete(router, &uri).await.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn subjects_need_matching_version() {
        let (_dir, _db, router) = app().await;
        let request = Request::get("/api/subjects").body(Body::empty()).unwrap();

        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn br_j2_subject_used_by_course_is_conflict() {
        let (_dir, _db, router) = app().await;
        let subject_id = create_subject(router.clone(), "요가").await;
        let term_id = create_term(router.clone()).await;
        create_course(router.clone(), term_id, subject_id, None).await;

        let response = delete(router, &format!("/api/subjects/{subject_id}")).await;

        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = json_body(response).await;
        assert_eq!(body["code"], "SUBJECT_IN_USE");
        assert_eq!(body["rule"], "BR-J2");
    }
}
