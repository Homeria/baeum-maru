//! SQLite DB 열기와 구조 변경(마이그레이션).
//!
//! ADR 0005: SQLite + sqlx. 구조 변경은 시작할 때 자동으로 적용한다.

pub mod access_logs;
#[cfg_attr(not(test), expect(dead_code, reason = "신청 API에서 처음 쓴다"))]
pub mod applications;
pub mod courses;
pub mod members;
pub mod recovery;
pub mod rooms;
pub mod sessions;
pub mod staff;
pub mod subjects;
pub mod terms;
#[cfg(test)]
pub(crate) mod test_support;

use std::path::Path;
use std::time::Duration;

use time::{OffsetDateTime, UtcOffset};

use sqlx::SqlitePool;
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};

/// 데이터 폴더 안의 DB 파일 이름.
const FILE_NAME: &str = "baeum-maru.db";

/// `src-tauri/migrations/`의 구조 변경 파일. 빌드할 때 실행 파일에 들어간다.
static MIGRATOR: Migrator = sqlx::migrate!();

/// DB를 열지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("데이터 폴더를 만들지 못했습니다: {0}")]
    CreateDir(#[source] std::io::Error),
    #[error("DB를 열지 못했습니다: {0}")]
    Open(#[from] sqlx::Error),
    #[error("DB 구조 변경을 적용하지 못했습니다: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
}

/// DB에 시각을 쓰는 모양. [docs/conventions.md]: UTC, 밀리초까지, 끝에 Z
///
/// 글자끼리 크기를 비교하므로 자리 수가 늘 같아야 한다.
pub fn timestamp_text(time: OffsetDateTime) -> String {
    let time = time.to_offset(UtcOffset::UTC);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        time.year(),
        u8::from(time.month()),
        time.day(),
        time.hour(),
        time.minute(),
        time.second(),
        time.millisecond(),
    )
}

/// UNIQUE 제약을 어겨서 난 오류인지. 테이블마다 어떤 UNIQUE인지는 부르는 쪽이 안다.
fn is_unique_violation(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .is_some_and(|db_error| db_error.is_unique_violation())
}

/// 외래 키를 어겨서 난 오류인지. 예: 강좌가 쓰는 학기를 지우려 할 때.
///
/// 마이그레이션은 `ON DELETE NO ACTION`으로 막는다. `RESTRICT`는 다른 오류 코드를 내서 여기서 알아보지 못한다.
fn is_foreign_key_violation(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .is_some_and(|db_error| db_error.is_foreign_key_violation())
}

/// 지우지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum DeleteError {
    /// 다른 데이터가 쓰고 있다. 예: 강좌가 있는 학기 (BR-T5)
    #[error("다른 데이터가 쓰고 있어 지울 수 없습니다")]
    InUse,
    #[error("지우지 못했습니다: {0}")]
    Db(sqlx::Error),
}

impl From<sqlx::Error> for DeleteError {
    fn from(error: sqlx::Error) -> Self {
        if is_foreign_key_violation(&error) {
            DeleteError::InUse
        } else {
            DeleteError::Db(error)
        }
    }
}

/// 테스트용: 임시 폴더에 새 DB를 연다. 폴더는 돌려준 `TempDir`가 사라질 때 지워진다.
#[cfg(test)]
pub(crate) async fn open_temp() -> (tempfile::TempDir, SqlitePool) {
    let dir = tempfile::tempdir().unwrap();
    let pool = open(dir.path()).await.unwrap();
    (dir, pool)
}

/// `dir` 안의 DB를 열고(없으면 만들고) 구조 변경을 적용한다.
pub async fn open(dir: &Path) -> Result<SqlitePool, DbError> {
    tokio::fs::create_dir_all(dir)
        .await
        .map_err(DbError::CreateDir)?;

    // [ADR 0005]: 실험 01의 설정. 정전 때 마지막 저장을 잃지 않도록 synchronous는 FULL
    let options = SqliteConnectOptions::new()
        .filename(dir.join(FILE_NAME))
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Full)
        .busy_timeout(Duration::from_secs(5))
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(options)
        .await?;
    MIGRATOR.run(&pool).await?;
    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::test_support::{
        NOW, insert_application, insert_course, insert_deletion, insert_member, insert_room,
        insert_schedule, insert_staff, insert_subject, insert_term,
    };
    use time::macros::datetime;

    use super::*;

    #[test]
    fn timestamp_text_always_has_same_shape() {
        // 글자끼리 비교하므로 밀리초 세 자리와 Z가 늘 붙어야 한다
        assert_eq!(
            timestamp_text(datetime!(2026-11-15 01:02:03 UTC)),
            "2026-11-15T01:02:03.000Z"
        );
        assert_eq!(
            timestamp_text(datetime!(2026-11-15 01:02:03.25 UTC)),
            "2026-11-15T01:02:03.250Z"
        );
        // 다른 시간대 시각도 UTC로 바꿔 쓴다
        assert_eq!(
            timestamp_text(datetime!(2026-11-15 09:00 +09:00)),
            "2026-11-15T00:00:00.000Z"
        );
    }

    #[tokio::test]
    async fn reads_back_written_timestamp() {
        let (_dir, pool) = open_temp().await;
        let written = datetime!(2026-11-15 01:02:03.456 UTC);

        let read: OffsetDateTime = sqlx::query_scalar("SELECT ?")
            .bind(timestamp_text(written))
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(read, written);
    }

    async fn text(pool: &SqlitePool, sql: &'static str) -> String {
        sqlx::query_scalar(sql).fetch_one(pool).await.unwrap()
    }

    #[tokio::test]
    async fn creates_db_file_in_new_folder() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("data");

        open(&dir).await.unwrap();

        assert!(dir.join(FILE_NAME).is_file());
    }

    #[tokio::test]
    async fn applies_settings() {
        let (_dir, pool) = open_temp().await;

        let journal_mode = "SELECT journal_mode FROM pragma_journal_mode";
        assert_eq!(text(&pool, journal_mode).await, "wal");
        // 2 = FULL
        let synchronous = "SELECT CAST(synchronous AS TEXT) FROM pragma_synchronous";
        assert_eq!(text(&pool, synchronous).await, "2");
        let foreign_keys = "SELECT CAST(foreign_keys AS TEXT) FROM pragma_foreign_keys";
        assert_eq!(text(&pool, foreign_keys).await, "1");
        let busy_timeout = "SELECT CAST(timeout AS TEXT) FROM pragma_busy_timeout";
        assert_eq!(text(&pool, busy_timeout).await, "5000");
    }

    #[tokio::test]
    async fn opens_existing_db_again() {
        let dir = tempfile::tempdir().unwrap();
        open(dir.path()).await.unwrap().close().await;

        open(dir.path()).await.unwrap();
    }

    #[tokio::test]
    async fn br_m2_rejects_duplicate_member_no() {
        let (_dir, pool) = open_temp().await;

        insert_member(&pool, "00001").await.unwrap();
        let duplicate = insert_member(&pool, "00001").await.unwrap_err();

        let is_unique_violation = duplicate
            .as_database_error()
            .is_some_and(|error| error.is_unique_violation());
        assert!(is_unique_violation, "{duplicate}");
    }

    #[tokio::test]
    async fn br_m2_allows_different_member_no() {
        let (_dir, pool) = open_temp().await;

        insert_member(&pool, "00001").await.unwrap();
        insert_member(&pool, "00002").await.unwrap();
    }

    #[tokio::test]
    async fn members_reject_unknown_gender() {
        let (_dir, pool) = open_temp().await;

        let result = sqlx::query(
            "INSERT INTO members (member_no, name, phone_number, gender, created_at, updated_at)
             VALUES ('00001', '회원1', '01000000001', 'other', '2026-11-15T00:00:00.000Z', '2026-11-15T00:00:00.000Z')",
        )
        .execute(&pool)
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn br_r1_rejects_same_name_in_same_building() {
        let (_dir, pool) = open_temp().await;

        insert_room(&pool, Some("본관"), Some(3), "다목적실")
            .await
            .unwrap();
        // 층이 달라도 겹친 것으로 봄
        let duplicate = insert_room(&pool, Some("본관"), Some(2), "다목적실")
            .await
            .unwrap_err();

        let is_unique_violation = duplicate
            .as_database_error()
            .is_some_and(|error| error.is_unique_violation());
        assert!(is_unique_violation, "{duplicate}");
    }

    #[tokio::test]
    async fn br_r1_allows_same_name_in_other_building() {
        let (_dir, pool) = open_temp().await;

        insert_room(&pool, Some("본관"), None, "다목적실")
            .await
            .unwrap();
        insert_room(&pool, Some("별관"), None, "다목적실")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn br_r1_rooms_require_building() {
        let (_dir, pool) = open_temp().await;

        assert!(insert_room(&pool, None, None, "다목적실").await.is_err());
    }

    #[tokio::test]
    async fn rooms_accept_floors_from_basement_2_to_5() {
        let (_dir, pool) = open_temp().await;

        for (i, floor) in [None, Some(-2), Some(-1), Some(1), Some(5)]
            .into_iter()
            .enumerate()
        {
            let name = format!("강의실{i}");
            insert_room(&pool, Some("본관"), floor, &name)
                .await
                .unwrap();
        }
    }

    #[tokio::test]
    async fn rooms_reject_floor_zero_and_out_of_range() {
        let (_dir, pool) = open_temp().await;

        for floor in [0, -3, 6] {
            let result = insert_room(&pool, Some("본관"), Some(floor), "다목적실").await;
            assert!(result.is_err(), "{floor}");
        }
    }

    #[tokio::test]
    async fn br_j1_rejects_same_name_key() {
        let (_dir, pool) = open_temp().await;

        insert_subject(&pool, "한글교실", "한글교실", None)
            .await
            .unwrap();
        // 분류가 달라도 겹친 것으로 봄
        let duplicate = insert_subject(&pool, "한글 교실", "한글교실", Some("평생교육"))
            .await
            .unwrap_err();

        assert!(is_unique_violation(&duplicate), "{duplicate}");
    }

    #[tokio::test]
    async fn br_j1_allows_different_names() {
        let (_dir, pool) = open_temp().await;

        insert_subject(&pool, "스마트폰", "스마트폰", None)
            .await
            .unwrap();
        insert_subject(&pool, "스마트폰 활용", "스마트폰활용", None)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn subjects_keep_no_category_as_null_not_empty_text() {
        let (_dir, pool) = open_temp().await;

        insert_subject(&pool, "요가", "요가", None).await.unwrap();
        assert!(
            insert_subject(&pool, "댄스", "댄스", Some(""))
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn subjects_reject_empty_name_key() {
        let (_dir, pool) = open_temp().await;

        assert!(insert_subject(&pool, " ", "", None).await.is_err());
    }

    /// 학기 하나, 과목 하나를 넣어 둔 DB. (폴더, DB, 학기 id, 과목 id)
    async fn with_term_and_subject() -> (tempfile::TempDir, SqlitePool, i64, i64) {
        let (dir, pool) = open_temp().await;
        let term_id = insert_term(&pool, "2027년 1학기", "2027-03-02", "2027-06-30")
            .await
            .unwrap();
        let subject_id = insert_subject(&pool, "요가", "요가", None).await.unwrap();
        (dir, pool, term_id, subject_id)
    }

    #[tokio::test]
    async fn br_t1_rejects_same_term_name() {
        let (_dir, pool) = open_temp().await;
        insert_term(&pool, "2027년 1학기", "2027-03-02", "2027-06-30")
            .await
            .unwrap();

        let duplicate = insert_term(&pool, "2027년 1학기", "2027-09-01", "2027-12-31")
            .await
            .unwrap_err();

        assert!(is_unique_violation(&duplicate), "{duplicate}");
    }

    #[tokio::test]
    async fn br_t2_rejects_end_before_start_and_allows_one_day() {
        let (_dir, pool) = open_temp().await;

        assert!(
            insert_term(&pool, "거꾸로", "2027-06-30", "2027-03-02")
                .await
                .is_err()
        );
        insert_term(&pool, "하루", "2027-03-02", "2027-03-02")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn br_t3_rejects_same_course_in_same_term() {
        let (_dir, pool, term_id, subject_id) = with_term_and_subject().await;
        insert_course(&pool, term_id, subject_id, Some("초급"), Some("1반"))
            .await
            .unwrap();

        let duplicate = insert_course(&pool, term_id, subject_id, Some("초급"), Some("1반"))
            .await
            .unwrap_err();

        assert!(is_unique_violation(&duplicate), "{duplicate}");
    }

    #[tokio::test]
    async fn br_t3_treats_empty_level_and_section_as_one_value() {
        let (_dir, pool, term_id, subject_id) = with_term_and_subject().await;
        insert_course(&pool, term_id, subject_id, None, None)
            .await
            .unwrap();

        let duplicate = insert_course(&pool, term_id, subject_id, None, None)
            .await
            .unwrap_err();

        assert!(is_unique_violation(&duplicate), "{duplicate}");
    }

    #[tokio::test]
    async fn br_t3_allows_other_section_or_other_term() {
        let (_dir, pool, term_id, subject_id) = with_term_and_subject().await;
        let next_term = insert_term(&pool, "2027년 2학기", "2027-09-01", "2027-12-31")
            .await
            .unwrap();

        insert_course(&pool, term_id, subject_id, Some("초급"), Some("1반"))
            .await
            .unwrap();
        insert_course(&pool, term_id, subject_id, Some("초급"), Some("2반"))
            .await
            .unwrap();
        insert_course(&pool, next_term, subject_id, Some("초급"), Some("1반"))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn courses_reject_capacity_out_of_range() {
        let (_dir, pool, term_id, subject_id) = with_term_and_subject().await;

        for capacity in [0, 201] {
            let result = sqlx::query(
                "INSERT INTO courses (term_id, subject_id, capacity, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind(term_id)
            .bind(subject_id)
            .bind(capacity)
            .bind(NOW)
            .bind(NOW)
            .execute(&pool)
            .await;
            assert!(result.is_err(), "{capacity}");
        }
    }

    #[tokio::test]
    async fn courses_need_existing_term_and_subject() {
        let (_dir, pool, term_id, subject_id) = with_term_and_subject().await;

        let no_term = insert_course(&pool, 999, subject_id, None, None)
            .await
            .unwrap_err();
        let no_subject = insert_course(&pool, term_id, 999, None, None)
            .await
            .unwrap_err();

        assert!(is_foreign_key_violation(&no_term), "{no_term}");
        assert!(is_foreign_key_violation(&no_subject), "{no_subject}");
    }

    #[tokio::test]
    async fn br_t4_schedule_end_must_be_after_start() {
        let (_dir, pool, term_id, subject_id) = with_term_and_subject().await;
        let course_id = insert_course(&pool, term_id, subject_id, None, None)
            .await
            .unwrap();

        insert_schedule(&pool, course_id, 1, 600, 660, None)
            .await
            .unwrap();
        assert!(
            insert_schedule(&pool, course_id, 1, 660, 660, None)
                .await
                .is_err()
        );
        assert!(
            insert_schedule(&pool, course_id, 8, 600, 660, None)
                .await
                .is_err()
        );
        assert!(
            insert_schedule(&pool, course_id, 1, 600, 1440, None)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn br_t5_rejects_deleting_term_with_course() {
        let (_dir, pool, term_id, subject_id) = with_term_and_subject().await;
        insert_course(&pool, term_id, subject_id, None, None)
            .await
            .unwrap();

        let error = sqlx::query("DELETE FROM terms WHERE id = ?")
            .bind(term_id)
            .execute(&pool)
            .await
            .unwrap_err();

        assert!(is_foreign_key_violation(&error), "{error}");
    }

    #[tokio::test]
    async fn br_j2_rejects_deleting_subject_used_by_course() {
        let (_dir, pool, term_id, subject_id) = with_term_and_subject().await;
        insert_course(&pool, term_id, subject_id, None, None)
            .await
            .unwrap();

        let error = sqlx::query("DELETE FROM subjects WHERE id = ?")
            .bind(subject_id)
            .execute(&pool)
            .await
            .unwrap_err();

        assert!(is_foreign_key_violation(&error), "{error}");
    }

    #[tokio::test]
    async fn br_r2_rejects_deleting_room_used_by_schedule() {
        let (_dir, pool, term_id, subject_id) = with_term_and_subject().await;
        let room_id = insert_room(&pool, Some("본관"), None, "다목적실")
            .await
            .unwrap();
        let course_id = insert_course(&pool, term_id, subject_id, None, None)
            .await
            .unwrap();
        insert_schedule(&pool, course_id, 1, 600, 660, Some(room_id))
            .await
            .unwrap();

        let error = sqlx::query("DELETE FROM rooms WHERE id = ?")
            .bind(room_id)
            .execute(&pool)
            .await
            .unwrap_err();

        assert!(is_foreign_key_violation(&error), "{error}");
    }

    #[tokio::test]
    async fn deleting_course_removes_its_schedules() {
        let (_dir, pool, term_id, subject_id) = with_term_and_subject().await;
        let course_id = insert_course(&pool, term_id, subject_id, None, None)
            .await
            .unwrap();
        insert_schedule(&pool, course_id, 1, 600, 660, None)
            .await
            .unwrap();

        sqlx::query("DELETE FROM courses WHERE id = ?")
            .bind(course_id)
            .execute(&pool)
            .await
            .unwrap();

        let left: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM course_schedules")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(left, 0);
    }

    #[tokio::test]
    async fn br_k2_rejects_same_staff_name_key() {
        let (_dir, pool) = open_temp().await;
        insert_staff(&pool, "직원1", "staff", None).await.unwrap();

        let result = insert_staff(&pool, "직원1", "admin", None).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn rejects_unknown_role() {
        let (_dir, pool) = open_temp().await;

        let result = insert_staff(&pool, "직원1", "boss", None).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn br_k5_temp_staff_needs_valid_until() {
        let (_dir, pool) = open_temp().await;

        assert!(
            insert_staff(&pool, "직원1", "temp_staff", None)
                .await
                .is_err()
        );
        insert_staff(&pool, "직원1", "temp_staff", Some("2027-03-31"))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn br_k13_access_logs_cannot_be_changed_or_deleted() {
        let (_dir, pool) = open_temp().await;
        let staff_id = insert_staff(&pool, "직원1", "staff", None).await.unwrap();
        sqlx::query(
            "INSERT INTO access_logs (occurred_at, staff_id, event) VALUES (?, ?, 'login')",
        )
        .bind(NOW)
        .bind(staff_id)
        .execute(&pool)
        .await
        .unwrap();

        let update = sqlx::query("UPDATE access_logs SET event = 'logout'")
            .execute(&pool)
            .await;
        let delete = sqlx::query("DELETE FROM access_logs").execute(&pool).await;

        assert!(update.is_err());
        assert!(delete.is_err());
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM access_logs")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn rejects_unknown_access_event() {
        let (_dir, pool) = open_temp().await;

        let result = sqlx::query("INSERT INTO access_logs (occurred_at, event) VALUES (?, 'hack')")
            .bind(NOW)
            .execute(&pool)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn keeps_only_one_recovery_code() {
        let (_dir, pool) = open_temp().await;
        let insert = |id: i64| {
            sqlx::query(
                "INSERT INTO recovery_code (id, code_hash, created_at) VALUES (?, 'hash', ?)",
            )
            .bind(id)
            .bind(NOW)
            .execute(&pool)
        };

        insert(1).await.unwrap();

        assert!(insert(1).await.is_err());
        assert!(insert(2).await.is_err());
    }

    #[tokio::test]
    async fn session_token_hash_is_32_unique_bytes() {
        let (_dir, pool) = open_temp().await;
        let staff_id = insert_staff(&pool, "직원1", "staff", None).await.unwrap();
        let insert = |hash: Vec<u8>| {
            sqlx::query(
                "INSERT INTO sessions (staff_id, token_hash, started_at, last_seen_at)
                 VALUES (?, ?, ?, ?)",
            )
            .bind(staff_id)
            .bind(hash)
            .bind(NOW)
            .bind(NOW)
            .execute(&pool)
        };

        insert(vec![7; 32]).await.unwrap();

        assert!(insert(vec![7; 32]).await.is_err());
        assert!(insert(vec![8; 16]).await.is_err());
    }

    /// 회원, 강좌, 직원 하나씩. (폴더, DB, 회원 id, 강좌 id, 직원 id)
    async fn with_member_course_staff() -> (tempfile::TempDir, SqlitePool, i64, i64, i64) {
        let (dir, pool, term_id, subject_id) = with_term_and_subject().await;
        let member_id = insert_member(&pool, "00001").await.unwrap();
        let course_id = insert_course(&pool, term_id, subject_id, None, None)
            .await
            .unwrap();
        let staff_id = insert_staff(&pool, "직원1", "staff", None).await.unwrap();
        (dir, pool, member_id, course_id, staff_id)
    }

    #[tokio::test]
    async fn br_a4_rejects_second_valid_application() {
        let (_dir, pool, member_id, course_id, staff_id) = with_member_course_staff().await;
        insert_application(&pool, member_id, course_id, "submitted", staff_id)
            .await
            .unwrap();

        let error = insert_application(&pool, member_id, course_id, "submitted", staff_id)
            .await
            .unwrap_err();

        assert!(is_unique_violation(&error), "{error}");
    }

    #[tokio::test]
    async fn br_a4_allows_again_after_withdrawn_or_rejected() {
        let (_dir, pool, member_id, course_id, staff_id) = with_member_course_staff().await;
        for status in ["withdrawn", "rejected", "submitted"] {
            insert_application(&pool, member_id, course_id, status, staff_id)
                .await
                .unwrap();
        }
    }

    #[tokio::test]
    async fn applications_reject_unknown_status() {
        let (_dir, pool, member_id, course_id, staff_id) = with_member_course_staff().await;

        let result = insert_application(&pool, member_id, course_id, "paid", staff_id).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn br_a7_rule_overrides_take_only_overridable_rules_with_reason() {
        let (_dir, pool, member_id, course_id, staff_id) = with_member_course_staff().await;
        let application_id = insert_application(&pool, member_id, course_id, "submitted", staff_id)
            .await
            .unwrap();
        let insert = |rule: &'static str, reason: &'static str| {
            sqlx::query(
                "INSERT INTO rule_overrides (application_id, rule, reason, staff_id, created_at)
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind(application_id)
            .bind(rule)
            .bind(reason)
            .bind(staff_id)
            .bind(NOW)
            .execute(&pool)
        };

        assert!(insert("no_duplicate", "사유").await.is_err());
        assert!(insert("no_time_overlap", "").await.is_err());
        insert("no_time_overlap", "강사 허락").await.unwrap();
        insert("no_same_subject", "강사 허락").await.unwrap();
    }

    #[tokio::test]
    async fn deleting_application_removes_its_overrides() {
        let (_dir, pool, member_id, course_id, staff_id) = with_member_course_staff().await;
        let application_id = insert_application(&pool, member_id, course_id, "submitted", staff_id)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO rule_overrides (application_id, rule, reason, staff_id, created_at)
             VALUES (?, 'no_time_overlap', '강사 허락', ?, ?)",
        )
        .bind(application_id)
        .bind(staff_id)
        .bind(NOW)
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query("DELETE FROM applications WHERE id = ?")
            .bind(application_id)
            .execute(&pool)
            .await
            .unwrap();

        let left: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM rule_overrides")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(left, 0);
    }

    #[tokio::test]
    async fn br_d3_deletion_records_cannot_be_changed_or_deleted() {
        let (_dir, pool, member_id, course_id, staff_id) = with_member_course_staff().await;
        insert_deletion(&pool, member_id, course_id, staff_id)
            .await
            .unwrap();

        let update = sqlx::query("UPDATE application_deletions SET reason = '고침'")
            .execute(&pool)
            .await;
        let delete = sqlx::query("DELETE FROM application_deletions")
            .execute(&pool)
            .await;

        assert!(update.is_err());
        assert!(delete.is_err());
    }

    #[tokio::test]
    async fn br_m5_and_br_t6_reject_deleting_member_and_course_with_application() {
        let (_dir, pool, member_id, course_id, staff_id) = with_member_course_staff().await;
        insert_application(&pool, member_id, course_id, "submitted", staff_id)
            .await
            .unwrap();

        let member = sqlx::query("DELETE FROM members WHERE id = ?")
            .bind(member_id)
            .execute(&pool)
            .await
            .unwrap_err();
        let course = sqlx::query("DELETE FROM courses WHERE id = ?")
            .bind(course_id)
            .execute(&pool)
            .await
            .unwrap_err();

        assert!(is_foreign_key_violation(&member), "{member}");
        assert!(is_foreign_key_violation(&course), "{course}");
    }

    #[tokio::test]
    async fn br_m5_and_br_t6_reject_deleting_member_and_course_with_deletion_record() {
        let (_dir, pool, member_id, course_id, staff_id) = with_member_course_staff().await;
        insert_deletion(&pool, member_id, course_id, staff_id)
            .await
            .unwrap();

        let member = sqlx::query("DELETE FROM members WHERE id = ?")
            .bind(member_id)
            .execute(&pool)
            .await
            .unwrap_err();
        let course = sqlx::query("DELETE FROM courses WHERE id = ?")
            .bind(course_id)
            .execute(&pool)
            .await
            .unwrap_err();

        assert!(is_foreign_key_violation(&member), "{member}");
        assert!(is_foreign_key_violation(&course), "{course}");
    }
}
