//! 회원 저장, 찾기, 고치기, 지우기.

use baeum_domain::{Gender, NewMember, Personal};
use sqlx::SqlitePool;
use time::{Date, OffsetDateTime};

use crate::infra::db::{DeleteError, is_unique_violation, timestamp_text};

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
        if is_unique_violation(&error) {
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
    now: OffsetDateTime,
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
pub(crate) fn gender_from_text(text: &str) -> Result<Gender, sqlx::Error> {
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
    pub birth_date: Option<Personal<Date>>,
    pub gender: Option<Personal<Gender>>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

/// SELECT·RETURNING에서 이 순서로 칸을 읽는다.
type MemberRow = (
    i64,
    String,
    String,
    String,
    Option<Date>,
    Option<String>,
    OffsetDateTime,
    OffsetDateTime,
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
    now: OffsetDateTime,
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
/// BR-M5: 신청이나 삭제 기록이 있는 회원은 [`DeleteError::InUse`]. 두 테이블의 외래 키가 막는다.
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<bool, DeleteError> {
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
pub(super) fn phone_digits(query: &str) -> Option<String> {
    let only_phone_chars = query
        .chars()
        .all(|c| c.is_ascii_digit() || c == '-' || c == ' ');
    let digits: String = query.chars().filter(char::is_ascii_digit).collect();
    (only_phone_chars && !digits.is_empty()).then_some(digits)
}
