//! 학기 저장, 목록, 고치기, 지우기.

use baeum_domain::NewTerm;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::SqlitePool;

use super::{DeleteError, is_unique_violation, timestamp_text};

/// 학기를 저장하지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum SaveTermError {
    /// BR-T1: 학기 이름은 겹치지 않는다.
    #[error("같은 이름의 학기가 있습니다")]
    DuplicateName,
    #[error("학기를 저장하지 못했습니다: {0}")]
    Db(sqlx::Error),
}

impl From<sqlx::Error> for SaveTermError {
    fn from(error: sqlx::Error) -> Self {
        // terms에서 UNIQUE는 name 하나뿐이다
        if is_unique_violation(&error) {
            SaveTermError::DuplicateName
        } else {
            SaveTermError::Db(error)
        }
    }
}

/// DB에 저장된 학기 하나.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct TermRecord {
    pub id: i64,
    pub name: String,
    pub starts_on: NaiveDate,
    pub ends_on: NaiveDate,
}

/// 확인을 통과한 학기를 저장하고 새 `id`를 돌려준다.
pub async fn insert(
    pool: &SqlitePool,
    term: &NewTerm,
    now: DateTime<Utc>,
) -> Result<i64, SaveTermError> {
    let now = timestamp_text(now);
    let result = sqlx::query(
        "INSERT INTO terms (name, starts_on, ends_on, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(term.name().as_str())
    .bind(term.period().starts_on())
    .bind(term.period().ends_on())
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

/// 학기를 모두 읽는다. 시작일이 늦은 학기부터 (가장 최근 학기가 맨 앞).
pub async fn list(pool: &SqlitePool) -> Result<Vec<TermRecord>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, name, starts_on, ends_on FROM terms
         ORDER BY starts_on DESC, id DESC",
    )
    .fetch_all(pool)
    .await
}

/// `id`인 학기를 읽는다. 없으면 `None`.
pub async fn get(pool: &SqlitePool, id: i64) -> Result<Option<TermRecord>, sqlx::Error> {
    sqlx::query_as("SELECT id, name, starts_on, ends_on FROM terms WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// `id`인 학기의 칸을 모두 `term`으로 바꾸고 고친 뒤 모습을 돌려준다. 없으면 `None`.
pub async fn update(
    pool: &SqlitePool,
    id: i64,
    term: &NewTerm,
    now: DateTime<Utc>,
) -> Result<Option<TermRecord>, SaveTermError> {
    let record = sqlx::query_as(
        "UPDATE terms SET name = ?, starts_on = ?, ends_on = ?, updated_at = ?
         WHERE id = ?
         RETURNING id, name, starts_on, ends_on",
    )
    .bind(term.name().as_str())
    .bind(term.period().starts_on())
    .bind(term.period().ends_on())
    .bind(timestamp_text(now))
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(record)
}

/// `id`인 학기를 지운다. 지웠으면 `true`, 없으면 `false`.
///
/// BR-T5: 강좌가 있는 학기는 외래 키가 막아 [`DeleteError::InUse`].
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<bool, DeleteError> {
    let result = sqlx::query("DELETE FROM terms WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use baeum_domain::TermInput;
    use chrono::TimeZone;

    use super::*;
    use crate::db::open_temp;
    use crate::db::test_support::{insert_course, insert_subject};

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 15, 1, 2, 3).unwrap()
    }

    fn date(text: &str) -> NaiveDate {
        text.parse().unwrap()
    }

    fn new_term(name: &str, starts_on: &str, ends_on: &str) -> NewTerm {
        NewTerm::parse(TermInput {
            name,
            starts_on: Some(date(starts_on)),
            ends_on: Some(date(ends_on)),
        })
        .unwrap()
    }

    #[tokio::test]
    async fn saves_and_lists_term() {
        let (_dir, pool) = open_temp().await;

        let id = insert(
            &pool,
            &new_term("2027년 1학기", "2027-03-02", "2027-06-30"),
            now(),
        )
        .await
        .unwrap();

        assert_eq!(
            list(&pool).await.unwrap(),
            [TermRecord {
                id,
                name: "2027년 1학기".to_owned(),
                starts_on: date("2027-03-02"),
                ends_on: date("2027-06-30"),
            }]
        );
    }

    #[tokio::test]
    async fn lists_latest_term_first() {
        let (_dir, pool) = open_temp().await;
        for (name, starts_on, ends_on) in [
            ("2027년 1학기", "2027-03-02", "2027-06-30"),
            ("2027년 2학기", "2027-09-01", "2027-12-31"),
            ("2026년 2학기", "2026-09-01", "2026-12-31"),
        ] {
            insert(&pool, &new_term(name, starts_on, ends_on), now())
                .await
                .unwrap();
        }

        let names: Vec<String> = list(&pool)
            .await
            .unwrap()
            .into_iter()
            .map(|term| term.name)
            .collect();

        assert_eq!(names, ["2027년 2학기", "2027년 1학기", "2026년 2학기"]);
    }

    #[tokio::test]
    async fn br_t1_duplicate_name_is_told_apart() {
        let (_dir, pool) = open_temp().await;
        insert(
            &pool,
            &new_term("2027년 1학기", "2027-03-02", "2027-06-30"),
            now(),
        )
        .await
        .unwrap();

        let error = insert(
            &pool,
            &new_term("2027년 1학기", "2027-09-01", "2027-12-31"),
            now(),
        )
        .await
        .unwrap_err();

        assert!(matches!(error, SaveTermError::DuplicateName), "{error}");
    }

    #[tokio::test]
    async fn get_update_and_unknown_id() {
        let (_dir, pool) = open_temp().await;
        let id = insert(
            &pool,
            &new_term("2027년 1학기", "2027-03-02", "2027-06-30"),
            now(),
        )
        .await
        .unwrap();

        let term = update(
            &pool,
            id,
            &new_term("2027년 봄학기", "2027-03-04", "2027-06-27"),
            now(),
        )
        .await
        .unwrap()
        .unwrap();

        assert_eq!(get(&pool, id).await.unwrap(), Some(term.clone()));
        assert_eq!(term.name, "2027년 봄학기");
        assert_eq!(term.starts_on, date("2027-03-04"));
        assert!(get(&pool, 999).await.unwrap().is_none());
        assert!(
            update(
                &pool,
                999,
                &new_term("없음", "2027-03-02", "2027-06-30"),
                now()
            )
            .await
            .unwrap()
            .is_none()
        );
    }

    #[tokio::test]
    async fn deletes_term_without_course() {
        let (_dir, pool) = open_temp().await;
        let id = insert(
            &pool,
            &new_term("2027년 1학기", "2027-03-02", "2027-06-30"),
            now(),
        )
        .await
        .unwrap();

        assert!(delete(&pool, id).await.unwrap());
        assert!(!delete(&pool, id).await.unwrap());
    }

    #[tokio::test]
    async fn br_t5_term_with_course_is_in_use() {
        let (_dir, pool) = open_temp().await;
        let id = insert(
            &pool,
            &new_term("2027년 1학기", "2027-03-02", "2027-06-30"),
            now(),
        )
        .await
        .unwrap();
        let subject_id = insert_subject(&pool, "요가", "요가", None).await.unwrap();
        insert_course(&pool, id, subject_id, None, None)
            .await
            .unwrap();

        let error = delete(&pool, id).await.unwrap_err();

        assert!(matches!(error, DeleteError::InUse), "{error}");
    }
}
