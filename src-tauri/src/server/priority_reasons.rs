//! `/api/priority-reasons`: 우선 배정 사유(BR-S10, BR-S11).
//!
//! 목록은 모든 역할이 보고, 바꾸기는 관리자만 한다.

mod json;

use axum::Json;
use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use baeum_domain::{Action, NewPriorityReason, PriorityError};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::auth::CurrentStaff;
use super::error::ApiError;
use crate::db::priority_reasons::{self, SavePriorityReasonError};

use json::{
    CreatePriorityReasonResponse, PriorityReasonListResponse, PriorityReasonRequest,
    PriorityReasonResponse,
};

/// 사유를 모두 보여 준다. 이름 순서.
pub async fn list_priority_reasons(
    State(db): State<SqlitePool>,
) -> Result<Json<PriorityReasonListResponse>, ApiError> {
    let records = priority_reasons::list(&db)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(PriorityReasonListResponse {
        items: records
            .into_iter()
            .map(PriorityReasonResponse::from)
            .collect(),
    }))
}

/// 새 사유를 저장한다. 성공하면 201과 새 id를 돌려준다.
pub async fn create_priority_reason(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    body: Result<Json<PriorityReasonRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    staff.require(Action::ManagePriorityReasons)?;
    let Json(request) = body?;
    let reason = parse(&request)?;
    let id = priority_reasons::insert(&db, &reason, OffsetDateTime::now_utc()).await?;
    // BR-S15: 사유 이름은 민감할 수 있어 id만 남긴다
    tracing::info!(staff_id = staff.id, reason_id = id, "우선 배정 사유 저장");

    let location = format!("/api/priority-reasons/{id}");
    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, location)],
        Json(CreatePriorityReasonResponse { id }),
    ))
}

/// `id`인 사유의 이름과 방식을 본문 값으로 바꾼다.
pub async fn update_priority_reason(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
    body: Result<Json<PriorityReasonRequest>, JsonRejection>,
) -> Result<Json<PriorityReasonResponse>, ApiError> {
    staff.require(Action::ManagePriorityReasons)?;
    let Path(id) = id?;
    let Json(request) = body?;
    let reason = parse(&request)?;
    let record = priority_reasons::update(&db, id, &reason, OffsetDateTime::now_utc())
        .await?
        .ok_or(ApiError::PriorityReasonNotFound)?;
    tracing::info!(staff_id = staff.id, reason_id = id, "우선 배정 사유 고침");
    Ok(Json(PriorityReasonResponse::from(record)))
}

/// `id`인 사유를 지운다. 성공하면 204. BR-S10: 신청에 붙었던 사유면 409.
pub async fn delete_priority_reason(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    id: Result<Path<i64>, PathRejection>,
) -> Result<StatusCode, ApiError> {
    staff.require(Action::ManagePriorityReasons)?;
    let Path(id) = id?;
    let deleted = priority_reasons::delete(&db, id)
        .await
        .map_err(|error| ApiError::from_delete(error, ApiError::PriorityReasonInUse))?;
    if !deleted {
        return Err(ApiError::PriorityReasonNotFound);
    }
    tracing::info!(staff_id = staff.id, reason_id = id, "우선 배정 사유 지움");
    Ok(StatusCode::NO_CONTENT)
}

fn parse(request: &PriorityReasonRequest) -> Result<NewPriorityReason, ApiError> {
    NewPriorityReason::parse(&request.name, request.method.into())
        .map_err(|error| ApiError::invalid_input(vec![error], field_name))
}

/// 오류가 난 칸의 요청 JSON 이름.
fn field_name(error: &PriorityError) -> &'static str {
    match error {
        PriorityError::EmptyName | PriorityError::NameTooLong => "name",
    }
}

impl From<SavePriorityReasonError> for ApiError {
    fn from(error: SavePriorityReasonError) -> Self {
        match error {
            SavePriorityReasonError::DuplicateName => ApiError::DuplicatePriorityReasonName,
            SavePriorityReasonError::Db(error) => ApiError::internal(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::db::test_support::{
        insert_application, insert_course, insert_member, insert_staff, insert_subject, insert_term,
    };
    use crate::server::test_support::{app, app_as, create, delete, get, json_body, patch, post};

    fn reason(name: &str) -> String {
        json!({ "name": name, "method": "guaranteed" }).to_string()
    }

    #[tokio::test]
    async fn creates_lists_and_renames_reason() {
        let (_dir, _db, router) = app().await;

        let id = create(router.clone(), "/api/priority-reasons", &reason(" 반장 ")).await;
        let renamed = patch(
            router.clone(),
            &format!("/api/priority-reasons/{id}"),
            &reason("지난 학기 반장"),
        )
        .await;

        assert_eq!(renamed.status(), StatusCode::OK);
        let list = json_body(get(router, "/api/priority-reasons").await).await;
        assert_eq!(
            list,
            json!({ "items": [{ "id": id, "name": "지난 학기 반장", "method": "guaranteed" }] })
        );
    }

    #[tokio::test]
    async fn br_s10_duplicate_name_and_blank_name_are_refused() {
        let (_dir, _db, router) = app().await;
        create(
            router.clone(),
            "/api/priority-reasons",
            &reason("장애인 우대"),
        )
        .await;

        let duplicate = post(
            router.clone(),
            "/api/priority-reasons",
            &reason("장애인우대"),
        )
        .await;
        let blank = post(router.clone(), "/api/priority-reasons", &reason("  ")).await;
        let unknown_method = post(
            router,
            "/api/priority-reasons",
            &json!({ "name": "다자녀", "method": "quota" }).to_string(),
        )
        .await;

        assert_eq!(duplicate.status(), StatusCode::CONFLICT);
        assert_eq!(
            json_body(duplicate).await["code"],
            "DUPLICATE_PRIORITY_REASON_NAME"
        );
        assert_eq!(blank.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(json_body(blank).await["fields"][0]["field"], "name");
        assert!(unknown_method.status().is_client_error());
    }

    #[tokio::test]
    async fn br_s10_only_admin_changes_reasons_but_everyone_sees_them() {
        for role in ["staff", "temp_staff", "read_only"] {
            let (_dir, _db, router) = app_as(role).await;

            let created = post(router.clone(), "/api/priority-reasons", &reason("반장")).await;
            let listed = get(router, "/api/priority-reasons").await;

            assert_eq!(created.status(), StatusCode::FORBIDDEN, "{role}");
            assert_eq!(listed.status(), StatusCode::OK, "{role}");
        }
    }

    #[tokio::test]
    async fn br_s10_used_reason_cannot_be_deleted() {
        let (_dir, db, router) = app().await;
        let used = create(router.clone(), "/api/priority-reasons", &reason("반장")).await;
        let unused = create(
            router.clone(),
            "/api/priority-reasons",
            &reason("국가유공자"),
        )
        .await;
        let term_id = insert_term(&db, "2027년 1학기", "2027-03-02", "2027-06-30")
            .await
            .unwrap();
        let subject_id = insert_subject(&db, "요가", "요가", None).await.unwrap();
        let course_id = insert_course(&db, term_id, subject_id, None, None)
            .await
            .unwrap();
        let member_id = insert_member(&db, "00001").await.unwrap();
        let staff_id = insert_staff(&db, "직원1", "staff", None).await.unwrap();
        let application_id = insert_application(&db, member_id, course_id, "submitted", staff_id)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO priority_changes
                 (application_id, member_id, course_id, reason_id, change, changed_by, changed_at)
             VALUES (?, ?, ?, ?, 'added', ?, '2026-11-22T00:00:00.000Z')",
        )
        .bind(application_id)
        .bind(member_id)
        .bind(course_id)
        .bind(used)
        .bind(staff_id)
        .execute(&db)
        .await
        .unwrap();

        let in_use = delete(router.clone(), &format!("/api/priority-reasons/{used}")).await;
        let removed = delete(router.clone(), &format!("/api/priority-reasons/{unused}")).await;
        let missing = delete(router, &format!("/api/priority-reasons/{unused}")).await;

        assert_eq!(in_use.status(), StatusCode::CONFLICT);
        assert_eq!(json_body(in_use).await["code"], "PRIORITY_REASON_IN_USE");
        assert_eq!(removed.status(), StatusCode::NO_CONTENT);
        assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    }
}
