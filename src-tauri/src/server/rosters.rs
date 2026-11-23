//! 결과 명단 내보내기: `POST /api/terms/{termId}/roster-exports`. 응답은 Excel 파일이다.
//!
//! 원본 명단은 관리자·직원만, 게시용은 모든 역할이 내보낸다(BR-P10). 내보낼 때마다 기록을
//! 남긴다(BR-P11). 개인정보가 담긴 파일이라 응답을 캐시하지 않게 한다.

mod json;

use axum::Json;
use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::extract::{Path, State};
use axum::http::header;
use axum::response::IntoResponse;
use baeum_domain::{Action, RosterChoice, RosterColumn, RosterError, RosterKind, RosterSpec};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::applications::{field_error, require_term};
use super::auth::CurrentStaff;
use super::error::ApiError;
use crate::db::rosters;
use crate::roster_xlsx;

use json::RosterExportRequest;

/// Excel 파일(.xlsx)의 MIME 형식.
const XLSX: &str = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";

/// 학기에서 선발을 마감한 강좌들의 결과 명단을 Excel로 만든다.
///
/// 학기가 없으면 404, 마감한 강좌가 없으면 409, 그 종류에서 고를 수 없는 칸이면 422.
pub async fn export_roster(
    State(db): State<SqlitePool>,
    staff: CurrentStaff,
    term_id: Result<Path<i64>, PathRejection>,
    body: Result<Json<RosterExportRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let Path(term_id) = term_id?;
    let Json(request) = body?;
    let kind = RosterKind::from(request.kind);
    if kind == RosterKind::Raw {
        staff.require(Action::ExportRawRoster)?;
    }
    let include: Vec<RosterColumn> = request.include.into_iter().map(Into::into).collect();
    let mask: Vec<RosterColumn> = request.mask.into_iter().map(Into::into).collect();
    let spec = RosterSpec::new(RosterChoice {
        kind,
        include: &include,
        mask: &mask,
        mask_char: request.mask_char.into(),
    })
    .map_err(|error| {
        let field = match error {
            RosterError::NotIncludable(_) => "include",
            RosterError::NotMaskable(_) => "mask",
        };
        field_error(field, error.to_string())
    })?;
    require_term(&db, term_id).await?;

    let roster = rosters::term_roster(&db, term_id)
        .await
        .map_err(ApiError::internal)?;
    if roster.is_empty() {
        return Err(ApiError::NoClosedCourses);
    }
    let file = roster_xlsx::build(&spec, &roster).map_err(ApiError::internal)?;
    let export_id = rosters::record_export(
        &db,
        term_id,
        &spec,
        roster.len(),
        staff.id,
        OffsetDateTime::now_utc(),
    )
    .await
    .map_err(ApiError::internal)?;
    tracing::info!(
        staff_id = staff.id,
        term_id,
        export_id,
        kind = kind.code(),
        "결과 명단 내보내기"
    );
    Ok((
        [
            (header::CONTENT_TYPE, XLSX),
            // 개인정보가 담긴 파일을 어디에도 남기지 않게
            (header::CACHE_CONTROL, "no-store"),
        ],
        file,
    ))
}

#[cfg(test)]
mod tests {
    use axum::Router;
    use axum::http::StatusCode;
    use serde_json::{Value, json};

    use super::*;
    use crate::db::test_support::insert_drawn_course;
    use crate::roster_xlsx::test_support::read_sheet;
    use crate::server::test_support::{app, app_as, json_body, post};

    async fn export(router: Router, term_id: i64, body: Value) -> axum::response::Response {
        let uri = format!("/api/terms/{term_id}/roster-exports");
        post(router, &uri, &body.to_string()).await
    }

    async fn bytes(response: axum::response::Response) -> Vec<u8> {
        axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec()
    }

    async fn export_count(db: &SqlitePool) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM roster_exports")
            .fetch_one(db)
            .await
            .unwrap()
    }

    fn raw_with_phone() -> Value {
        json!({ "kind": "raw", "include": ["phone_number"], "mask": [], "maskChar": "circle" })
    }

    #[tokio::test]
    async fn br_p11_exports_excel_and_records_it() {
        let (_dir, db, router) = app().await;
        let drawn = insert_drawn_course(&db, 2, 5, true).await;

        let response = export(router, drawn.term_id, raw_with_phone()).await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CONTENT_TYPE], XLSX);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        let (sheet, rows) = read_sheet(&bytes(response).await, 0);
        assert_eq!(sheet, "요가(2)");
        // 머리글 + 당첨 2명과 대기 3명 중 긴 쪽
        assert_eq!(rows.len(), 4);
        assert_eq!(rows[0][2], "당첨자 연락처");
        assert!(rows[1][2].starts_with("010-"));
        let recorded: (String, String) =
            sqlx::query_as("SELECT kind, included_columns FROM roster_exports")
                .fetch_one(&db)
                .await
                .unwrap();
        assert_eq!(
            recorded,
            (
                "raw".to_owned(),
                "course_name,capacity,result,waitlist_number,name,member_no,phone_number,guaranteed"
                    .to_owned()
            )
        );
    }

    #[tokio::test]
    async fn br_p10_read_only_exports_posted_but_not_raw() {
        let (_dir, db, router) = app_as("read_only").await;
        let drawn = insert_drawn_course(&db, 2, 5, true).await;
        let posted = json!({
            "kind": "posted_by_member", "include": [], "mask": ["name"], "maskChar": "x"
        });

        let raw = export(router.clone(), drawn.term_id, raw_with_phone()).await;
        let member = export(router, drawn.term_id, posted).await;

        assert_eq!(raw.status(), StatusCode::FORBIDDEN);
        assert_eq!(member.status(), StatusCode::OK);
        assert_eq!(export_count(&db).await, 1);
    }

    #[tokio::test]
    async fn br_p1_term_without_closed_course_is_conflict() {
        let (_dir, db, router) = app().await;
        let drawn = insert_drawn_course(&db, 2, 5, false).await;

        let response = export(router.clone(), drawn.term_id, raw_with_phone()).await;
        let missing = export(router, 999, raw_with_phone()).await;

        assert_eq!(response.status(), StatusCode::CONFLICT);
        assert_eq!(json_body(response).await["code"], "NO_CLOSED_COURSES");
        assert_eq!(missing.status(), StatusCode::NOT_FOUND);
        assert_eq!(export_count(&db).await, 0);
    }

    #[tokio::test]
    async fn refuses_columns_the_kind_cannot_choose() {
        let (_dir, db, router) = app().await;
        let drawn = insert_drawn_course(&db, 2, 5, true).await;
        let body = json!({
            "kind": "raw", "include": [], "mask": ["gender"], "maskChar": "circle"
        });
        let unknown = json!({
            "kind": "raw", "include": ["priority_reason"], "mask": [], "maskChar": "circle"
        });

        let response = export(router.clone(), drawn.term_id, body).await;
        let unknown = export(router, drawn.term_id, unknown).await;

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(json_body(response).await["fields"][0]["field"], "mask");
        // 없는 칸 이름(예: 우선 배정 사유)은 JSON 모양부터 틀려 400 (BR-P8)
        assert_eq!(unknown.status(), StatusCode::BAD_REQUEST);
        assert_eq!(export_count(&db).await, 0);
    }
}
