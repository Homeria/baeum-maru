//! 처음 설정 테스트.

mod handler {
    use axum::http::StatusCode;
    use baeum_domain::RecoveryCode;
    use serde_json::json;

    use crate::infra::secret;
    use crate::server::test_support::{app_without_staff, get, json_body, post};

    fn admin(name: &str, pin: &str) -> String {
        json!({ "name": name, "pin": pin }).to_string()
    }

    async fn setup_needed(router: axum::Router) -> bool {
        let response = get(router, "/api/setup").await;
        assert_eq!(response.status(), StatusCode::OK);
        json_body(response).await["needed"].as_bool().unwrap()
    }

    #[tokio::test]
    async fn new_db_needs_setup() {
        let (_dir, _db, router) = app_without_staff().await;

        assert!(setup_needed(router).await);
    }

    #[tokio::test]
    async fn creates_admin_and_shows_recovery_code_once() {
        let (_dir, db, router) = app_without_staff().await;

        let response = post(router.clone(), "/api/setup", &admin("관리자", "123456")).await;

        assert_eq!(response.status(), StatusCode::CREATED);
        let shown = json_body(response).await["recoveryCode"]
            .as_str()
            .unwrap()
            .to_owned();
        let code = RecoveryCode::parse(&shown).unwrap();
        assert_eq!(code.to_string(), shown);
        assert!(!setup_needed(router).await);

        // DB에는 해시만 있고, 그 해시가 보여 준 코드·넣은 PIN과 맞는다
        let (pin_hash, role): (String, String) = sqlx::query_as("SELECT pin_hash, role FROM staff")
            .fetch_one(&db)
            .await
            .unwrap();
        assert_eq!(role, "admin");
        assert!(secret::verify("123456", &pin_hash));
        let code_hash: String = sqlx::query_scalar("SELECT code_hash FROM recovery_code")
            .fetch_one(&db)
            .await
            .unwrap();
        assert!(secret::verify(code.as_str(), &code_hash));
    }

    #[tokio::test]
    async fn setup_runs_only_once() {
        let (_dir, _db, router) = app_without_staff().await;
        let first = post(router.clone(), "/api/setup", &admin("관리자", "123456")).await;
        assert_eq!(first.status(), StatusCode::CREATED);

        let response = post(router, "/api/setup", &admin("다른관리자", "654321")).await;

        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = json_body(response).await;
        assert_eq!(body["code"], "SETUP_ALREADY_DONE");
        assert_eq!(body["rule"], "BR-K11");
    }

    #[tokio::test]
    async fn rejects_empty_name_and_short_admin_pin() {
        let (_dir, _db, router) = app_without_staff().await;

        let response = post(router.clone(), "/api/setup", &admin(" ", "1234")).await;

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = json_body(response).await;
        assert_eq!(body["fields"][0]["field"], "name");
        assert_eq!(body["fields"][1]["field"], "pin");
        assert_eq!(
            body["fields"][1]["message"],
            "관리자 PIN은 6자리 이상입니다"
        );
        // 넣은 PIN은 응답에 다시 싣지 않는다
        assert!(!body.to_string().contains("1234"));
        assert!(setup_needed(router).await);
    }

    #[tokio::test]
    async fn rejects_malformed_body() {
        let (_dir, _db, router) = app_without_staff().await;

        let response = post(router, "/api/setup", r#"{"name":"관리자"}"#).await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}

mod repository {
    use baeum_domain::StaffName;
    use time::OffsetDateTime;
    use time::macros::datetime;

    use crate::db::test_support::insert_staff;
    use crate::features::setup::repository::*;
    use crate::infra::db::open_temp;

    fn now() -> OffsetDateTime {
        datetime!(2026-11-15 01:02:03 UTC)
    }

    fn name(input: &str) -> StaffName {
        StaffName::parse(input).unwrap()
    }

    #[tokio::test]
    async fn setup_is_needed_only_until_first_staff() {
        let (_dir, pool) = open_temp().await;
        assert!(is_setup_needed(&pool).await.unwrap());

        insert_staff(&pool, "직원1", "staff", None).await.unwrap();

        assert!(!is_setup_needed(&pool).await.unwrap());
    }

    #[tokio::test]
    async fn setup_saves_admin_and_recovery_code() {
        let (_dir, pool) = open_temp().await;

        let id = setup(&pool, &name(" 관리 자 "), "pin-hash", "code-hash", now())
            .await
            .unwrap();

        let (staff_name, name_key, pin_hash, role, valid_until): (
            String,
            String,
            String,
            String,
            Option<String>,
        ) = sqlx::query_as(
            "SELECT name, name_key, pin_hash, role, valid_until FROM staff WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(staff_name, "관리 자");
        assert_eq!(name_key, "관리자");
        assert_eq!(pin_hash, "pin-hash");
        assert_eq!(role, "admin");
        assert_eq!(valid_until, None);

        let code_hash: String = sqlx::query_scalar("SELECT code_hash FROM recovery_code")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(code_hash, "code-hash");
        assert!(!is_setup_needed(&pool).await.unwrap());
    }

    #[tokio::test]
    async fn setup_runs_only_once() {
        let (_dir, pool) = open_temp().await;
        setup(&pool, &name("관리자"), "pin-hash", "code-hash", now())
            .await
            .unwrap();

        let result = setup(&pool, &name("다른관리자"), "pin-2", "code-2", now()).await;

        assert!(matches!(result, Err(SetupError::AlreadyDone)), "{result:?}");
        let staff_count: i64 = sqlx::query_scalar("SELECT count(*) FROM staff")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(staff_count, 1);
        let code_hash: String = sqlx::query_scalar("SELECT code_hash FROM recovery_code")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(code_hash, "code-hash");
    }
}
