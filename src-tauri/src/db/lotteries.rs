//! 추첨(BR-S6, BR-S7), 재추첨(BR-L4), 선발 마감(BR-L5)과 추첨 화면에 보여 줄 신청자.
//!
//! 시드는 부른 쪽이 만들어 넘긴다(ADR 0016). 호스트 PC에서만 부르는지(BR-L6)와 역할은
//! 서버가 확인한다.

use baeum_domain::{Capacity, LotterySeed, LotteryVersion, Personal, draw_lottery};
use sqlx::{SqliteConnection, SqlitePool};
use time::OffsetDateTime;

use super::timestamp_text;

/// 강좌 한 줄에서 선발 단계를 읽는 SELECT 칸. 강좌 테이블은 `c`.
macro_rules! stage_columns {
    () => {
        "EXISTS (SELECT 1 FROM lottery_runs r
                 WHERE r.course_id = c.id AND r.cancelled_at IS NULL) AS drawn,
         EXISTS (SELECT 1 FROM selection_closures s WHERE s.course_id = c.id) AS closed"
    };
}

/// 강좌의 선발이 어디까지 왔는지.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionStage {
    /// 아직 추첨하지 않았다. 신청을 받는다.
    NotDrawn,
    /// 추첨했다. 마감 전이라 다시 추첨할 수 있다(BR-L4).
    Drawn,
    /// 선발 마감(BR-L5).
    Closed,
}

/// 학기 강좌 하나의 선발 단계.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CourseStage {
    pub course_id: i64,
    pub stage: SelectionStage,
}

/// 추첨 화면의 신청자 한 명.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entrant {
    pub application_id: i64,
    pub member_name: Personal<String>,
    pub member_no: Personal<String>,
}

/// 추첨 화면에 보여 줄 강좌 하나의 선발.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CourseLottery {
    pub capacity: Capacity,
    pub stage: SelectionStage,
    /// 추첨 전이면 접수됨 신청을 접수한 순서로, 추첨 뒤면 뽑힌 순서로.
    pub entrants: Vec<Entrant>,
    /// 추첨 뒤 `entrants`의 앞에서 몇 명이 당첨인지. 나머지는 차례로 대기 순번 1, 2, …
    /// 추첨 전이면 0.
    pub selected_count: usize,
}

/// 추첨하지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum RunLotteryError {
    #[error("강좌를 찾지 못했습니다")]
    CourseNotFound,
    /// BR-L5: 마감 뒤에는 다시 추첨할 수 없다.
    #[error("선발을 마감한 강좌입니다")]
    Closed,
    /// 접수됨 신청이 없다. 빈 추첨으로 신청만 막히는 일(BR-L3)을 피한다.
    #[error("추첨할 신청이 없습니다")]
    NoApplicants,
    #[error("추첨하지 못했습니다: {0}")]
    Db(#[from] sqlx::Error),
}

/// 선발을 마감하지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum CloseSelectionError {
    #[error("강좌를 찾지 못했습니다")]
    CourseNotFound,
    /// 추첨 전에는 마감할 결과가 없다.
    #[error("추첨하지 않은 강좌입니다")]
    NotDrawn,
    #[error("이미 선발을 마감한 강좌입니다")]
    AlreadyClosed,
    #[error("선발을 마감하지 못했습니다: {0}")]
    Db(#[from] sqlx::Error),
}

/// `term_id` 학기 강좌마다 선발 단계. 강좌 id 순.
pub async fn stages_in_term(
    pool: &SqlitePool,
    term_id: i64,
) -> Result<Vec<CourseStage>, sqlx::Error> {
    let rows: Vec<StageRow> = sqlx::query_as(concat!(
        "SELECT c.id AS course_id, ",
        stage_columns!(),
        " FROM courses c WHERE c.term_id = ? ORDER BY c.id"
    ))
    .bind(term_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| CourseStage {
            course_id: row.course_id,
            stage: row.stage(),
        })
        .collect())
}

/// 강좌 하나의 추첨 화면. 강좌가 없으면 `None`.
pub async fn course_lottery(
    pool: &SqlitePool,
    course_id: i64,
) -> Result<Option<CourseLottery>, sqlx::Error> {
    let mut conn = pool.acquire().await?;
    let Some(course) = course_row(&mut conn, course_id).await? else {
        return Ok(None);
    };
    let capacity = course.capacity()?;
    let stage = course.stage();

    let (entrants, selected_count) = match active_run(&mut conn, course_id).await? {
        Some(run) => {
            let rows: Vec<EntrantRow> = sqlx::query_as(
                "SELECT a.id AS application_id, m.name, m.member_no
                 FROM lottery_entries e
                 JOIN applications a ON a.id = e.application_id
                 JOIN members m ON m.id = a.member_id
                 WHERE e.lottery_run_id = ?
                 ORDER BY e.position",
            )
            .bind(run.id)
            .fetch_all(&mut *conn)
            .await?;
            let selected = rows.len().min(run.capacity()?);
            (rows, selected)
        }
        None => {
            let rows: Vec<EntrantRow> = sqlx::query_as(
                "SELECT a.id AS application_id, m.name, m.member_no
                 FROM applications a JOIN members m ON m.id = a.member_id
                 WHERE a.course_id = ? AND a.status = 'submitted'
                 ORDER BY a.submitted_at, a.id",
            )
            .bind(course_id)
            .fetch_all(&mut *conn)
            .await?;
            (rows, 0)
        }
    };

    Ok(Some(CourseLottery {
        capacity,
        stage,
        entrants: entrants.into_iter().map(Entrant::from).collect(),
        selected_count,
    }))
}

/// 강좌의 접수됨 신청을 추첨하고 새 추첨 기록의 id를 돌려준다.
///
/// 이미 추첨한 강좌면 이전 기록에 취소 표시를 하고 그 신청들을 접수됨으로 되돌린 뒤 다시
/// 뽑는다(BR-L4). 읽고 확인한 뒤 쓰므로 `BEGIN IMMEDIATE`로 한다 (ADR 0005).
pub async fn run(
    pool: &SqlitePool,
    course_id: i64,
    staff_id: i64,
    seed: LotterySeed,
    now: OffsetDateTime,
) -> Result<i64, RunLotteryError> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let course = course_row(&mut tx, course_id)
        .await?
        .ok_or(RunLotteryError::CourseNotFound)?;
    if course.closed {
        return Err(RunLotteryError::Closed);
    }
    let capacity = course.capacity()?;
    let now = timestamp_text(now);

    cancel_active_run(&mut tx, course_id, staff_id, &now).await?;

    let application_ids: Vec<i64> = sqlx::query_scalar(
        "SELECT id FROM applications WHERE course_id = ? AND status = 'submitted'",
    )
    .bind(course_id)
    .fetch_all(&mut *tx)
    .await?;
    if application_ids.is_empty() {
        return Err(RunLotteryError::NoApplicants);
    }

    let version = LotteryVersion::CURRENT;
    let draw = draw_lottery(&application_ids, capacity, seed, version);
    let run_id = sqlx::query(
        "INSERT INTO lottery_runs (course_id, run_by, run_at, seed, version, capacity)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(course_id)
    .bind(staff_id)
    .bind(&now)
    .bind(seed.to_hex())
    .bind(version.number())
    .bind(i64::from(capacity.value()))
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();

    for (index, &application_id) in draw.order().iter().enumerate() {
        let status = if index < draw.selected().len() {
            "selected"
        } else {
            "waitlisted"
        };
        // 정원이 200명까지라 순서도 i64에 들어간다
        let position = i64::try_from(index + 1).map_err(|_| decode_error("순서가 너무 큽니다"))?;
        sqlx::query(
            "INSERT INTO lottery_entries (lottery_run_id, application_id, position)
             VALUES (?, ?, ?)",
        )
        .bind(run_id)
        .bind(application_id)
        .bind(position)
        .execute(&mut *tx)
        .await?;
        sqlx::query("UPDATE applications SET status = ? WHERE id = ?")
            .bind(status)
            .bind(application_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(run_id)
}

/// 강좌의 선발을 마감한다(BR-L5). 마감 뒤에는 다시 추첨할 수 없다.
pub async fn close(
    pool: &SqlitePool,
    course_id: i64,
    staff_id: i64,
    now: OffsetDateTime,
) -> Result<(), CloseSelectionError> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let course = course_row(&mut tx, course_id)
        .await?
        .ok_or(CloseSelectionError::CourseNotFound)?;
    match course.stage() {
        SelectionStage::NotDrawn => return Err(CloseSelectionError::NotDrawn),
        SelectionStage::Closed => return Err(CloseSelectionError::AlreadyClosed),
        SelectionStage::Drawn => {}
    }
    sqlx::query(
        "INSERT INTO selection_closures (course_id, closed_by, closed_at) VALUES (?, ?, ?)",
    )
    .bind(course_id)
    .bind(staff_id)
    .bind(timestamp_text(now))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

#[derive(sqlx::FromRow)]
struct StageRow {
    course_id: i64,
    drawn: bool,
    closed: bool,
}

impl StageRow {
    fn stage(&self) -> SelectionStage {
        stage_of(self.drawn, self.closed)
    }
}

fn stage_of(drawn: bool, closed: bool) -> SelectionStage {
    match (drawn, closed) {
        (_, true) => SelectionStage::Closed,
        (true, false) => SelectionStage::Drawn,
        (false, false) => SelectionStage::NotDrawn,
    }
}

#[derive(sqlx::FromRow)]
struct CourseRow {
    capacity: i64,
    drawn: bool,
    closed: bool,
}

impl CourseRow {
    fn capacity(&self) -> Result<Capacity, sqlx::Error> {
        Capacity::new(self.capacity).map_err(|_| decode_error("정원이 범위 밖입니다"))
    }

    fn stage(&self) -> SelectionStage {
        stage_of(self.drawn, self.closed)
    }
}

async fn course_row(
    conn: &mut SqliteConnection,
    course_id: i64,
) -> Result<Option<CourseRow>, sqlx::Error> {
    sqlx::query_as(concat!(
        "SELECT c.capacity, ",
        stage_columns!(),
        " FROM courses c WHERE c.id = ?"
    ))
    .bind(course_id)
    .fetch_optional(conn)
    .await
}

#[derive(sqlx::FromRow)]
struct RunRow {
    id: i64,
    capacity: i64,
}

impl RunRow {
    fn capacity(&self) -> Result<usize, sqlx::Error> {
        usize::try_from(self.capacity).map_err(|_| decode_error("추첨 기록의 정원이 범위 밖입니다"))
    }
}

/// 강좌의 취소되지 않은 추첨 기록. 없으면 `None`.
/// 강좌의 취소되지 않은 추첨 기록에 취소 표시를 하고, 그 추첨으로 당첨·대기가 된 신청을
/// 접수됨으로 되돌린다(BR-L4, BR-S13). 취소할 기록이 있었으면 `true`.
///
/// `now`는 [`timestamp_text`] 글자. 부르는 쪽이 `BEGIN IMMEDIATE` 트랜잭션 안에서 부른다.
pub(super) async fn cancel_active_run(
    conn: &mut SqliteConnection,
    course_id: i64,
    staff_id: i64,
    now: &str,
) -> Result<bool, sqlx::Error> {
    let Some(run) = active_run(conn, course_id).await? else {
        return Ok(false);
    };
    sqlx::query("UPDATE lottery_runs SET cancelled_by = ?, cancelled_at = ? WHERE id = ?")
        .bind(staff_id)
        .bind(now)
        .bind(run.id)
        .execute(&mut *conn)
        .await?;
    sqlx::query(
        "UPDATE applications SET status = 'submitted'
         WHERE status IN ('selected', 'waitlisted')
           AND id IN (SELECT application_id FROM lottery_entries WHERE lottery_run_id = ?1
                      UNION
                      SELECT application_id FROM lottery_guaranteed_entries
                      WHERE lottery_run_id = ?1)",
    )
    .bind(run.id)
    .execute(&mut *conn)
    .await?;
    Ok(true)
}

async fn active_run(
    conn: &mut SqliteConnection,
    course_id: i64,
) -> Result<Option<RunRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, capacity FROM lottery_runs WHERE course_id = ? AND cancelled_at IS NULL",
    )
    .bind(course_id)
    .fetch_optional(conn)
    .await
}

#[derive(sqlx::FromRow)]
struct EntrantRow {
    application_id: i64,
    name: String,
    member_no: String,
}

impl From<EntrantRow> for Entrant {
    fn from(row: EntrantRow) -> Self {
        Entrant {
            application_id: row.application_id,
            member_name: Personal::new(row.name),
            member_no: Personal::new(row.member_no),
        }
    }
}

/// 테이블 CHECK가 막는 값을 만났다. DB가 망가진 것이다.
fn decode_error(message: &str) -> sqlx::Error {
    sqlx::Error::Decode(message.into())
}

#[cfg(test)]
mod tests {
    use baeum_domain::LotteryDraw;
    use time::macros::datetime;

    use super::*;
    use crate::db::open_temp;
    use crate::db::test_support::{
        insert_application, insert_course, insert_member, insert_staff, insert_subject, insert_term,
    };

    fn now() -> OffsetDateTime {
        datetime!(2026-11-22 01:02:03 UTC)
    }

    const SEED: u64 = 0x0123_4567_89AB_CDEF;

    /// 정원 `capacity`명인 강좌 하나와 접수됨 신청 `applicants`건.
    struct Fixture {
        _dir: tempfile::TempDir,
        pool: SqlitePool,
        term_id: i64,
        course_id: i64,
        staff_id: i64,
        /// 신청 id. 접수한 순서.
        application_ids: Vec<i64>,
    }

    async fn fixture(capacity: i64, applicants: usize) -> Fixture {
        let (dir, pool) = open_temp().await;
        let term_id = insert_term(&pool, "2027년 1학기", "2027-03-02", "2027-06-30")
            .await
            .unwrap();
        let subject_id = insert_subject(&pool, "요가", "요가", None).await.unwrap();
        let course_id = insert_course(&pool, term_id, subject_id, None, None)
            .await
            .unwrap();
        sqlx::query("UPDATE courses SET capacity = ? WHERE id = ?")
            .bind(capacity)
            .bind(course_id)
            .execute(&pool)
            .await
            .unwrap();
        let staff_id = insert_staff(&pool, "직원1", "staff", None).await.unwrap();
        let mut application_ids = Vec::with_capacity(applicants);
        for number in 1..=applicants {
            let member_id = insert_member(&pool, &format!("{number:05}")).await.unwrap();
            let id = insert_application(&pool, member_id, course_id, "submitted", staff_id)
                .await
                .unwrap();
            application_ids.push(id);
        }
        Fixture {
            _dir: dir,
            pool,
            term_id,
            course_id,
            staff_id,
            application_ids,
        }
    }

    impl Fixture {
        async fn run(&self, seed: u64) -> Result<i64, RunLotteryError> {
            run(
                &self.pool,
                self.course_id,
                self.staff_id,
                LotterySeed::new(seed),
                now(),
            )
            .await
        }

        async fn lottery(&self) -> CourseLottery {
            course_lottery(&self.pool, self.course_id)
                .await
                .unwrap()
                .unwrap()
        }

        async fn statuses(&self) -> Vec<(i64, String)> {
            sqlx::query_as("SELECT id, status FROM applications ORDER BY id")
                .fetch_all(&self.pool)
                .await
                .unwrap()
        }

        fn expected(&self, capacity: i64, seed: u64) -> LotteryDraw {
            draw_lottery(
                &self.application_ids,
                Capacity::new(capacity).unwrap(),
                LotterySeed::new(seed),
                LotteryVersion::CURRENT,
            )
        }
    }

    fn ids(lottery: &CourseLottery) -> Vec<i64> {
        lottery.entrants.iter().map(|e| e.application_id).collect()
    }

    #[tokio::test]
    async fn before_lottery_shows_submitted_in_order() {
        let f = fixture(2, 3).await;

        let lottery = f.lottery().await;

        assert_eq!(lottery.stage, SelectionStage::NotDrawn);
        assert_eq!(ids(&lottery), f.application_ids);
        assert_eq!(lottery.selected_count, 0);
        assert_eq!(lottery.entrants[0].member_no.expose(), "00001");
    }

    #[tokio::test]
    async fn br_s7_draw_selects_capacity_and_waitlists_rest_in_order() {
        let f = fixture(2, 5).await;

        let run_id = f.run(SEED).await.unwrap();

        let expected = f.expected(2, SEED);
        let lottery = f.lottery().await;
        assert_eq!(lottery.stage, SelectionStage::Drawn);
        assert_eq!(ids(&lottery), expected.order());
        assert_eq!(lottery.selected_count, 2);
        for (id, status) in f.statuses().await {
            let want = if expected.selected().contains(&id) {
                "selected"
            } else {
                "waitlisted"
            };
            assert_eq!(status, want, "신청 {id}");
        }
        let (seed, version, capacity): (String, i64, i64) =
            sqlx::query_as("SELECT seed, version, capacity FROM lottery_runs WHERE id = ?")
                .bind(run_id)
                .fetch_one(&f.pool)
                .await
                .unwrap();
        assert_eq!(
            (seed.as_str(), version, capacity),
            ("0123456789abcdef", 1, 2)
        );
    }

    #[tokio::test]
    async fn br_s6_everyone_selected_when_not_over_capacity() {
        let f = fixture(5, 3).await;

        f.run(SEED).await.unwrap();

        assert_eq!(f.lottery().await.selected_count, 3);
        assert!(f.statuses().await.iter().all(|(_, s)| s == "selected"));
    }

    #[tokio::test]
    async fn br_l2_record_reproduces_result_even_after_capacity_changes() {
        let f = fixture(2, 5).await;
        f.run(SEED).await.unwrap();
        sqlx::query("UPDATE courses SET capacity = 4")
            .execute(&f.pool)
            .await
            .unwrap();

        let lottery = f.lottery().await;

        assert_eq!(ids(&lottery), f.expected(2, SEED).order());
        assert_eq!(lottery.selected_count, 2);
    }

    #[tokio::test]
    async fn br_l4_redraw_cancels_previous_and_draws_everyone_again() {
        let f = fixture(2, 5).await;
        let first = f.run(SEED).await.unwrap();

        let second = f.run(7).await.unwrap();

        assert_ne!(first, second);
        assert_eq!(ids(&f.lottery().await), f.expected(2, 7).order());
        let cancelled: Option<String> =
            sqlx::query_scalar("SELECT cancelled_at FROM lottery_runs WHERE id = ?")
                .bind(first)
                .fetch_one(&f.pool)
                .await
                .unwrap();
        assert!(cancelled.is_some());
        // 이전 기록의 대상과 순서는 그대로 남는다
        let kept: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM lottery_entries WHERE lottery_run_id = ?")
                .bind(first)
                .fetch_one(&f.pool)
                .await
                .unwrap();
        assert_eq!(kept, 5);
    }

    #[tokio::test]
    async fn br_l5_closed_course_cannot_be_drawn_again() {
        let f = fixture(2, 3).await;
        f.run(SEED).await.unwrap();

        close(&f.pool, f.course_id, f.staff_id, now())
            .await
            .unwrap();

        assert_eq!(f.lottery().await.stage, SelectionStage::Closed);
        assert!(matches!(f.run(7).await, Err(RunLotteryError::Closed)));
        assert!(matches!(
            close(&f.pool, f.course_id, f.staff_id, now()).await,
            Err(CloseSelectionError::AlreadyClosed)
        ));
    }

    #[tokio::test]
    async fn cannot_close_before_lottery() {
        let f = fixture(2, 3).await;

        let result = close(&f.pool, f.course_id, f.staff_id, now()).await;

        assert!(matches!(result, Err(CloseSelectionError::NotDrawn)));
    }

    #[tokio::test]
    async fn no_applicants_is_not_drawn() {
        let f = fixture(2, 0).await;

        assert!(matches!(
            f.run(SEED).await,
            Err(RunLotteryError::NoApplicants)
        ));
        assert_eq!(f.lottery().await.stage, SelectionStage::NotDrawn);
    }

    #[tokio::test]
    async fn missing_course_is_not_found() {
        let f = fixture(2, 1).await;
        let missing = f.course_id + 100;

        assert!(course_lottery(&f.pool, missing).await.unwrap().is_none());
        assert!(matches!(
            run(&f.pool, missing, f.staff_id, LotterySeed::new(SEED), now()).await,
            Err(RunLotteryError::CourseNotFound)
        ));
        assert!(matches!(
            close(&f.pool, missing, f.staff_id, now()).await,
            Err(CloseSelectionError::CourseNotFound)
        ));
    }

    #[tokio::test]
    async fn br_s2_only_submitted_applications_are_drawn() {
        let f = fixture(2, 2).await;
        let member_id = insert_member(&f.pool, "09999").await.unwrap();
        insert_application(&f.pool, member_id, f.course_id, "withdrawn", f.staff_id)
            .await
            .unwrap();

        f.run(SEED).await.unwrap();

        assert_eq!(f.lottery().await.entrants.len(), 2);
    }

    #[tokio::test]
    async fn stages_in_term_lists_every_course() {
        let f = fixture(2, 1).await;
        let subject_id = insert_subject(&f.pool, "한글교실", "한글교실", None)
            .await
            .unwrap();
        let other = insert_course(&f.pool, f.term_id, subject_id, None, None)
            .await
            .unwrap();
        f.run(SEED).await.unwrap();

        let stages = stages_in_term(&f.pool, f.term_id).await.unwrap();

        assert_eq!(
            stages,
            [
                CourseStage {
                    course_id: f.course_id,
                    stage: SelectionStage::Drawn
                },
                CourseStage {
                    course_id: other,
                    stage: SelectionStage::NotDrawn
                },
            ]
        );
    }
}
