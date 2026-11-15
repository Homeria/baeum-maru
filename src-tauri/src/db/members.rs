//! 회원 저장, 찾기, 고치기, 지우기.

use baeum_domain::{Gender, NewMember, Personal};
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::SqlitePool;

use super::timestamp_text;

/// 회원을 저장하거나 고치지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum SaveMemberError {
    /// BR-M2: 회원번호는 겹치지 않는다.
    #[error("이미 있는 회원번호입니다")]
    DuplicateMemberNo,
    #[error("회원을 저장하지 못했습니다: {0}")]
    Db(sqlx::Error),
}

impl From<sqlx::Error> for SaveMemberError {
    fn from(error: sqlx::Error) -> Self {
        // members에서 UNIQUE는 member_no 하나뿐이다
        let is_unique_violation = error
            .as_database_error()
            .is_some_and(|db_error| db_error.is_unique_violation());
        if is_unique_violation {
            SaveMemberError::DuplicateMemberNo
        } else {
            SaveMemberError::Db(error)
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
) -> Result<i64, SaveMemberError> {
    let now = timestamp_text(now);
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

/// [`gender_text`]의 반대. CHECK 제약이 있어 다른 글자는 나오지 않는다.
fn gender_from_text(text: &str) -> Result<Gender, sqlx::Error> {
    match text {
        "male" => Ok(Gender::Male),
        "female" => Ok(Gender::Female),
        _ => Err(sqlx::Error::Decode("알 수 없는 성별".into())),
    }
}

/// DB에 저장된 회원 한 명.
#[derive(Debug)]
pub struct MemberRecord {
    pub id: i64,
    pub member_no: Personal<String>,
    pub name: Personal<String>,
    pub phone_number: Personal<String>,
    pub birth_date: Option<Personal<NaiveDate>>,
    pub gender: Option<Personal<Gender>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// SELECT·RETURNING에서 이 순서로 칸을 읽는다.
type MemberRow = (
    i64,
    String,
    String,
    String,
    Option<NaiveDate>,
    Option<String>,
    DateTime<Utc>,
    DateTime<Utc>,
);

impl TryFrom<MemberRow> for MemberRecord {
    type Error = sqlx::Error;

    fn try_from(row: MemberRow) -> Result<Self, Self::Error> {
        let (id, member_no, name, phone_number, birth_date, gender, created_at, updated_at) = row;
        let gender = gender.as_deref().map(gender_from_text).transpose()?;
        Ok(MemberRecord {
            id,
            member_no: Personal::new(member_no),
            name: Personal::new(name),
            phone_number: Personal::new(phone_number),
            birth_date: birth_date.map(Personal::new),
            gender: gender.map(Personal::new),
            created_at,
            updated_at,
        })
    }
}

/// 찾은 회원 한 쪽과 조건에 맞는 전체 수.
#[derive(Debug)]
pub struct MemberPage {
    pub items: Vec<MemberRecord>,
    pub total: i64,
}

/// 검색어로 회원을 찾는다. 이름순, 같으면 회원번호순.
///
/// 검색어는 이름과 회원번호의 일부, 연락처 숫자의 일부와 맞춰 본다.
/// 빈 검색어는 모두 찾는다. `page`는 1부터 센다.
pub async fn search(
    pool: &SqlitePool,
    query: &str,
    page: u32,
    page_size: u32,
) -> Result<MemberPage, sqlx::Error> {
    let text_pattern = format!("%{}%", escape_like(query.trim()));
    let phone_pattern = phone_digits(query).map(|digits| format!("%{digits}%"));
    let offset = i64::from(page.saturating_sub(1)) * i64::from(page_size);

    // ?1 이름·회원번호 패턴, ?2 연락처 패턴(숫자 검색어가 아니면 NULL)
    // sqlx는 고정 글자 SQL만 받으므로 아래 COUNT와 WHERE를 똑같이 두 번 씀
    let rows: Vec<MemberRow> = sqlx::query_as(
        r"SELECT id, member_no, name, phone_number, birth_date, gender, created_at, updated_at
          FROM members
          WHERE name LIKE ?1 ESCAPE '\' OR member_no LIKE ?1 ESCAPE '\'
             OR (?2 IS NOT NULL AND phone_number LIKE ?2)
          ORDER BY name, member_no
          LIMIT ?3 OFFSET ?4",
    )
    .bind(&text_pattern)
    .bind(&phone_pattern)
    .bind(i64::from(page_size))
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let total: i64 = sqlx::query_scalar(
        r"SELECT COUNT(*)
          FROM members
          WHERE name LIKE ?1 ESCAPE '\' OR member_no LIKE ?1 ESCAPE '\'
             OR (?2 IS NOT NULL AND phone_number LIKE ?2)",
    )
    .bind(&text_pattern)
    .bind(&phone_pattern)
    .fetch_one(pool)
    .await?;

    let items = rows
        .into_iter()
        .map(MemberRecord::try_from)
        .collect::<Result<_, _>>()?;
    Ok(MemberPage { items, total })
}

/// `id`인 회원을 읽는다. 없으면 `None`.
pub async fn get(pool: &SqlitePool, id: i64) -> Result<Option<MemberRecord>, sqlx::Error> {
    let row: Option<MemberRow> = sqlx::query_as(
        "SELECT id, member_no, name, phone_number, birth_date, gender, created_at, updated_at
         FROM members WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    row.map(MemberRecord::try_from).transpose()
}

/// `id`인 회원의 칸을 모두 `member`로 바꾸고 고친 뒤 모습을 돌려준다. 없으면 `None`.
///
/// `created_at`은 그대로, `updated_at`은 `now`가 된다.
pub async fn update(
    pool: &SqlitePool,
    id: i64,
    member: &NewMember,
    now: DateTime<Utc>,
) -> Result<Option<MemberRecord>, SaveMemberError> {
    let row: Option<MemberRow> = sqlx::query_as(
        "UPDATE members
         SET member_no = ?, name = ?, phone_number = ?, birth_date = ?, gender = ?, updated_at = ?
         WHERE id = ?
         RETURNING id, member_no, name, phone_number, birth_date, gender, created_at, updated_at",
    )
    .bind(member.member_no().expose().as_str())
    .bind(member.name().expose().as_str())
    .bind(member.phone_number().expose().as_str())
    .bind(member.birth_date().map(|date| *date.expose()))
    .bind(member.gender().map(|gender| gender_text(*gender.expose())))
    .bind(timestamp_text(now))
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(MemberRecord::try_from).transpose()?)
}

/// `id`인 회원을 지운다. 지웠으면 `true`, 없으면 `false`.
///
/// BR-M5: 신청 기록이 있는 회원은 지우지 않는다. 신청 테이블의 외래 키가 막는다(신청 기능에서 추가).
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM members WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// LIKE에서 특별한 뜻이 있는 `%`, `_`와 이스케이프 글자 `\`를 글자 그대로 찾게 한다.
fn escape_like(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for c in text.chars() {
        if matches!(c, '%' | '_' | '\\') {
            escaped.push('\\');
        }
        escaped.push(c);
    }
    escaped
}

/// 검색어가 숫자, 하이픈, 공백으로만 되어 있으면 숫자만 남긴다. 연락처는 숫자만 저장되어 있다.
fn phone_digits(query: &str) -> Option<String> {
    let only_phone_chars = query
        .chars()
        .all(|c| c.is_ascii_digit() || c == '-' || c == ' ');
    let digits: String = query.chars().filter(char::is_ascii_digit).collect();
    (only_phone_chars && !digits.is_empty()).then_some(digits)
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
            matches!(error, SaveMemberError::DuplicateMemberNo),
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

    async fn add(pool: &SqlitePool, member_no: &str, name: &str, phone_number: &str) {
        let member = new_member(MemberInput {
            name,
            phone_number,
            ..input(member_no)
        });
        insert(pool, &member, now()).await.unwrap();
    }

    async fn found(pool: &SqlitePool, query: &str) -> Vec<String> {
        search(pool, query, 1, 20)
            .await
            .unwrap()
            .items
            .into_iter()
            .map(|member| member.member_no.into_inner())
            .collect()
    }

    async fn sample() -> (tempfile::TempDir, SqlitePool) {
        let (dir, pool) = open().await;
        add(&pool, "00001", "나회원", "01011112222").await;
        add(&pool, "00002", "가회원", "01033334444").await;
        add(&pool, "A-100", "다손님", "0325556666").await;
        (dir, pool)
    }

    #[tokio::test]
    async fn empty_query_finds_everyone_by_name() {
        let (_dir, pool) = sample().await;

        assert_eq!(found(&pool, "").await, ["00002", "00001", "A-100"]);
    }

    #[tokio::test]
    async fn finds_by_part_of_name() {
        let (_dir, pool) = sample().await;

        assert_eq!(found(&pool, "회원").await, ["00002", "00001"]);
        assert_eq!(found(&pool, " 손님 ").await, ["A-100"]);
    }

    #[tokio::test]
    async fn finds_by_part_of_member_no() {
        let (_dir, pool) = sample().await;

        assert_eq!(found(&pool, "A-1").await, ["A-100"]);
    }

    #[tokio::test]
    async fn finds_by_phone_digits_with_hyphens() {
        let (_dir, pool) = sample().await;

        assert_eq!(found(&pool, "3333-4444").await, ["00002"]);
        assert_eq!(found(&pool, "032 555").await, ["A-100"]);
    }

    #[tokio::test]
    async fn like_symbols_are_plain_text() {
        let (_dir, pool) = sample().await;

        assert!(found(&pool, "%").await.is_empty());
        assert!(found(&pool, "_").await.is_empty());
        assert!(found(&pool, "\\").await.is_empty());
    }

    #[tokio::test]
    async fn pages_and_total() {
        let (_dir, pool) = sample().await;

        let first = search(&pool, "", 1, 2).await.unwrap();
        let second = search(&pool, "", 2, 2).await.unwrap();

        assert_eq!(first.total, 3);
        assert_eq!(first.items.len(), 2);
        assert_eq!(second.total, 3);
        assert_eq!(second.items[0].member_no.expose(), "A-100");
    }

    #[tokio::test]
    async fn search_returns_saved_fields() {
        let (_dir, pool) = open().await;
        let member = new_member(MemberInput {
            birth_date: NaiveDate::from_ymd_opt(1950, 3, 1),
            gender: Some(Gender::Male),
            ..input("00001")
        });
        let id = insert(&pool, &member, now()).await.unwrap();

        let page = search(&pool, "", 1, 20).await.unwrap();

        let record = &page.items[0];
        assert_eq!(record.id, id);
        assert_eq!(record.name.expose(), "회원1");
        assert_eq!(record.phone_number.expose(), "01000000001");
        let birth_date = record.birth_date.as_ref().map(|date| *date.expose());
        assert_eq!(birth_date, NaiveDate::from_ymd_opt(1950, 3, 1));
        let gender = record.gender.as_ref().map(|gender| *gender.expose());
        assert_eq!(gender, Some(Gender::Male));
    }

    #[test]
    fn phone_digits_only_for_phone_like_query() {
        assert_eq!(phone_digits("010-1234").as_deref(), Some("0101234"));
        assert_eq!(phone_digits("회원1"), None);
        assert_eq!(phone_digits(" - "), None);
    }

    fn later() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 16, 9, 0, 0).unwrap()
    }

    #[tokio::test]
    async fn get_reads_member_with_times() {
        let (_dir, pool) = open().await;
        let id = insert(&pool, &new_member(input("00001")), now())
            .await
            .unwrap();

        let record = get(&pool, id).await.unwrap().unwrap();

        assert_eq!(record.member_no.expose(), "00001");
        assert_eq!(record.created_at, now());
        assert_eq!(record.updated_at, now());
    }

    #[tokio::test]
    async fn get_unknown_id_is_none() {
        let (_dir, pool) = open().await;

        assert!(get(&pool, 999).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn update_replaces_every_field() {
        let (_dir, pool) = open().await;
        let original = new_member(MemberInput {
            birth_date: NaiveDate::from_ymd_opt(1950, 3, 1),
            gender: Some(Gender::Female),
            ..input("00001")
        });
        let id = insert(&pool, &original, now()).await.unwrap();
        let changed = new_member(MemberInput {
            member_no: "00009",
            name: "회원2",
            phone_number: "0320000000",
            birth_date: None,
            gender: None,
        });

        let record = update(&pool, id, &changed, later()).await.unwrap().unwrap();

        assert_eq!(record.id, id);
        assert_eq!(record.member_no.expose(), "00009");
        assert_eq!(record.name.expose(), "회원2");
        assert_eq!(record.phone_number.expose(), "0320000000");
        assert!(record.birth_date.is_none());
        assert!(record.gender.is_none());
        assert_eq!(record.created_at, now());
        assert_eq!(record.updated_at, later());
    }

    #[tokio::test]
    async fn update_unknown_id_is_none() {
        let (_dir, pool) = open().await;

        let result = update(&pool, 999, &new_member(input("00001")), now())
            .await
            .unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn br_m2_update_to_taken_member_no_is_told_apart() {
        let (_dir, pool) = open().await;
        insert(&pool, &new_member(input("00001")), now())
            .await
            .unwrap();
        let id = insert(&pool, &new_member(input("00002")), now())
            .await
            .unwrap();

        let error = update(&pool, id, &new_member(input("00001")), later())
            .await
            .unwrap_err();

        assert!(
            matches!(error, SaveMemberError::DuplicateMemberNo),
            "{error}"
        );
    }

    #[tokio::test]
    async fn update_keeping_own_member_no_is_fine() {
        let (_dir, pool) = open().await;
        let id = insert(&pool, &new_member(input("00001")), now())
            .await
            .unwrap();

        let record = update(&pool, id, &new_member(input("00001")), later())
            .await
            .unwrap();

        assert!(record.is_some());
    }

    #[tokio::test]
    async fn br_m5_deletes_member_without_records() {
        let (_dir, pool) = open().await;
        let id = insert(&pool, &new_member(input("00001")), now())
            .await
            .unwrap();

        assert!(delete(&pool, id).await.unwrap());
        assert!(get(&pool, id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn delete_unknown_id_is_false() {
        let (_dir, pool) = open().await;

        assert!(!delete(&pool, 999).await.unwrap());
    }

    #[tokio::test]
    async fn member_no_can_be_reused_after_delete() {
        let (_dir, pool) = open().await;
        let id = insert(&pool, &new_member(input("00001")), now())
            .await
            .unwrap();
        delete(&pool, id).await.unwrap();

        insert(&pool, &new_member(input("00001")), now())
            .await
            .unwrap();
    }
}
