//! 결과 명단 요청 처리 테스트.

use axum::Router;
use axum::http::StatusCode;
use axum::http::header;
use serde_json::{Value, json};
use sqlx::SqlitePool;

use crate::db::test_support::insert_drawn_course;
use crate::features::rosters::handler::*;
use crate::features::rosters::xlsx::test_support::read_sheet;
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
    // 제목, 빈 줄, 머리글 + 당첨 2명과 대기 3명 중 긴 쪽
    assert_eq!(rows.len(), 6);
    assert_eq!(rows[2][2], "연락처");
    assert!(rows[3][2].starts_with("010-"));
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
