//! 과목 저장, 목록, 고치기, 지우기.

use baeum_domain::{CategoryName, NewSubject};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use crate::infra::db::{DeleteError, is_unique_violation, timestamp_text};

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
    now: OffsetDateTime,
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
    now: OffsetDateTime,
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
