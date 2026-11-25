//! 추첨 요청 처리 테스트.

use axum::Router;
use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::SqlitePool;

use crate::db::test_support::{
    insert_application, insert_course, insert_member, insert_priority_reason, insert_staff,
    insert_subject, insert_term,
};
use crate::infra::backup_file::name::BackupKind;
use crate::server::test_support::{app, app_as, from_approved_pc, get, json_body, post, put};

/// 정원 2명인 강좌에 접수됨 신청 셋. 강좌를 만들 수 없는 역할도 시험하려고 DB에 바로 넣는다.
struct Setup {
    term_id: i64,
    course_id: i64,
}

async fn setup(db: &SqlitePool, applicants: usize) -> Setup {
    let term_id = insert_term(db, "2027년 1학기", "2027-03-02", "2027-06-30")
        .await
        .unwrap();
    let subject_id = insert_subject(db, "요가", "요가", None).await.unwrap();
    let course_id = insert_course(db, term_id, subject_id, None, None)
        .await
        .unwrap();
    sqlx::query("UPDATE courses SET capacity = 2 WHERE id = ?")
        .bind(course_id)
        .execute(db)
        .await
        .unwrap();
    let staff_id = insert_staff(db, "접수직원", "staff", None).await.unwrap();
    for number in 1..=applicants {
        let member_id = insert_member(db, &format!("12-{number:05}")).await.unwrap();
        insert_application(db, member_id, course_id, "submitted", staff_id)
            .await
            .unwrap();
    }
    Setup { term_id, course_id }
}

impl Setup {
    fn lottery_uri(&self) -> String {
        format!("/api/courses/{}/lottery", self.course_id)
    }

    fn closure_uri(&self) -> String {
        format!("/api/courses/{}/selection-closure", self.course_id)
    }

    async fn draw(&self, router: Router) -> (StatusCode, Value) {
        let response = post(router, &self.lottery_uri(), "").await;
        (response.status(), json_body(response).await)
    }

    async fn close(&self, router: Router) -> StatusCode {
        post(router, &self.closure_uri(), "").await.status()
    }

    async fn lottery(&self, router: Router) -> Value {
        json_body(get(router, &self.lottery_uri()).await).await
    }
}

fn member_nos(lottery: &Value) -> Vec<String> {
    let mut nos: Vec<String> = lottery["entrants"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entrant| entrant["memberNo"].as_str().unwrap().to_owned())
        .collect();
    nos.sort();
    nos
}

#[tokio::test]
async fn shows_applicants_before_lottery() {
    let (_dir, db, router) = app().await;
    let s = setup(&db, 3).await;

    let lottery = s.lottery(router).await;

    assert_eq!(lottery["capacity"], 2);
    assert_eq!(lottery["stage"], "not_drawn");
    assert_eq!(lottery["selectedCount"], 0);
    assert_eq!(lottery["canDraw"], true);
    assert_eq!(lottery["entrants"][0]["memberNo"], "12-00001");
    assert_eq!(lottery["entrants"][0]["memberName"], "회원1");
}

#[tokio::test]
async fn br_s7_draws_and_shows_drawn_order() {
    let (_dir, db, router) = app().await;
    let s = setup(&db, 3).await;

    let (status, drawn) = s.draw(router.clone()).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(drawn["stage"], "drawn");
    assert_eq!(drawn["selectedCount"], 2);
    assert_eq!(member_nos(&drawn), ["12-00001", "12-00002", "12-00003"]);
    assert_eq!(s.lottery(router.clone()).await, drawn);
    let selections_uri = format!("/api/terms/{}/selections", s.term_id);
    assert_eq!(
        json_body(get(router, &selections_uri).await).await,
        json!({ "items": [{ "courseId": s.course_id, "stage": "drawn" }] })
    );
}

#[tokio::test]
async fn br_l4_l5_redraw_until_closed() {
    let (dir, db, router) = app().await;
    let s = setup(&db, 3).await;
    assert_eq!(s.close(router.clone()).await, StatusCode::CONFLICT);
    s.draw(router.clone()).await;

    let (again, _) = s.draw(router.clone()).await;
    let closed = s.close(router.clone()).await;

    assert_eq!(again, StatusCode::OK);
    assert_eq!(closed, StatusCode::NO_CONTENT);
    // ADR 0011: 추첨과 마감 직전마다 백업이 남는다
    let kinds: Vec<BackupKind> =
        crate::infra::backup_file::store::list(&crate::infra::backup_file::dir(dir.path()))
            .await
            .unwrap()
            .into_iter()
            .map(|backup| backup.name.kind)
            .collect();
    assert!(kinds.contains(&BackupKind::BeforeLottery {
        course_id: s.course_id
    }));
    assert!(kinds.contains(&BackupKind::BeforeClose {
        course_id: s.course_id
    }));
    assert_eq!(s.lottery(router.clone()).await["stage"], "closed");
    let (status, body) = s.draw(router.clone()).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "SELECTION_CLOSED");
    assert_eq!(body["rule"], "BR-L5");
    assert_eq!(s.close(router).await, StatusCode::CONFLICT);
}

#[tokio::test]
async fn br_l6_other_pc_can_view_but_not_draw() {
    let (_dir, db, router) = app().await;
    let s = setup(&db, 3).await;
    let other_pc = from_approved_pc(&db, router).await;

    let (status, body) = s.draw(other_pc.clone()).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["code"], "HOST_ONLY");
    assert_eq!(body["rule"], "BR-L6");
    assert_eq!(s.close(other_pc.clone()).await, StatusCode::FORBIDDEN);
    let lottery = s.lottery(other_pc).await;
    assert_eq!(lottery["stage"], "not_drawn");
    assert_eq!(lottery["canDraw"], false);
}

#[tokio::test]
async fn temp_staff_and_read_only_can_view_but_not_draw() {
    for role in ["temp_staff", "read_only"] {
        let (_dir, db, router) = app_as(role).await;
        let s = setup(&db, 3).await;

        let (status, body) = s.draw(router.clone()).await;

        assert_eq!(status, StatusCode::FORBIDDEN, "{role}");
        assert_eq!(body["code"], "NOT_ALLOWED", "{role}");
        assert_eq!(s.close(router.clone()).await, StatusCode::FORBIDDEN);
        assert_eq!(s.lottery(router).await["canDraw"], false, "{role}");
    }
}

#[tokio::test]
async fn no_applicants_and_missing_course_or_term() {
    let (_dir, db, router) = app().await;
    let s = setup(&db, 0).await;

    let (status, body) = s.draw(router.clone()).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "NO_APPLICANTS");

    let missing = Setup {
        term_id: s.term_id + 100,
        course_id: s.course_id + 100,
    };
    assert_eq!(missing.draw(router.clone()).await.0, StatusCode::NOT_FOUND);
    assert_eq!(missing.close(router.clone()).await, StatusCode::NOT_FOUND);
    let response = get(router.clone(), &missing.lottery_uri()).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let selections_uri = format!("/api/terms/{}/selections", missing.term_id);
    assert_eq!(
        get(router, &selections_uri).await.status(),
        StatusCode::NOT_FOUND
    );
}

/// 접수한 순서로 `count`건의 신청에 자리 보장 사유를 붙인다. 붙인 신청 id를 돌려준다.
async fn guarantee(db: &SqlitePool, router: Router, course_id: i64, count: i64) -> Vec<i64> {
    let reason_id = insert_priority_reason(db, "반장").await.unwrap();
    let ids: Vec<i64> =
        sqlx::query_scalar("SELECT id FROM applications WHERE course_id = ? ORDER BY id LIMIT ?")
            .bind(course_id)
            .bind(count)
            .fetch_all(db)
            .await
            .unwrap();
    for &id in &ids {
        let uri = format!("/api/applications/{id}/priority-reasons");
        let body = json!({ "reasonIds": [reason_id] }).to_string();
        assert_eq!(
            put(router.clone(), &uri, &body).await.status(),
            StatusCode::OK
        );
    }
    ids
}

#[tokio::test]
async fn br_s4_guaranteed_entrant_is_first_and_marked() {
    let (_dir, db, router) = app().await;
    let s = setup(&db, 3).await;
    let ids = guarantee(&db, router.clone(), s.course_id, 1).await;

    let (status, drawn) = s.draw(router).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(drawn["entrants"][0]["applicationId"], ids[0]);
    assert_eq!(drawn["entrants"][0]["guaranteed"], true);
    assert_eq!(drawn["entrants"][1]["guaranteed"], false);
    assert_eq!(drawn["selectedCount"], 2);
}

#[tokio::test]
async fn br_s5_more_guaranteed_than_capacity_is_conflict() {
    let (_dir, db, router) = app().await;
    let s = setup(&db, 3).await;
    guarantee(&db, router.clone(), s.course_id, 3).await;

    let (status, body) = s.draw(router).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "PRIORITY_OVER_CAPACITY");
}

#[tokio::test]
async fn br_s13_changing_priority_after_draw_cancels_lottery() {
    let (_dir, db, router) = app().await;
    let s = setup(&db, 3).await;
    s.draw(router.clone()).await;

    let reason_id = insert_priority_reason(&db, "반장").await.unwrap();
    let id: i64 = sqlx::query_scalar("SELECT MIN(id) FROM applications")
        .fetch_one(&db)
        .await
        .unwrap();
    let response = put(
        router.clone(),
        &format!("/api/applications/{id}/priority-reasons"),
        &json!({ "reasonIds": [reason_id] }).to_string(),
    )
    .await;

    assert_eq!(json_body(response).await["lotteryCancelled"], true);
    assert_eq!(s.lottery(router).await["stage"], "not_drawn");
}
