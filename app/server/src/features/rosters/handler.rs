//! 결과 명단 내보내기: `POST /api/terms/{termId}/roster-exports`. 응답은 Excel 파일이다.
//!
//! 원본 명단은 관리자·직원만, 게시용은 모든 역할이 내보낸다(BR-P10). 내보낼 때마다 기록을
//! 남긴다(BR-P11). 개인정보가 담긴 파일이라 응답을 캐시하지 않게 한다.

use axum::Json;
use axum::Router;
use axum::extract::FromRef;
use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::extract::{Path, State};
use axum::http::header;
use axum::response::IntoResponse;
use axum::routing::post;
use baeum_domain::{Action, RosterChoice, RosterColumn, RosterError, RosterKind, RosterSpec};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::dto::RosterExportRequest;
use super::repository;
use super::xlsx;
use crate::features::auth::middleware::CurrentStaff;
use crate::features::terms::repository as terms;
use crate::http::changes::ChangeNotices;
use crate::http::error::ApiError;
use crate::http::error::field_error;

/// Excel 파일(.xlsx)의 MIME 형식.
pub(super) const XLSX: &str = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";

/// 결과 명단 요청 주소. 로그인한 요청만 받는 묶음에 들어간다 (ADR 0023).
pub(crate) fn routes<S>(_notices: &ChangeNotices) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    SqlitePool: FromRef<S>,
{
    Router::new().route("/api/terms/{term_id}/roster-exports", post(export_roster))
}

/// 학기에서 선발을 마감한 강좌들의 결과 명단을 Excel로 만든다.
///
/// 학기가 없으면 404, 마감한 강좌가 없으면 409, 그 종류에서 고를 수 없는 칸이면 422.
async fn export_roster(
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
    let term = terms::get(&db, term_id)
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::TermNotFound)?;

    let roster = repository::term_roster(&db, term_id)
        .await
        .map_err(ApiError::internal)?;
    if roster.is_empty() {
        return Err(ApiError::NoClosedCourses);
    }
    let file = xlsx::build(&spec, &roster, &term.name).map_err(ApiError::internal)?;
    let export_id = repository::record_export(
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
