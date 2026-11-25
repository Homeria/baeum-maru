//! 추첨(BR-S6, BR-S7), 자리 보장(BR-S4, BR-S5), 재추첨(BR-L4), 선발 마감(BR-L5)과
//! 추첨 화면에 보여 줄 신청자.
//!
//! 시드는 부른 쪽이 만들어 넘긴다(ADR 0016). 호스트 PC에서만 부르는지(BR-L6)와 역할은
//! 서버가 확인한다.

use baeum_domain::{Capacity, LotterySeed, LotteryVersion, Personal};
use sqlx::{SqliteConnection, SqlitePool};
use time::OffsetDateTime;

use crate::infra::db::{decode_error, timestamp_text};

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

/// 신청에 자리 보장 사유가 붙었는지 읽는 SELECT 칸. 신청 테이블은 `a`.
macro_rules! guaranteed_column {
    () => {
        "EXISTS (SELECT 1 FROM application_priorities p
                 JOIN priority_reasons pr ON pr.id = p.reason_id
                 WHERE p.application_id = a.id AND pr.method = 'guaranteed') AS guaranteed"
    };
}

/// 추첨 화면의 신청자 한 명.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entrant {
    pub application_id: i64,
    pub member_name: Personal<String>,
    pub member_no: Personal<String>,
    /// 자리 보장 사유가 붙었다(BR-S4). 추첨 전에는 붙은 사유로, 추첨 뒤에는 기록으로 정한다.
    pub guaranteed: bool,
}

/// 추첨 화면에 보여 줄 강좌 하나의 선발.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CourseLottery {
    pub capacity: Capacity,
    pub stage: SelectionStage,
    /// 자리 보장 신청이 언제나 맨 앞이다. 그 뒤는 추첨 전이면 접수한 순서, 추첨 뒤면 뽑힌 순서.
    pub entrants: Vec<Entrant>,
    /// 추첨 뒤 `entrants`의 앞에서 몇 명이 당첨인지(자리 보장 포함). 나머지는 차례로
    /// 대기 순번 1, 2, … 추첨 전이면 0.
    pub selected_count: usize,
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
            // 자리 보장(id 순) 다음에 추첨 대상(뽑힌 순서)
            let rows: Vec<EntrantRow> = sqlx::query_as(
                "SELECT a.id AS application_id, m.name, m.member_no, 1 AS guaranteed,
                        a.id AS sort_key
                 FROM lottery_guaranteed_entries g
                 JOIN applications a ON a.id = g.application_id
                 JOIN members m ON m.id = a.member_id
                 WHERE g.lottery_run_id = ?1
                 UNION ALL
                 SELECT a.id, m.name, m.member_no, 0, e.position
                 FROM lottery_entries e
                 JOIN applications a ON a.id = e.application_id
                 JOIN members m ON m.id = a.member_id
                 WHERE e.lottery_run_id = ?1
                 ORDER BY guaranteed DESC, sort_key",
            )
            .bind(run.id)
            .fetch_all(&mut *conn)
            .await?;
            let guaranteed = rows.iter().filter(|row| row.guaranteed).count();
            // BR-S4: 남은 자리만큼 추첨 대상에서 당첨
            let remaining = run.capacity()?.saturating_sub(guaranteed);
            let drawn = rows.len() - guaranteed;
            (rows, guaranteed + drawn.min(remaining))
        }
        None => {
            let rows: Vec<EntrantRow> = sqlx::query_as(concat!(
                "SELECT a.id AS application_id, m.name, m.member_no, ",
                guaranteed_column!(),
                " FROM applications a JOIN members m ON m.id = a.member_id
                 WHERE a.course_id = ? AND a.status = 'submitted'
                 ORDER BY guaranteed DESC, a.submitted_at, a.id"
            ))
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

/// 강좌의 접수됨 신청과 자리 보장 사유가 붙었는지. 접수한 순서.
pub(super) async fn submitted_applications(
    conn: &mut SqliteConnection,
    course_id: i64,
) -> Result<Vec<(i64, bool)>, sqlx::Error> {
    sqlx::query_as(concat!(
        "SELECT a.id, ",
        guaranteed_column!(),
        " FROM applications a WHERE a.course_id = ? AND a.status = 'submitted'"
    ))
    .bind(course_id)
    .fetch_all(conn)
    .await
}

/// 추첨 기록을 남기고 id를 돌려준다. `now`는 [`timestamp_text`] 글자.
pub(super) async fn insert_run(
    conn: &mut SqliteConnection,
    course_id: i64,
    staff_id: i64,
    now: &str,
    seed: LotterySeed,
    version: LotteryVersion,
    capacity: Capacity,
) -> Result<i64, sqlx::Error> {
    Ok(sqlx::query(
        "INSERT INTO lottery_runs (course_id, run_by, run_at, seed, version, capacity)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(course_id)
    .bind(staff_id)
    .bind(now)
    .bind(seed.to_hex())
    .bind(version.number())
    .bind(i64::from(capacity.value()))
    .execute(conn)
    .await?
    .last_insert_rowid())
}

/// 자리 보장으로 당첨된 신청을 추첨 기록에 남긴다(BR-S4).
pub(super) async fn insert_guaranteed_entry(
    conn: &mut SqliteConnection,
    run_id: i64,
    application_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO lottery_guaranteed_entries (lottery_run_id, application_id) VALUES (?, ?)",
    )
    .bind(run_id)
    .bind(application_id)
    .execute(conn)
    .await?;
    Ok(())
}

/// 뽑힌 신청을 추첨 기록에 순서와 함께 남긴다. `position`은 1부터.
pub(super) async fn insert_entry(
    conn: &mut SqliteConnection,
    run_id: i64,
    application_id: i64,
    position: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO lottery_entries (lottery_run_id, application_id, position)
         VALUES (?, ?, ?)",
    )
    .bind(run_id)
    .bind(application_id)
    .bind(position)
    .execute(conn)
    .await?;
    Ok(())
}

/// 추첨 결과로 신청 상태를 당첨이나 대기로 바꾼다.
pub(super) async fn set_drawn_status(
    conn: &mut SqliteConnection,
    application_id: i64,
    selected: bool,
) -> Result<(), sqlx::Error> {
    let status = if selected { "selected" } else { "waitlisted" };
    sqlx::query("UPDATE applications SET status = ? WHERE id = ?")
        .bind(status)
        .bind(application_id)
        .execute(conn)
        .await?;
    Ok(())
}

/// 선발 마감을 남긴다(BR-L5).
pub(super) async fn insert_closure(
    conn: &mut SqliteConnection,
    course_id: i64,
    staff_id: i64,
    now: OffsetDateTime,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO selection_closures (course_id, closed_by, closed_at) VALUES (?, ?, ?)",
    )
    .bind(course_id)
    .bind(staff_id)
    .bind(timestamp_text(now))
    .execute(conn)
    .await?;
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
pub(super) struct CourseRow {
    capacity: i64,
    drawn: bool,
    closed: bool,
}

impl CourseRow {
    pub(super) fn capacity(&self) -> Result<Capacity, sqlx::Error> {
        Capacity::new(self.capacity).map_err(|_| decode_error("정원이 범위 밖입니다"))
    }

    pub(super) fn stage(&self) -> SelectionStage {
        stage_of(self.drawn, self.closed)
    }
}

pub(super) async fn course_row(
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
pub(crate) struct RunRow {
    pub(crate) id: i64,
    capacity: i64,
}

impl RunRow {
    /// 추첨한 때의 정원.
    pub(crate) fn capacity(&self) -> Result<usize, sqlx::Error> {
        usize::try_from(self.capacity).map_err(|_| decode_error("추첨 기록의 정원이 범위 밖입니다"))
    }
}

/// 강좌의 취소되지 않은 추첨 기록에 취소 표시를 하고, 그 추첨으로 당첨·대기가 된 신청을
/// 접수됨으로 되돌린다(BR-L4, BR-S13). 취소할 기록이 있었으면 `true`.
///
/// `now`는 [`timestamp_text`] 글자. 부르는 쪽이 `BEGIN IMMEDIATE` 트랜잭션 안에서 부른다.
pub(crate) async fn cancel_active_run(
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

/// 강좌의 취소되지 않은 추첨 기록. 없으면 `None`.
pub(crate) async fn active_run(
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
    guaranteed: bool,
}

impl From<EntrantRow> for Entrant {
    fn from(row: EntrantRow) -> Self {
        Entrant {
            application_id: row.application_id,
            member_name: Personal::new(row.name),
            member_no: Personal::new(row.member_no),
            guaranteed: row.guaranteed,
        }
    }
}
