//! 백업 요청 처리 테스트. 백업 만들기·검사·복원의 테스트는 그 파일 안에 있다.

mod handler {
    use axum::http::StatusCode;
    use serde_json::{Value, json};

    use time::macros::datetime;

    use crate::db::test_support::{insert_course, insert_subject, insert_term};
    use crate::features::backups::backup::Backups;
    use crate::infra::backup_file::name::BackupKind;
    use crate::server::test_support::{app, app_as, from_approved_pc, get, json_body, post, put};

    #[tokio::test]
    async fn only_admin_manages_backups() {
        for role in ["staff", "temp_staff", "read_only"] {
            let (_dir, _db, router) = app_as(role).await;

            let list = get(router.clone(), "/api/backups").await;
            let create = post(router, "/api/backups", "{}").await;

            assert_eq!(list.status(), StatusCode::FORBIDDEN, "{role}");
            assert_eq!(create.status(), StatusCode::FORBIDDEN, "{role}");
        }
    }

    #[tokio::test]
    async fn backup_now_shows_in_the_list() {
        let (_dir, _db, router) = app().await;

        let created = post(router.clone(), "/api/backups", "{}").await;
        assert_eq!(created.status(), StatusCode::CREATED);
        let created = json_body(created).await;
        let list = json_body(get(router, "/api/backups").await).await;

        assert_eq!(created["kind"], "manual");
        assert!(created["size"].as_u64().unwrap() > 0);
        assert_eq!(list["items"][0]["fileName"], created["fileName"]);
        assert_eq!(list["lastFailure"], Value::Null);
        assert_eq!(
            list["secondLocation"],
            json!({ "path": null, "reachable": false, "lastCopiedAt": null, "changeable": true })
        );
    }

    #[tokio::test]
    async fn lottery_backup_shows_course_name() {
        let (dir, db, router) = app().await;
        let term_id = insert_term(&db, "2027년 1학기", "2027-03-01", "2027-06-30")
            .await
            .unwrap();
        let subject_id = insert_subject(&db, "요가", "요가", None).await.unwrap();
        let course_id = insert_course(&db, term_id, subject_id, Some("초급"), None)
            .await
            .unwrap();
        Backups::new(db, dir.path())
            .make(
                BackupKind::BeforeLottery { course_id },
                datetime!(2026-11-23 01:00 UTC),
            )
            .await
            .unwrap();

        let list = json_body(get(router, "/api/backups").await).await;

        assert_eq!(list["items"][0]["kind"], "before_lottery");
        assert_eq!(list["items"][0]["courseId"], course_id);
        assert_eq!(list["items"][0]["courseName"], "요가 초급");
    }

    #[tokio::test]
    async fn sets_and_clears_second_location_on_host() {
        let (_dir, _db, router) = app().await;
        let second = tempfile::tempdir().unwrap();
        let path = second.path().display().to_string();

        let set = put(
            router.clone(),
            "/api/backups/second-location",
            &json!({ "path": path }).to_string(),
        )
        .await;
        assert_eq!(set.status(), StatusCode::OK);
        assert_eq!(json_body(set).await["path"], path.as_str());

        let cleared = put(
            router.clone(),
            "/api/backups/second-location",
            &json!({ "path": null }).to_string(),
        )
        .await;
        assert_eq!(json_body(cleared).await["path"], Value::Null);
    }

    #[tokio::test]
    async fn missing_folder_is_invalid_input() {
        let (dir, _db, router) = app().await;
        let missing = dir.path().join("없는 폴더").display().to_string();

        let response = put(
            router,
            "/api/backups/second-location",
            &json!({ "path": missing }).to_string(),
        )
        .await;

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = json_body(response).await;
        assert_eq!(body["fields"][0]["field"], "path");
    }

    #[tokio::test]
    async fn other_pc_sees_but_cannot_change_second_location() {
        let (_dir, db, router) = app().await;
        let router = from_approved_pc(&db, router).await;
        let second = tempfile::tempdir().unwrap();

        let list = json_body(get(router.clone(), "/api/backups").await).await;
        let response = put(
            router,
            "/api/backups/second-location",
            &json!({ "path": second.path().display().to_string() }).to_string(),
        )
        .await;

        assert_eq!(list["secondLocation"]["changeable"], false);
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        assert_eq!(json_body(response).await["code"], "HOST_ONLY");
    }

    async fn first_backup(router: axum::Router) -> String {
        let created = json_body(post(router, "/api/backups", "{}").await).await;
        created["fileName"].as_str().unwrap().to_owned()
    }

    #[tokio::test]
    async fn previews_backup_from_list_and_from_other_place() {
        let (dir, db, router) = app().await;
        insert_term(&db, "2027년 1학기", "2027-03-01", "2027-06-30")
            .await
            .unwrap();
        let file_name = first_backup(router.clone()).await;
        let elsewhere = tempfile::tempdir().unwrap();
        let copied = elsewhere.path().join("옛 백업.db");
        tokio::fs::copy(
            crate::infra::backup_file::dir(dir.path()).join(&file_name),
            &copied,
        )
        .await
        .unwrap();

        let from_list = post(
            router.clone(),
            "/api/backups/restore-preview",
            &json!({ "fileName": file_name }).to_string(),
        )
        .await;
        let from_path = post(
            router,
            "/api/backups/restore-preview",
            &json!({ "path": copied.display().to_string() }).to_string(),
        )
        .await;

        assert_eq!(from_list.status(), StatusCode::OK);
        let from_list = json_body(from_list).await;
        assert_eq!(from_list["kind"], "manual");
        assert_eq!(from_list["termNames"], json!(["2027년 1학기"]));
        assert_eq!(from_list["memberCount"], 0);
        assert_eq!(from_list["applicationCount"], 0);
        let from_path = json_body(from_path).await;
        // 이름을 바꾼 파일은 시각을 알 수 없다
        assert_eq!(from_path["takenAt"], Value::Null);
        assert_eq!(from_path["termNames"], json!(["2027년 1학기"]));
    }

    #[tokio::test]
    async fn restore_is_prepared_for_next_start() {
        let (dir, _db, router) = app().await;
        let file_name = first_backup(router.clone()).await;

        let response = post(
            router.clone(),
            "/api/backups/restore",
            &json!({ "fileName": file_name }).to_string(),
        )
        .await;

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert!(
            tokio::fs::try_exists(dir.path().join("restore-pending.db"))
                .await
                .unwrap()
        );
        let list = json_body(get(router, "/api/backups").await).await;
        assert_eq!(list["items"][0]["kind"], "before_restore");
        assert_eq!(list["lastRestore"], Value::Null);
    }

    #[tokio::test]
    async fn unusable_sources_are_invalid_input() {
        let (dir, _db, router) = app().await;
        let text = dir.path().join("메모.db");
        tokio::fs::write(&text, b"not a database").await.unwrap();
        let cases = [
            (
                json!({ "fileName": "../baeum-maru.db" }),
                "fileName",
                "백업 목록에 없는 파일",
            ),
            (
                json!({ "fileName": "20261123-090000_daily.db" }),
                "fileName",
                "백업 파일이 없습니다",
            ),
            (json!({ "path": "백업.db" }), "path", "드라이브부터"),
            (
                json!({ "path": text.display().to_string() }),
                "path",
                "DB 파일이 아닙니다",
            ),
        ];
        for (body, field, message) in cases {
            let response = post(router.clone(), "/api/backups/restore", &body.to_string()).await;

            assert_eq!(
                response.status(),
                StatusCode::UNPROCESSABLE_ENTITY,
                "{body}"
            );
            let error = json_body(response).await;
            assert_eq!(error["fields"][0]["field"], field, "{body}");
            assert!(
                error["fields"][0]["message"]
                    .as_str()
                    .unwrap()
                    .contains(message),
                "{body}: {error}"
            );
        }
        let both = post(
            router,
            "/api/backups/restore",
            &json!({ "fileName": "a", "path": "b" }).to_string(),
        )
        .await;
        assert_eq!(both.status(), StatusCode::BAD_REQUEST);
        assert!(
            !tokio::fs::try_exists(dir.path().join("restore-pending.db"))
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn only_admin_on_host_restores() {
        let (_dir, _db, router) = app_as("staff").await;
        let staff = post(router, "/api/backups/restore", r#"{"fileName":"x"}"#).await;
        assert_eq!(staff.status(), StatusCode::FORBIDDEN);

        let (_dir, db, router) = app().await;
        let file_name = first_backup(router.clone()).await;
        let router = from_approved_pc(&db, router).await;
        for uri in ["/api/backups/restore-preview", "/api/backups/restore"] {
            let response = post(
                router.clone(),
                uri,
                &json!({ "fileName": file_name }).to_string(),
            )
            .await;

            assert_eq!(response.status(), StatusCode::FORBIDDEN, "{uri}");
            assert_eq!(json_body(response).await["code"], "HOST_ONLY");
        }
    }
}
