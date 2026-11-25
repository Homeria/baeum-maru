//! 넘기기·실행 모드 요청 처리 테스트. 넘기기·넘겨받기·실행 모드 파일의 테스트는 그 파일 안에 있다.

mod handler {
    mod handover {
        use crate::features::handover::service::run_mode::{self, RunMode};
        use axum::Router;
        use serde_json::json;

        use crate::features::access_logs::repository::list_for_test;
        use crate::server::test_support::{app, app_as, from_approved_pc, json_body, post};
        use axum::http::StatusCode;
        use sqlx::SqlitePool;

        /// 로그인해 둔 첫 직원의 PIN을 123456으로 정한다.
        async fn set_pin(db: &SqlitePool) {
            sqlx::query("UPDATE staff SET pin_hash = ?")
                .bind(crate::infra::secret::hash("123456").unwrap())
                .execute(db)
                .await
                .unwrap();
        }

        async fn hand_over(router: Router, pin: &str) -> axum::response::Response {
            let body = json!({ "pin": pin }).to_string();
            post(router, "/api/handover", &body).await
        }

        #[tokio::test]
        async fn br_h1_admin_pin_starts_handover_and_is_logged() {
            let (dir, db, router) = app().await;
            set_pin(&db).await;

            let response = hand_over(router, "123456").await;

            assert_eq!(response.status(), StatusCode::NO_CONTENT);
            assert_eq!(run_mode::resolve(dir.path()), Some(RunMode::HandingOver));
            assert_eq!(
                list_for_test(&db).await.last().unwrap().1,
                "handover".to_owned()
            );
        }

        #[tokio::test]
        async fn wrong_pin_does_not_hand_over() {
            let (dir, db, router) = app().await;
            set_pin(&db).await;

            let response = hand_over(router, "000000").await;

            assert_eq!(json_body(response).await["code"], "WRONG_PIN");
            assert_ne!(run_mode::resolve(dir.path()), Some(RunMode::HandingOver));
            assert_eq!(
                list_for_test(&db).await.last().unwrap().1,
                "login_failed".to_owned()
            );
        }

        #[tokio::test]
        async fn only_admin_can_hand_over() {
            let (dir, db, router) = app_as("staff").await;
            set_pin(&db).await;

            let response = hand_over(router, "123456").await;

            assert_eq!(response.status(), StatusCode::FORBIDDEN);
            assert_ne!(run_mode::resolve(dir.path()), Some(RunMode::HandingOver));
        }

        #[tokio::test]
        async fn br_h1_only_on_host_pc() {
            let (dir, db, router) = app().await;
            set_pin(&db).await;
            let router = from_approved_pc(&db, router).await;

            let response = hand_over(router, "123456").await;

            assert_eq!(json_body(response).await["code"], "HOST_ONLY");
            assert_ne!(run_mode::resolve(dir.path()), Some(RunMode::HandingOver));
        }
    }

    mod run_mode {
        use axum::Router;
        use serde_json::json;

        use axum::http::StatusCode;

        use crate::db::test_support::insert_staff;
        use crate::features::access_logs::repository::list_for_test;
        use crate::server::test_support::{app_without_staff, get, json_body, post};

        /// 처음 설정으로 관리자(PIN 123456)를 만들고 id를 돌려준다.
        async fn setup_admin(router: Router) -> i64 {
            let setup = json!({ "name": "관리자", "pin": "123456" }).to_string();
            post(router.clone(), "/api/setup", &setup).await;
            let list = json_body(get(router, "/api/login/staff").await).await;
            list["items"][0]["id"].as_i64().unwrap()
        }

        fn body(staff_id: i64, pin: &str) -> String {
            json!({ "staffId": staff_id, "pin": pin }).to_string()
        }

        #[tokio::test]
        async fn br_k17_admin_pin_permits_and_is_logged() {
            let (_dir, db, router) = app_without_staff().await;
            let admin = setup_admin(router.clone()).await;

            let response = post(router, "/api/run-mode/permit", &body(admin, "123456")).await;

            assert_eq!(response.status(), StatusCode::NO_CONTENT);
            assert_eq!(
                list_for_test(&db).await.last().unwrap(),
                &(Some(admin), "run_mode_changed".to_owned())
            );
        }

        #[tokio::test]
        async fn wrong_pin_counts_as_failed_login() {
            let (_dir, db, router) = app_without_staff().await;
            let admin = setup_admin(router.clone()).await;

            let response = post(router, "/api/run-mode/permit", &body(admin, "000000")).await;

            assert_eq!(json_body(response).await["code"], "WRONG_PIN");
            assert_eq!(
                list_for_test(&db).await.last().unwrap(),
                &(Some(admin), "login_failed".to_owned())
            );
        }

        #[tokio::test]
        async fn only_admin_can_permit() {
            let (_dir, db, router) = app_without_staff().await;
            setup_admin(router.clone()).await;
            let staff_id = insert_staff(&db, "직원1", "staff", None).await.unwrap();

            let response = post(router, "/api/run-mode/permit", &body(staff_id, "1234")).await;

            assert_eq!(json_body(response).await["code"], "NOT_ALLOWED");
        }

        #[tokio::test]
        async fn unapproved_pc_cannot_ask() {
            let (_dir, _db, router) = app_without_staff().await;
            let admin = setup_admin(router.clone()).await;
            let other = crate::server::test_support::from_other_pc(router);

            let response = post(other, "/api/run-mode/permit", &body(admin, "123456")).await;

            assert_eq!(json_body(response).await["code"], "DEVICE_NOT_APPROVED");
        }
    }
}
