//! 회원 저장.

use baeum_domain::{Gender, NewMember};
use chrono::{DateTime, SecondsFormat, Utc};
use sqlx::SqlitePool;

/// 회원을 저장하지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum InsertMemberError {
    /// BR-M2: 회원번호는 겹치지 않는다.
    #[error("이미 있는 회원번호입니다")]
    DuplicateMemberNo,
    #[error("회원을 저장하지 못했습니다: {0}")]
    Db(sqlx::Error),
}

impl From<sqlx::Error> for InsertMemberError {
    fn from(error: sqlx::Error) -> Self {
        // members에서 UNIQUE는 member_no 하나뿐이다
        let is_unique_violation = error
            .as_database_error()
            .is_some_and(|db_error| db_error.is_unique_violation());
        if is_unique_violation {
            InsertMemberError::DuplicateMemberNo
        } else {
            InsertMemberError::Db(error)
        }
    }
}

/// 확인을 통과한 새 회원을 저장하고 새 `id`를 돌려준다.
///
/// `now`는 `created_at`과 `updated_at`에 같이 들어간다.
pub async fn insert(
    pool: &SqlitePool,
    member: &NewMember,
    now: DateTime<Utc>,
) -> Result<i64, InsertMemberError> {
    // [docs/conventions.md]: 밀리초까지, 끝에 Z
    let now = now.to_rfc3339_opts(SecondsFormat::Millis, true);
    let result = sqlx::query(
        "INSERT INTO members
             (member_no, name, phone_number, birth_date, gender, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(member.member_no().expose().as_str())
    .bind(member.name().expose().as_str())
    .bind(member.phone_number().expose().as_str())
    .bind(member.birth_date().map(|date| *date.expose()))
    .bind(member.gender().map(|gender| gender_text(*gender.expose())))
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

/// DB의 `gender` 칸에 쓰는 글자.
fn gender_text(gender: Gender) -> &'static str {
    match gender {
        Gender::Male => "male",
        Gender::Female => "female",
    }
}

#[cfg(test)]
mod tests {
    use baeum_domain::MemberInput;
    use chrono::{NaiveDate, TimeZone};

    use super::*;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 15).unwrap()
    }

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 15, 1, 2, 3).unwrap()
    }

    fn input(member_no: &str) -> MemberInput<'_> {
        MemberInput {
            member_no,
            name: "회원1",
            phone_number: "010-0000-0001",
            birth_date: None,
            gender: None,
        }
    }

    fn new_member(input: MemberInput<'_>) -> NewMember {
        NewMember::parse(input, today()).unwrap()
    }

    async fn open() -> (tempfile::TempDir, SqlitePool) {
        let dir = tempfile::tempdir().unwrap();
        let pool = crate::db::open(dir.path()).await.unwrap();
        (dir, pool)
    }

    type Row = (
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        String,
        String,
    );

    async fn row(pool: &SqlitePool, id: i64) -> Row {
        sqlx::query_as(
            "SELECT member_no, name, phone_number, birth_date, gender, created_at, updated_at
             FROM members WHERE id = ?",
        )
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn saves_all_fields() {
        let (_dir, pool) = open().await;
        let member = new_member(MemberInput {
            birth_date: NaiveDate::from_ymd_opt(1950, 3, 1),
            gender: Some(Gender::Female),
            ..input("A-001")
        });

        let id = insert(&pool, &member, now()).await.unwrap();

        let (member_no, name, phone_number, birth_date, gender, created_at, updated_at) =
            row(&pool, id).await;
        assert_eq!(member_no, "A-001");
        assert_eq!(name, "회원1");
        assert_eq!(phone_number, "01000000001");
        assert_eq!(birth_date.as_deref(), Some("1950-03-01"));
        assert_eq!(gender.as_deref(), Some("female"));
        assert_eq!(created_at, "2026-11-15T01:02:03.000Z");
        assert_eq!(updated_at, created_at);
    }

    #[tokio::test]
    async fn br_m4_saves_without_optional_fields() {
        let (_dir, pool) = open().await;

        let id = insert(&pool, &new_member(input("00001")), now())
            .await
            .unwrap();

        let (_, _, _, birth_date, gender, _, _) = row(&pool, id).await;
        assert_eq!(birth_date, None);
        assert_eq!(gender, None);
    }

    #[tokio::test]
    async fn saves_male_as_text() {
        let (_dir, pool) = open().await;
        let member = new_member(MemberInput {
            gender: Some(Gender::Male),
            ..input("00001")
        });

        let id = insert(&pool, &member, now()).await.unwrap();

        let (_, _, _, _, gender, _, _) = row(&pool, id).await;
        assert_eq!(gender.as_deref(), Some("male"));
    }

    #[tokio::test]
    async fn br_m2_duplicate_member_no_is_told_apart() {
        let (_dir, pool) = open().await;
        insert(&pool, &new_member(input("00001")), now())
            .await
            .unwrap();

        let error = insert(&pool, &new_member(input("00001")), now())
            .await
            .unwrap_err();

        assert!(
            matches!(error, InsertMemberError::DuplicateMemberNo),
            "{error}"
        );
    }

    #[tokio::test]
    async fn gives_new_id_each_time() {
        let (_dir, pool) = open().await;

        let first = insert(&pool, &new_member(input("00001")), now())
            .await
            .unwrap();
        let second = insert(&pool, &new_member(input("00002")), now())
            .await
            .unwrap();

        assert_ne!(first, second);
    }
}
