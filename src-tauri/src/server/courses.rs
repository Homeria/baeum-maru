//! 강좌: 목록·등록은 `/api/terms/{termId}/courses`, 보기·고치기·지우기는 `/api/courses/{id}`.

mod json;

use axum::Json;
use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use baeum_domain::{Action, CourseError, NewCourse};
use chrono::Utc;
use sqlx::SqlitePool;

use super::auth::CurrentStaff;
use super::error::ApiError;
use crate::db::courses::{self, MissingReference, SaveCourseError};
use crate::db::terms;

use json::{CourseListResponse, CourseRequest, CourseResponse, CreateCourseResponse};

/// 학기의 강좌를 모두 보여 준다. 과목 이름, 등록 순서. 학기가 없으면 404.
pub async fn list_courses(
    State(db): State<SqlitePool>,
    term_id: Result<Path<i64>, PathRejection>,
) -> Result<Json<CourseListResponse>, ApiError> {
    let Path(term_id) = term_id?;
    terms::get(&db, term_id)
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::TermNotFound)?;
    let records = courses::list(&db, term_id)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(CourseListResponse {
        items: records.into_iter().map(CourseResponse::from).collect(),
    }))
}

/// `id`인 강좌 하나를 보여 준다.
pub async fn get_course(
    State(db): State<SqlitePool>,
    id: Result<Path<i64>, PathRejection>,
) -> Result<Json<CourseResponse>, ApiError> {
    let Path(id) = id?;
    let record = courses::get(&db, id)
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::CourseNotFound)?;
    Ok(Json(CourseResponse::from(record)))
}

/// 학기에 새 강좌를 저장한다. 성공하면 201과 새 id를 돌려준다.
pub async fn create_course(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    term_id: Result<Path<i64>, PathRejection>,
    body: Result<Json<CourseRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    staff.require(Action::EditCourses)?;
    let Path(term_id) = term_id?;
    let Json(request) = body?;
    let course = parse(&request)?;
    let id = courses::insert(&db, term_id, &course, Utc::now()).await?;
    tracing::info!(staff_id = staff.id, term_id, course_id = id, "강좌 저장");

    let location = format!("/api/courses/{id}");
    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, location)],
        Json(CreateCourseResponse { id }),
    ))
}

/// `id`인 강좌의 칸과 수업 시간을 모두 본문 값으로 바꾼다. 확인 규칙은 등록과 같다.
pub async fn update_course(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
    body: Result<Json<CourseRequest>, JsonRejection>,
) -> Result<Json<CourseResponse>, ApiError> {
    staff.require(Action::EditCourses)?;
    let Path(id) = id?;
    let Json(request) = body?;
    let course = parse(&request)?;
    let record = courses::update(&db, id, &course, Utc::now())
        .await?
        .ok_or(ApiError::CourseNotFound)?;
    tracing::info!(staff_id = staff.id, course_id = id, "강좌 고침");
    Ok(Json(CourseResponse::from(record)))
}

/// `id`인 강좌와 그 수업 시간을 지운다. 성공하면 204(본문 없음).
pub async fn delete_course(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
) -> Result<StatusCode, ApiError> {
    staff.require(Action::EditCourses)?;
    let Path(id) = id?;
    let deleted = courses::delete(&db, id).await.map_err(ApiError::internal)?;
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

#[cfg(test)]
mod tests {
    use axum::Router;
    use serde_json::{Value, json};

    use super::*;
    use crate::server::test_support::{
        app, create, create_room, create_subject, create_term, delete, get, json_body, patch, post,
    };

    /// 학기 하나, 과목 하나(요가), 강의실 하나를 API로 넣어 둔다. (학기 id, 과목 id, 강의실 id)
    async fn setup(router: &Router) -> (i64, i64, i64) {
        let term_id = create_term(router.clone()).await;
        let subject_id = create_subject(router.clone(), "요가").await;
        let room_id = create_room(router.clone()).await;
        (term_id, subject_id, room_id)
    }

    fn course(subject_id: Value, schedules: Value) -> String {
        json!({
            "subjectId": subject_id,
            "level": "초급",
            "section": null,
            "capacity": 20,
            "schedules": schedules,
        })
        .to_string()
    }

    fn courses_uri(term_id: i64) -> String {
        format!("/api/terms/{term_id}/courses")
    }

    fn field_names(body: &Value) -> Vec<&str> {
        body["fields"]
            .as_array()
            .unwrap()
            .iter()
            .map(|field| field["field"].as_str().unwrap())
            .collect()
    }

    #[tokio::test]
    async fn creates_and_lists_course() {
        let (_dir, _db, router) = app().await;
        let (term_id, subject_id, room_id) = setup(&router).await;
        let body = course(
            json!(subject_id),
            json!([
                { "weekday": 3, "start": "10:00", "end": "11:00", "roomId": null },
                { "weekday": 1, "start": "10:00", "end": "11:00", "roomId": room_id },
            ]),
        );

        let response = post(router.clone(), &courses_uri(term_id), &body).await;

        assert_eq!(response.status(), StatusCode::CREATED);
        let id = json_body(response).await["id"].clone();
        let list = json_body(get(router, &courses_uri(term_id)).await).await;
        assert_eq!(
            list,
            json!({ "items": [{
                "id": id,
                "termId": term_id,
                "subjectId": subject_id,
                "subjectName": "요가",
                "level": "초급",
                "section": null,
                "capacity": 20,
                "schedules": [
                    { "weekday": 1, "start": "10:00", "end": "11:00",
                      "room": { "id": room_id, "building": "본관", "name": "201호" } },
                    { "weekday": 3, "start": "10:00", "end": "11:00", "room": null },
                ]
            }]})
        );
    }

    #[tokio::test]
    async fn invalid_fields_are_field_errors() {
        let (_dir, _db, router) = app().await;
        let (term_id, _, _) = setup(&router).await;
        let body = json!({
            "subjectId": null,
            "level": null,
            "section": null,
            "capacity": 0,
            "schedules": [{ "weekday": 1, "start": "11:00", "end": "10:00", "roomId": null }],
        })
        .to_string();

        let response = post(router, &courses_uri(term_id), &body).await;

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = json_body(response).await;
        assert_eq!(field_names(&body), ["subjectId", "capacity", "schedules"]);
        assert_eq!(
            body["fields"][2]["message"],
            "1번째 수업 시간: 종료 시각은 시작 시각보다 늦어야 합니다"
        );
    }

    #[tokio::test]
    async fn weekday_out_of_range_is_malformed() {
        let (_dir, _db, router) = app().await;
        let (term_id, subject_id, _) = setup(&router).await;
        let body = course(
            json!(subject_id),
            json!([{ "weekday": 8, "start": "10:00", "end": "11:00", "roomId": null }]),
        );

        let response = post(router, &courses_uri(term_id), &body).await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn missing_subject_and_room_are_field_errors() {
        let (_dir, _db, router) = app().await;
        let (term_id, _, room_id) = setup(&router).await;
        let body = course(
            json!(999),
            json!([
                { "weekday": 1, "start": "10:00", "end": "11:00", "roomId": room_id },
                { "weekday": 3, "start": "10:00", "end": "11:00", "roomId": 998 },
            ]),
        );

        let response = post(router, &courses_uri(term_id), &body).await;

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = json_body(response).await;
        assert_eq!(field_names(&body), ["subjectId", "schedules"]);
        assert_eq!(
            body["fields"][1]["message"],
            "2번째 수업 시간: 목록에 없는 강의실입니다. 다시 골라 주세요"
        );
    }

    #[tokio::test]
    async fn unknown_term_is_not_found() {
        let (_dir, _db, router) = app().await;
        let (_, subject_id, _) = setup(&router).await;
        let body = course(
            json!(subject_id),
            json!([{ "weekday": 1, "start": "10:00", "end": "11:00", "roomId": null }]),
        );

        let response = get(router.clone(), &courses_uri(999)).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(json_body(response).await["code"], "TERM_NOT_FOUND");
        let response = post(router, &courses_uri(999), &body).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn br_t3_duplicate_course_is_conflict() {
        let (_dir, _db, router) = app().await;
        let (term_id, subject_id, _) = setup(&router).await;
        let body = course(
            json!(subject_id),
            json!([{ "weekday": 1, "start": "10:00", "end": "11:00", "roomId": null }]),
        );
        post(router.clone(), &courses_uri(term_id), &body).await;

        let response = post(router, &courses_uri(term_id), &body).await;

        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = json_body(response).await;
        assert_eq!(body["code"], "DUPLICATE_COURSE");
        assert_eq!(body["rule"], "BR-T3");
    }

    #[tokio::test]
    async fn gets_updates_and_deletes_course() {
        let (_dir, _db, router) = app().await;
        let (term_id, subject_id, _) = setup(&router).await;
        let body = course(
            json!(subject_id),
            json!([{ "weekday": 1, "start": "10:00", "end": "11:00", "roomId": null }]),
        );
        let id = create(router.clone(), &courses_uri(term_id), &body).await;
        let uri = format!("/api/courses/{id}");

        let changed = course(
            json!(subject_id),
            json!([{ "weekday": 5, "start": "14:00", "end": "15:30", "roomId": null }]),
        );
        let response = patch(router.clone(), &uri, &changed).await;

        assert_eq!(response.status(), StatusCode::OK);
        let read = json_body(get(router.clone(), &uri).await).await;
        assert_eq!(
            read["schedules"],
            json!([{ "weekday": 5, "start": "14:00", "end": "15:30", "room": null }])
        );
        assert_eq!(
            delete(router.clone(), &uri).await.status(),
            StatusCode::NO_CONTENT
        );
        let response = get(router.clone(), &uri).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(json_body(response).await["code"], "COURSE_NOT_FOUND");
        assert_eq!(
            patch(router.clone(), &uri, &changed).await.status(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(delete(router, &uri).await.status(), StatusCode::NOT_FOUND);
    }
}
