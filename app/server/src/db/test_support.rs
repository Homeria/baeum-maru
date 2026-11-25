//! DB 테스트에서 함께 쓰는 도우미. 테스트할 때만 컴파일된다.
//!
//! 업무 규칙 확인을 거치지 않고 SQL로 바로 넣는다. 그래서 테이블 제약(CHECK, UNIQUE,
//! 외래 키)을 시험할 때도, 다른 기능의 데이터를 미리 깔아 둘 때도 쓴다.
//! 모두 새 줄의 id를 돌려준다.

use sqlx::SqlitePool;

/// 만든 시각·고친 시각 칸에 넣는 값.
pub const NOW: &str = "2026-11-15T00:00:00.000Z";

/// 이름과 연락처는 지어낸 값이다.
pub async fn insert_member(pool: &SqlitePool, member_no: &str) -> Result<i64, sqlx::Error> {
    sqlx::query(
        "INSERT INTO members (member_no, name, phone_number, created_at, updated_at)
         VALUES (?, '회원1', '01000000001', ?, ?)",
    )
    .bind(member_no)
    .bind(NOW)
    .bind(NOW)
    .execute(pool)
    .await
    .map(|result| result.last_insert_rowid())
}

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

/// 상태는 `submitted` 같은 DB 글자.
pub async fn insert_application(
    pool: &SqlitePool,
    member_id: i64,
    course_id: i64,
    status: &str,
    staff_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query(
        "INSERT INTO applications (member_id, course_id, status, submitted_by, submitted_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(member_id)
    .bind(course_id)
    .bind(status)
    .bind(staff_id)
    .bind(NOW)
    .execute(pool)
    .await
    .map(|result| result.last_insert_rowid())
}

/// 사유는 없음(NULL).
pub async fn insert_deletion(
    pool: &SqlitePool,
    member_id: i64,
    course_id: i64,
    staff_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query(
        "INSERT INTO application_deletions (member_id, course_id, deleted_by, deleted_at)
         VALUES (?, ?, ?, ?)",
    )
    .bind(member_id)
    .bind(course_id)
    .bind(staff_id)
    .bind(NOW)
    .execute(pool)
    .await
    .map(|result| result.last_insert_rowid())
}

/// 시드는 `0123456789abcdef`, 버전 1, 정원 20명. 취소되지 않은 기록.
pub async fn insert_lottery_run(
    pool: &SqlitePool,
    course_id: i64,
    staff_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query(
        "INSERT INTO lottery_runs (course_id, run_by, run_at, seed, version, capacity)
         VALUES (?, ?, ?, '0123456789abcdef', 1, 20)",
    )
    .bind(course_id)
    .bind(staff_id)
    .bind(NOW)
    .execute(pool)
    .await
    .map(|result| result.last_insert_rowid())
}

/// 자리 보장 방식의 사유. 띄어쓰기를 뺀 글자를 `name_key`로 쓴다.
pub async fn insert_priority_reason(pool: &SqlitePool, name: &str) -> Result<i64, sqlx::Error> {
    let key: String = name.chars().filter(|c| !c.is_whitespace()).collect();
    sqlx::query(
        "INSERT INTO priority_reasons (name, name_key, method, created_at, updated_at)
         VALUES (?, ?, 'guaranteed', ?, ?)",
    )
    .bind(name)
    .bind(key)
    .bind(NOW)
    .bind(NOW)
    .execute(pool)
    .await
    .map(|result| result.last_insert_rowid())
}

/// 내보내기 기록 한 줄. 종류, 칸, 가림 글자는 `raw` 같은 DB 글자.
pub async fn insert_roster_export(
    pool: &SqlitePool,
    term_id: i64,
    kind: &str,
    mask_char: &str,
    staff_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query(
        "INSERT INTO roster_exports
             (term_id, kind, included_columns, masked_columns, mask_char, course_count,
              exported_by, exported_at)
         VALUES (?, ?, 'course_name,result,name,member_no', 'name', ?, 1, ?, ?)",
    )
    .bind(term_id)
    .bind(kind)
    .bind(mask_char)
    .bind(staff_id)
    .bind(NOW)
    .execute(pool)
    .await
    .map(|result| result.last_insert_rowid())
}

/// 선발 결과 변경 기록 한 줄. 종류와 상태는 `withdrawn` 같은 DB 글자.
pub async fn insert_selection_change(
    pool: &SqlitePool,
    application_id: i64,
    kind: &str,
    from_status: &str,
    to_status: &str,
    reason: Option<&str>,
    staff_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query(
        "INSERT INTO selection_changes
             (application_id, kind, from_status, to_status, reason, changed_by, changed_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(application_id)
    .bind(kind)
    .bind(from_status)
    .bind(to_status)
    .bind(reason)
    .bind(staff_id)
    .bind(NOW)
    .execute(pool)
    .await
    .map(|result| result.last_insert_rowid())
}

/// [`insert_drawn_course`]가 만든 것의 id.
pub struct DrawnCourse {
    pub term_id: i64,
    pub course_id: i64,
    pub staff_id: i64,
}

/// 정원 `capacity`명인 강좌에 회원 `applicants`명이 신청하고 추첨했다. `close`면 선발도 마감한다.
///
/// 추첨과 마감은 앱의 함수를 그대로 쓴다. 시드는 늘 같아서 결과도 같다.
pub async fn insert_drawn_course(
    pool: &SqlitePool,
    capacity: i64,
    applicants: usize,
    close: bool,
) -> DrawnCourse {
    let term_id = insert_term(pool, "2027년 1학기", "2027-03-02", "2027-06-30")
        .await
        .unwrap();
    let subject_id = insert_subject(pool, "요가", "요가", None).await.unwrap();
    let course_id = insert_course(pool, term_id, subject_id, None, None)
        .await
        .unwrap();
    sqlx::query("UPDATE courses SET capacity = ? WHERE id = ?")
        .bind(capacity)
        .bind(course_id)
        .execute(pool)
        .await
        .unwrap();
    let staff_id = insert_staff(pool, "직원1", "staff", None).await.unwrap();
    for number in 1..=applicants {
        let member_id = insert_member(pool, &format!("{number:05}")).await.unwrap();
        insert_application(pool, member_id, course_id, "submitted", staff_id)
            .await
            .unwrap();
    }
    let now = time::macros::datetime!(2026-11-22 01:02:03 UTC);
    crate::features::lotteries::repository::run(
        pool,
        course_id,
        staff_id,
        baeum_domain::LotterySeed::new(7),
        now,
    )
    .await
    .unwrap();
    if close {
        crate::features::lotteries::repository::close(pool, course_id, staff_id, now)
            .await
            .unwrap();
    }
    DrawnCourse {
        term_id,
        course_id,
        staff_id,
    }
}
