//! 과목 저장, 목록, 고치기, 지우기.

use baeum_domain::{CategoryName, NewSubject};
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

use super::{DeleteError, is_unique_violation, timestamp_text};

/// 과목을 저장하지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum SaveSubjectError {
    /// BR-J1: 과목 이름은 기관 전체에서 겹치지 않는다.
    #[error("이미 있는 과목입니다")]
    DuplicateName,
    #[error("과목을 저장하지 못했습니다: {0}")]
    Db(sqlx::Error),
}

impl From<sqlx::Error> for SaveSubjectError {
    fn from(error: sqlx::Error) -> Self {
        // subjects에서 UNIQUE는 name_key 하나뿐이다
        if is_unique_violation(&error) {
            SaveSubjectError::DuplicateName
        } else {
            SaveSubjectError::Db(error)
        }
    }
}

/// DB에 저장된 과목 하나. 개인정보가 아니라 감싸지 않는다.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct SubjectRecord {
    pub id: i64,
    pub name: String,
    pub category: Option<String>,
}

/// 확인을 통과한 과목을 저장하고 새 `id`를 돌려준다.
pub async fn insert(
    pool: &SqlitePool,
    subject: &NewSubject,
    now: DateTime<Utc>,
) -> Result<i64, SaveSubjectError> {
    let now = timestamp_text(now);
    let result = sqlx::query(
        "INSERT INTO subjects (name, name_key, category, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(subject.name().as_str())
    .bind(subject.name().key())
    .bind(subject.category().map(CategoryName::as_str))
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

/// 과목을 모두 읽는다. 분류(분류 없음은 끝), 이름 순서.
pub async fn list(pool: &SqlitePool) -> Result<Vec<SubjectRecord>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, name, category FROM subjects
         ORDER BY category IS NULL, category, name",
    )
    .fetch_all(pool)
    .await
}

/// `id`인 과목을 읽는다. 없으면 `None`.
pub async fn get(pool: &SqlitePool, id: i64) -> Result<Option<SubjectRecord>, sqlx::Error> {
    sqlx::query_as("SELECT id, name, category FROM subjects WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// `id`인 과목의 칸을 모두 `subject`로 바꾸고 고친 뒤 모습을 돌려준다. 없으면 `None`.
pub async fn update(
    pool: &SqlitePool,
    id: i64,
    subject: &NewSubject,
    now: DateTime<Utc>,
) -> Result<Option<SubjectRecord>, SaveSubjectError> {
    let record = sqlx::query_as(
        "UPDATE subjects SET name = ?, name_key = ?, category = ?, updated_at = ?
         WHERE id = ?
         RETURNING id, name, category",
    )
    .bind(subject.name().as_str())
    .bind(subject.name().key())
    .bind(subject.category().map(CategoryName::as_str))
    .bind(timestamp_text(now))
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(record)
}

/// `id`인 과목을 지운다. 지웠으면 `true`, 없으면 `false`.
///
/// BR-J2: 강좌가 쓰는 과목은 외래 키가 막아 [`DeleteError::InUse`].
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<bool, DeleteError> {
    let result = sqlx::query("DELETE FROM subjects WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use baeum_domain::SubjectInput;
    use chrono::TimeZone;

    use super::*;
    use crate::db::open_temp;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 15, 1, 2, 3).unwrap()
    }

    fn new_subject(name: &str, category: Option<&str>) -> NewSubject {
        NewSubject::parse(SubjectInput { name, category }).unwrap()
    }

    #[tokio::test]
    async fn saves_and_lists_subject() {
        let (_dir, pool) = open_temp().await;

        let id = insert(&pool, &new_subject(" 한글 교실 ", Some("평생교육")), now())
            .await
            .unwrap();

        assert_eq!(
            list(&pool).await.unwrap(),
            [SubjectRecord {
                id,
                name: "한글 교실".to_owned(),
                category: Some("평생교육".to_owned()),
            }]
        );
    }

    #[tokio::test]
    async fn blank_category_is_saved_as_null() {
        let (_dir, pool) = open_temp().await;

        insert(&pool, &new_subject("요가", Some("  ")), now())
            .await
            .unwrap();

        assert_eq!(list(&pool).await.unwrap()[0].category, None);
    }

    #[tokio::test]
    async fn br_j1_name_differing_only_in_spaces_is_told_apart() {
        let (_dir, pool) = open_temp().await;
        insert(&pool, &new_subject("한글교실", None), now())
            .await
            .unwrap();

        let error = insert(&pool, &new_subject("한글 교실", Some("평생교육")), now())
            .await
            .unwrap_err();

        assert!(matches!(error, SaveSubjectError::DuplicateName), "{error}");
    }

    #[tokio::test]
    async fn lists_by_category_then_name() {
        let (_dir, pool) = open_temp().await;
        for (name, category) in [
            ("요가", Some("취미여가")),
            ("상담", None),
            ("한글교실", Some("평생교육")),
            ("댄스", Some("취미여가")),
            ("스마트폰", Some("평생교육")),
        ] {
            insert(&pool, &new_subject(name, category), now())
                .await
                .unwrap();
        }

        let names: Vec<String> = list(&pool)
            .await
            .unwrap()
            .into_iter()
            .map(|subject| subject.name)
            .collect();

        // 취미여가(ㅊ)가 평생교육(ㅍ)보다 앞
        assert_eq!(names, ["댄스", "요가", "스마트폰", "한글교실", "상담"]);
    }

    #[tokio::test]
    async fn get_reads_subject_or_none() {
        let (_dir, pool) = open_temp().await;
        let id = insert(&pool, &new_subject("요가", None), now())
            .await
            .unwrap();

        assert_eq!(get(&pool, id).await.unwrap().unwrap().name, "요가");
        assert!(get(&pool, 999).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn update_replaces_every_field() {
        let (_dir, pool) = open_temp().await;
        let id = insert(&pool, &new_subject("요가", Some("취미여가")), now())
            .await
            .unwrap();

        let subject = update(&pool, id, &new_subject("필라테스", None), now())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            subject,
            SubjectRecord {
                id,
                name: "필라테스".to_owned(),
                category: None,
            }
        );
    }

    #[tokio::test]
    async fn update_can_change_only_spaces_of_own_name() {
        let (_dir, pool) = open_temp().await;
        let id = insert(&pool, &new_subject("한글교실", None), now())
            .await
            .unwrap();

        let subject = update(&pool, id, &new_subject("한글 교실", None), now())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(subject.name, "한글 교실");
    }

    #[tokio::test]
    async fn update_unknown_id_is_none() {
        let (_dir, pool) = open_temp().await;

        let subject = update(&pool, 999, &new_subject("요가", None), now())
            .await
            .unwrap();

        assert!(subject.is_none());
    }

    #[tokio::test]
    async fn br_j1_update_to_taken_name_is_told_apart() {
        let (_dir, pool) = open_temp().await;
        insert(&pool, &new_subject("한글교실", None), now())
            .await
            .unwrap();
        let id = insert(&pool, &new_subject("요가", None), now())
            .await
            .unwrap();

        let error = update(&pool, id, &new_subject("한글 교실", None), now())
            .await
            .unwrap_err();

        assert!(matches!(error, SaveSubjectError::DuplicateName), "{error}");
    }

    #[tokio::test]
    async fn delete_removes_subject() {
        let (_dir, pool) = open_temp().await;
        let id = insert(&pool, &new_subject("요가", None), now())
            .await
            .unwrap();

        assert!(delete(&pool, id).await.unwrap());
        assert!(!delete(&pool, id).await.unwrap());
        assert!(list(&pool).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn br_j2_subject_used_by_course_is_in_use() {
        let (_dir, pool) = open_temp().await;
        let id = insert(&pool, &new_subject("요가", None), now())
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO terms (name, starts_on, ends_on, created_at, updated_at)
             VALUES ('2027년 1학기', '2027-03-02', '2027-06-30', '', '')",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO courses (term_id, subject_id, capacity, created_at, updated_at)
             VALUES ((SELECT id FROM terms), ?, 20, '', '')",
        )
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();

        let error = delete(&pool, id).await.unwrap_err();

        assert!(matches!(error, DeleteError::InUse), "{error}");
    }
}
