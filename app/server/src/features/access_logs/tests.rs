//! 접속 기록 테스트.

mod handler {
    use axum::http::StatusCode;
    use serde_json::json;

    use crate::server::test_support::{app, app_as, get, json_body, post};

    #[tokio::test]
    async fn admin_sees_newest_first() {
        let (_dir, _db, router) = app().await;
        // app()이 로그인을 한 줄 남겼다. 없는 직원으로 로그인 실패를 하나 더한다
        let login = json!({ "staffId": 999, "pin": "123456" }).to_string();
        post(router.clone(), "/api/login", &login).await;

        let response = get(router.clone(), "/api/access-logs?pageSize=1").await;

        assert_eq!(response.status(), StatusCode::OK);
        let body = json_body(response).await;
        assert_eq!(body["total"], 2);
        assert_eq!(body["page"], 1);
        assert_eq!(body["pageSize"], 1);
        let item = &body["items"][0];
        assert_eq!(item["event"], "login_failed");
        assert_eq!(item["staffId"], json!(null));
        assert_eq!(item["staffName"], json!(null));
        assert!(item["occurredAt"].as_str().unwrap().ends_with('Z'));

        let second = json_body(get(router, "/api/access-logs?page=2&pageSize=1").await).await;
        assert_eq!(second["items"][0]["event"], "login");
        assert_eq!(second["items"][0]["staffName"], "시험직원");
    }

    #[tokio::test]
    async fn only_admin_sees_access_logs() {
        for role in ["staff", "temp_staff", "read_only"] {
            let (_dir, _db, router) = app_as(role).await;

            let response = get(router, "/api/access-logs").await;

            assert_eq!(response.status(), StatusCode::FORBIDDEN, "{role}");
        }
    }

    #[tokio::test]
    async fn rejects_page_out_of_range() {
        let (_dir, _db, router) = app().await;

        let response = get(router, "/api/access-logs?page=0&pageSize=101").await;

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = json_body(response).await;
        assert_eq!(body["fields"][0]["field"], "page");
        assert_eq!(body["fields"][1]["field"], "pageSize");
    }
}

mod repository {
    use time::macros::datetime;

    use crate::db::test_support::insert_staff;
    use crate::features::access_logs::repository::*;
    use crate::infra::db::open_temp;

    #[tokio::test]
    async fn records_event_with_and_without_staff() {
        let (_dir, pool) = open_temp().await;
        let staff_id = insert_staff(&pool, "직원1", "staff", None).await.unwrap();
        let now = datetime!(2026-11-15 01:02:03 UTC);

        record(&pool, Some(staff_id), AccessEvent::Login, now)
            .await
            .unwrap();
        record(&pool, None, AccessEvent::LoginFailed, now)
            .await
            .unwrap();

        assert_eq!(
            list_for_test(&pool).await,
            [
                (Some(staff_id), "login".to_owned()),
                (None, "login_failed".to_owned()),
            ]
        );
    }

    #[tokio::test]
    async fn lists_newest_first_by_page_with_staff_name() {
        let (_dir, pool) = open_temp().await;
        let staff_id = insert_staff(&pool, "직원1", "staff", None).await.unwrap();
        let at = |minute: u8| {
            datetime!(2026-11-15 01:00 UTC)
                .replace_minute(minute)
                .unwrap()
        };
        record(&pool, None, AccessEvent::LoginFailed, at(0))
            .await
            .unwrap();
        record(&pool, Some(staff_id), AccessEvent::Login, at(1))
            .await
            .unwrap();
        record(&pool, Some(staff_id), AccessEvent::Logout, at(2))
            .await
            .unwrap();

        let first = list(&pool, 1, 2).await.unwrap();
        let second = list(&pool, 2, 2).await.unwrap();

        assert_eq!(first.total, 3);
        let events: Vec<AccessEvent> = first.items.iter().map(|item| item.event).collect();
        assert_eq!(events, [AccessEvent::Logout, AccessEvent::Login]);
        assert_eq!(first.items[0].staff_name.as_deref(), Some("직원1"));
        assert_eq!(first.items[0].occurred_at, at(2));
        assert_eq!(
            second.items,
            [AccessLogRecord {
                id: second.items[0].id,
                occurred_at: at(0),
                staff_id: None,
                staff_name: None,
                device_name: None,
                event: AccessEvent::LoginFailed,
            }]
        );
    }
}
