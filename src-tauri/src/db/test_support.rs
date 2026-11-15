//! DB 테스트에서 함께 쓰는 도우미. 테스트할 때만 컴파일된다.
//!
//! 업무 규칙 확인을 거치지 않고 SQL로 바로 넣는다. 그래서 테이블 제약(CHECK, UNIQUE,
//! 외래 키)을 시험할 때도, 다른 기능의 데이터를 미리 깔아 둘 때도 쓴다.
//! 모두 새 줄의 id를 돌려준다.

use sqlx::SqlitePool;

/// 만든 시각·고친 시각 칸에 넣는 값.
pub const NOW: &str = "2026-11-15T00:00:00.000Z";

pub async fn insert_room(
    pool: &SqlitePool,
    building: Option<&str>,
    floor: Option<i64>,
    name: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query(
        "INSERT INTO rooms (building, floor, name, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(building)
    .bind(floor)
    .bind(name)
    .bind(NOW)
    .bind(NOW)
    .execute(pool)
    .await
    .map(|result| result.last_insert_rowid())
}

pub async fn insert_subject(
    pool: &SqlitePool,
    name: &str,
    name_key: &str,
    category: Option<&str>,
) -> Result<i64, sqlx::Error> {
    sqlx::query(
        "INSERT INTO subjects (name, name_key, category, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(name)
    .bind(name_key)
    .bind(category)
    .bind(NOW)
    .bind(NOW)
    .execute(pool)
    .await
    .map(|result| result.last_insert_rowid())
}

/// 날짜는 `YYYY-MM-DD` 글자.
pub async fn insert_term(
    pool: &SqlitePool,
    name: &str,
    starts_on: &str,
    ends_on: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query(
        "INSERT INTO terms (name, starts_on, ends_on, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(name)
    .bind(starts_on)
    .bind(ends_on)
    .bind(NOW)
    .bind(NOW)
    .execute(pool)
    .await
    .map(|result| result.last_insert_rowid())
}

/// 정원은 20명.
pub async fn insert_course(
    pool: &SqlitePool,
    term_id: i64,
    subject_id: i64,
    level: Option<&str>,
    section: Option<&str>,
) -> Result<i64, sqlx::Error> {
    sqlx::query(
        "INSERT INTO courses (term_id, subject_id, level, section, capacity, created_at, updated_at)
         VALUES (?, ?, ?, ?, 20, ?, ?)",
    )
    .bind(term_id)
    .bind(subject_id)
    .bind(level)
    .bind(section)
    .bind(NOW)
    .bind(NOW)
    .execute(pool)
    .await
    .map(|result| result.last_insert_rowid())
}

/// 요일은 1=월 … 7=일, 시각은 자정부터 센 분.
pub async fn insert_schedule(
    pool: &SqlitePool,
    course_id: i64,
    weekday: i64,
    start_minute: i64,
    end_minute: i64,
    room_id: Option<i64>,
) -> Result<i64, sqlx::Error> {
    sqlx::query(
        "INSERT INTO course_schedules (course_id, weekday, start_minute, end_minute, room_id)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(course_id)
    .bind(weekday)
    .bind(start_minute)
    .bind(end_minute)
    .bind(room_id)
    .execute(pool)
    .await
    .map(|result| result.last_insert_rowid())
}

/// PIN 해시 칸은 아무 글자나 넣는다. 해시는 직원 기능의 테스트가 확인한다.
pub async fn insert_staff(
    pool: &SqlitePool,
    name_key: &str,
    role: &str,
    valid_until: Option<&str>,
) -> Result<i64, sqlx::Error> {
    sqlx::query(
        "INSERT INTO staff (name, name_key, pin_hash, role, valid_until, created_at, updated_at)
         VALUES (?, ?, 'hash', ?, ?, ?, ?)",
    )
    .bind(name_key)
    .bind(name_key)
    .bind(role)
    .bind(valid_until)
    .bind(NOW)
    .bind(NOW)
    .execute(pool)
    .await
    .map(|result| result.last_insert_rowid())
}
