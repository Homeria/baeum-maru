//! 우선 배정 사유 저장, 목록, 고치기, 지우기 (BR-S10, BR-S11).

use baeum_domain::{NewPriorityReason, PriorityMethod};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use crate::infra::db::{DeleteError, is_unique_violation, timestamp_text};

/// 사유를 저장하지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum SavePriorityReasonError {
    /// BR-S10: 사유 이름은 겹치지 않는다.
    #[error("같은 이름의 사유가 있습니다")]
    DuplicateName,
    #[error("사유를 저장하지 못했습니다: {0}")]
    Db(sqlx::Error),
}

impl From<sqlx::Error> for SavePriorityReasonError {
    fn from(error: sqlx::Error) -> Self {
        // priority_reasons에서 UNIQUE는 name_key 하나뿐이다
        if is_unique_violation(&error) {
            SavePriorityReasonError::DuplicateName
        } else {
            SavePriorityReasonError::Db(error)
        }
    }
}

/// DB에 저장된 사유 하나.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriorityReasonRecord {
    pub id: i64,
    pub name: String,
    pub method: PriorityMethod,
}

#[derive(sqlx::FromRow)]
struct PriorityReasonRow {
    id: i64,
    name: String,
    method: String,
}

impl TryFrom<PriorityReasonRow> for PriorityReasonRecord {
    type Error = sqlx::Error;

    fn try_from(row: PriorityReasonRow) -> Result<Self, Self::Error> {
        // 마이그레이션의 CHECK가 막아서 모르는 글자는 나오지 않는다
        let method = PriorityMethod::from_code(&row.method).ok_or_else(|| {
            sqlx::Error::Decode(format!("모르는 배정 방식: {}", row.method).into())
        })?;
        Ok(PriorityReasonRecord {
            id: row.id,
            name: row.name,
            method,
        })
    }
}

/// 확인을 통과한 사유를 저장하고 새 `id`를 돌려준다.
pub async fn insert(
    pool: &SqlitePool,
    reason: &NewPriorityReason,
    now: OffsetDateTime,
) -> Result<i64, SavePriorityReasonError> {
    let now = timestamp_text(now);
    let result = sqlx::query(
        "INSERT INTO priority_reasons (name, name_key, method, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(reason.name().as_str())
    .bind(reason.name().key())
    .bind(reason.method().code())
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

/// 사유를 모두 읽는다. 이름 순서.
pub async fn list(pool: &SqlitePool) -> Result<Vec<PriorityReasonRecord>, sqlx::Error> {
    let rows: Vec<PriorityReasonRow> =
        sqlx::query_as("SELECT id, name, method FROM priority_reasons ORDER BY name")
            .fetch_all(pool)
            .await?;
    rows.into_iter()
        .map(PriorityReasonRecord::try_from)
        .collect()
}

/// `id`인 사유의 이름과 방식을 바꾸고 고친 뒤 모습을 돌려준다. 없으면 `None`.
pub async fn update(
    pool: &SqlitePool,
    id: i64,
    reason: &NewPriorityReason,
    now: OffsetDateTime,
) -> Result<Option<PriorityReasonRecord>, SavePriorityReasonError> {
    let row: Option<PriorityReasonRow> = sqlx::query_as(
        "UPDATE priority_reasons SET name = ?, name_key = ?, method = ?, updated_at = ?
         WHERE id = ?
         RETURNING id, name, method",
    )
    .bind(reason.name().as_str())
    .bind(reason.name().key())
    .bind(reason.method().code())
    .bind(timestamp_text(now))
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(PriorityReasonRecord::try_from).transpose()?)
}

/// `id`인 사유를 지운다. 지웠으면 `true`, 없으면 `false`.
///
/// BR-S10: 신청에 한 번이라도 붙었던 사유는 변경 기록의 외래 키가 막아 [`DeleteError::InUse`].
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<bool, DeleteError> {
    let result = sqlx::query("DELETE FROM priority_reasons WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use time::macros::datetime;

    use super::*;
    use crate::db::test_support::{
        NOW, insert_application, insert_course, insert_member, insert_staff, insert_subject,
        insert_term,
    };
    use crate::infra::db::open_temp;

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
