//! 우선 배정 사유 테스트.

mod handler {
    use axum::http::StatusCode;
    use serde_json::json;

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

mod repository {
    use baeum_domain::{NewPriorityReason, PriorityMethod};
    use time::OffsetDateTime;
    use time::macros::datetime;

    use crate::db::test_support::{
        NOW, insert_application, insert_course, insert_member, insert_staff, insert_subject,
        insert_term,
    };
    use crate::features::priority_reasons::error::*;
    use crate::features::priority_reasons::repository::*;
    use crate::infra::db::{DeleteError, open_temp};

    fn now() -> OffsetDateTime {
        datetime!(2026-11-22 01:02:03 UTC)
    }

    fn reason(name: &str) -> NewPriorityReason {
        NewPriorityReason::parse(name, PriorityMethod::Guaranteed).unwrap()
    }

    #[tokio::test]
    async fn saves_and_lists_by_name() {
        let (_dir, pool) = open_temp().await;
        let leader = insert(&pool, &reason(" 반장 "), now()).await.unwrap();
        let veteran = insert(&pool, &reason("국가유공자"), now()).await.unwrap();

        assert_eq!(
            list(&pool).await.unwrap(),
            [
                PriorityReasonRecord {
                    id: veteran,
                    name: "국가유공자".to_owned(),
                    method: PriorityMethod::Guaranteed,
                },
                PriorityReasonRecord {
                    id: leader,
                    name: "반장".to_owned(),
                    method: PriorityMethod::Guaranteed,
                },
            ]
        );
    }

    #[tokio::test]
    async fn br_s10_name_differing_only_in_spaces_is_told_apart() {
        let (_dir, pool) = open_temp().await;
        insert(&pool, &reason("장애인 우대"), now()).await.unwrap();

        let error = insert(&pool, &reason("장애인우대"), now())
            .await
            .unwrap_err();

        assert!(
            matches!(error, SavePriorityReasonError::DuplicateName),
            "{error}"
        );
    }

    #[tokio::test]
    async fn update_renames_or_is_none() {
        let (_dir, pool) = open_temp().await;
        let id = insert(&pool, &reason("반장"), now()).await.unwrap();

        let renamed = update(&pool, id, &reason("지난 학기 반장"), now())
            .await
            .unwrap()
            .unwrap();
        let unknown = update(&pool, 999, &reason("반장"), now()).await.unwrap();

        assert_eq!(renamed.name, "지난 학기 반장");
        assert!(unknown.is_none());
    }

    #[tokio::test]
    async fn br_s10_update_to_taken_name_is_told_apart() {
        let (_dir, pool) = open_temp().await;
        insert(&pool, &reason("반장"), now()).await.unwrap();
        let id = insert(&pool, &reason("국가유공자"), now()).await.unwrap();

        let error = update(&pool, id, &reason("반 장"), now())
            .await
            .unwrap_err();

        assert!(
            matches!(error, SavePriorityReasonError::DuplicateName),
            "{error}"
        );
    }

    #[tokio::test]
    async fn deletes_unused_reason_or_tells_unknown() {
        let (_dir, pool) = open_temp().await;
        let id = insert(&pool, &reason("반장"), now()).await.unwrap();

        assert!(delete(&pool, id).await.unwrap());
        assert!(!delete(&pool, id).await.unwrap());
    }

    #[tokio::test]
    async fn br_s10_reason_once_attached_cannot_be_deleted() {
        let (_dir, pool) = open_temp().await;
        let id = insert(&pool, &reason("반장"), now()).await.unwrap();
        let term_id = insert_term(&pool, "2027년 1학기", "2027-03-01", "2027-06-30")
            .await
            .unwrap();
        let subject_id = insert_subject(&pool, "요가", "요가", None).await.unwrap();
        let course_id = insert_course(&pool, term_id, subject_id, None, None)
            .await
            .unwrap();
        let member_id = insert_member(&pool, "00001").await.unwrap();
        let staff_id = insert_staff(&pool, "직원1", "staff", None).await.unwrap();
        let application_id = insert_application(&pool, member_id, course_id, "submitted", staff_id)
            .await
            .unwrap();
        // 붙였다가 뗀 사유: 신청에는 없고 변경 기록만 남았다
        sqlx::query(
            "INSERT INTO priority_changes
             (application_id, member_id, course_id, reason_id, change, changed_by, changed_at)
         VALUES (?, ?, ?, ?, 'added', ?, ?), (?, ?, ?, ?, 'removed', ?, ?)",
        )
        .bind(application_id)
        .bind(member_id)
        .bind(course_id)
        .bind(id)
        .bind(staff_id)
        .bind(NOW)
        .bind(application_id)
        .bind(member_id)
        .bind(course_id)
        .bind(id)
        .bind(staff_id)
        .bind(NOW)
        .execute(&pool)
        .await
        .unwrap();

        let error = delete(&pool, id).await.unwrap_err();

        assert!(matches!(error, DeleteError::InUse), "{error}");
    }
}
