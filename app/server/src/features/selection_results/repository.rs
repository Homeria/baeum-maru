//! 선발 결과: 마감한 강좌의 당첨자, 대기자, 포기한 사람(BR-C3, BR-W1~W8).
//!
//! 대기 순번은 추첨 기록의 순서에서 계산하고, 추가 신청은 받을 때 정해 둔 순번을 쓴다(BR-W6,
//! BR-X2). 연락할 차례는 저장하지 않고 그때그때 정한다(BR-W1).

pub(crate) mod changes;

use baeum_domain::{
    AdditionalOutcome, Capacity, ContactResult, Personal, Waiting, WithdrawnFrom,
    additional_outcome, contact_turns, open_seats,
};
use sqlx::{SqliteConnection, SqlitePool};

use super::error::CourseResultError;
use crate::features::lotteries::repository::active_run;
use crate::infra::db::decode_error;

/// 마감한 강좌 하나의 인원.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClosedCourse {
    pub course_id: i64,
    pub capacity: Capacity,
    pub selected: usize,
    pub waitlisted: usize,
    pub open_seats: usize,
    /// 지금 추가 신청을 하나 받으면 어떻게 되는지(BR-X2). 화면이 미리 보여 준다.
    pub next_additional: AdditionalOutcome,
}

/// 결과 목록의 회원 한 명.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultMember {
    pub application_id: i64,
    pub member_name: Personal<String>,
    pub member_no: Personal<String>,
}

/// 당첨자 한 명.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedEntry {
    pub member: ResultMember,
    /// 자리 보장으로 당첨됐다(BR-S4).
    pub guaranteed: bool,
    /// 대기에서 승격했으면 그때의 대기 순번.
    pub waitlist_number: Option<usize>,
}

/// 대기자 한 명. 대기 순번 순서로 늘어놓는다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaitingEntry {
    pub member: ResultMember,
    /// 연락에 쓴다(BR-W2). 누구에게 보여 줄지는 서버가 정한다.
    pub phone_number: Personal<String>,
    /// 추첨이나 추가 신청 때 받은 대기 순번. 1부터(BR-W6).
    pub waitlist_number: usize,
    pub last_contact: Option<ContactResult>,
    /// 지금 연락할 차례(BR-W1).
    pub contact_turn: bool,
}

/// 포기한 사람 한 명.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WithdrawnEntry {
    pub member: ResultMember,
    /// 포기 전 상태. 잘못 처리 바로잡기가 이 상태로 되돌린다(BR-W8).
    pub withdrawn_from: WithdrawnFrom,
    /// 추첨 때 받은 대기 순번. 당첨에서 바로 포기했으면 없다.
    pub waitlist_number: Option<usize>,
}

/// 마감한 강좌 하나의 선발 결과.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CourseResult {
    /// 지금 정원.
    pub capacity: Capacity,
    pub open_seats: usize,
    /// 자리 보장(id 순), 뽑힌 순서, 추가 신청(받은 순서).
    pub selected: Vec<SelectedEntry>,
    /// 대기 순번 순서.
    pub waitlisted: Vec<WaitingEntry>,
    /// 뽑힌 순서, 그다음 추가 신청(받은 순서).
    pub withdrawn: Vec<WithdrawnEntry>,
}

/// 학기에서 선발을 마감한 강좌의 id. 강좌 id 순서.
pub(super) async fn closed_course_ids(
    conn: &mut SqliteConnection,
    term_id: i64,
) -> Result<Vec<i64>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT c.id FROM courses c JOIN selection_closures s ON s.course_id = c.id
         WHERE c.term_id = ?
         ORDER BY c.id",
    )
    .bind(term_id)
    .fetch_all(conn)
    .await
}

/// 이 선발 결과에 추가 신청을 하나 더 받으면 어떻게 되는지(BR-X2).
///
/// 남은 대기자 수와 지금까지 쓴 가장 큰 대기 순번을 센다. 포기한 사람과 승격한 사람의 순번도
/// 쓴 번호다(BR-W6).
pub fn next_additional(result: &CourseResult) -> AdditionalOutcome {
    let last_waitlist_number = result
        .selected
        .iter()
        .filter_map(|entry| entry.waitlist_number)
        .chain(result.waitlisted.iter().map(|entry| entry.waitlist_number))
        .chain(
            result
                .withdrawn
                .iter()
                .filter_map(|entry| entry.waitlist_number),
        )
        .max();
    additional_outcome(
        result.open_seats,
        result.waitlisted.len(),
        last_waitlist_number,
    )
}

/// 마감한 강좌 하나의 선발 결과.
///
/// # Errors
///
/// 강좌가 없으면 [`CourseResultError::CourseNotFound`], 마감 전이면 [`CourseResultError::NotClosed`].
pub async fn course_result(
    pool: &SqlitePool,
    course_id: i64,
) -> Result<CourseResult, CourseResultError> {
    let mut conn = pool.acquire().await?;
    course_result_in(&mut conn, course_id).await
}

/// [`course_result`]를 주어진 연결에서 읽는다. 여러 강좌를 한 트랜잭션에서 읽을 때 쓴다.
pub(crate) async fn course_result_in(
    conn: &mut SqliteConnection,
    course_id: i64,
) -> Result<CourseResult, CourseResultError> {
    let course: Option<(i64, bool)> = sqlx::query_as(
        "SELECT c.capacity, EXISTS (SELECT 1 FROM selection_closures s WHERE s.course_id = c.id)
         FROM courses c WHERE c.id = ?",
    )
    .bind(course_id)
    .fetch_optional(&mut *conn)
    .await?;
    let Some((capacity, closed)) = course else {
        return Err(CourseResultError::CourseNotFound);
    };
    // 마감은 추첨 기록이 있어야 할 수 있다(lotteries::close)
    let run = match (closed, active_run(&mut *conn, course_id).await?) {
        (true, Some(run)) => run,
        _ => return Err(CourseResultError::NotClosed),
    };
    let capacity = course_capacity(capacity)?;

    // 자리 보장(id 순, 순서 없음), 추첨 대상(뽑힌 순서), 추가 신청(받은 순서. 대기 순번도 이 순서다)
    let rows: Vec<ResultRow> = sqlx::query_as(
        "SELECT a.id AS application_id, m.name, m.member_no, m.phone_number, a.status,
                NULL AS position, NULL AS additional_number, 0 AS additional,
                (SELECT x.kind FROM selection_changes x
                 WHERE x.application_id = a.id
                   AND x.kind IN ('accepted', 'declined', 'unreachable')
                 ORDER BY x.id DESC LIMIT 1) AS last_contact,
                (SELECT x.from_status FROM selection_changes x
                 WHERE x.application_id = a.id AND x.to_status = 'withdrawn'
                 ORDER BY x.id DESC LIMIT 1) AS withdrawn_from,
                0 AS sort_group, a.id AS sort_key
         FROM lottery_guaranteed_entries g
         JOIN applications a ON a.id = g.application_id
         JOIN members m ON m.id = a.member_id
         WHERE g.lottery_run_id = ?1
         UNION ALL
         SELECT a.id, m.name, m.member_no, m.phone_number, a.status, e.position, NULL, 0,
                (SELECT x.kind FROM selection_changes x
                 WHERE x.application_id = a.id
                   AND x.kind IN ('accepted', 'declined', 'unreachable')
                 ORDER BY x.id DESC LIMIT 1),
                (SELECT x.from_status FROM selection_changes x
                 WHERE x.application_id = a.id AND x.to_status = 'withdrawn'
                 ORDER BY x.id DESC LIMIT 1),
                1, e.position
         FROM lottery_entries e
         JOIN applications a ON a.id = e.application_id
         JOIN members m ON m.id = a.member_id
         WHERE e.lottery_run_id = ?1
         UNION ALL
         SELECT a.id, m.name, m.member_no, m.phone_number, a.status, NULL, x.waitlist_number, 1,
                (SELECT c.kind FROM selection_changes c
                 WHERE c.application_id = a.id
                   AND c.kind IN ('accepted', 'declined', 'unreachable')
                 ORDER BY c.id DESC LIMIT 1),
                (SELECT c.from_status FROM selection_changes c
                 WHERE c.application_id = a.id AND c.to_status = 'withdrawn'
                 ORDER BY c.id DESC LIMIT 1),
                2, a.id
         FROM additional_applications x
         JOIN applications a ON a.id = x.application_id
         JOIN members m ON m.id = a.member_id
         WHERE a.course_id = ?2
         ORDER BY sort_group, sort_key",
    )
    .bind(run.id)
    .bind(course_id)
    .fetch_all(&mut *conn)
    .await?;

    // BR-W6: 추첨 때 당첨된 수. 그 뒤의 뽑힌 순서가 대기 순번이다
    let guaranteed = rows.iter().filter(|row| row.is_guaranteed()).count();
    let drawn = rows.iter().filter(|row| row.position.is_some()).count();
    let drawn_selected = drawn.min(run.capacity()?.saturating_sub(guaranteed));
    let waitlist_number = |row: &ResultRow| -> Result<Option<usize>, sqlx::Error> {
        // BR-X2: 추가 신청은 받을 때 정한 순번
        if row.additional {
            return row
                .additional_number
                .map(|n| usize::try_from(n).map_err(|_| decode_error("대기 순번이 범위 밖입니다")))
                .transpose();
        }
        let Some(position) = row.position else {
            return Ok(None);
        };
        let position =
            usize::try_from(position).map_err(|_| decode_error("뽑힌 순서가 범위 밖입니다"))?;
        Ok(position.checked_sub(drawn_selected).filter(|&n| n > 0))
    };

    let mut selected = Vec::new();
    let mut waiting = Vec::new();
    let mut withdrawn = Vec::new();
    for row in rows {
        let number = waitlist_number(&row)?;
        // 아래에서 이름과 번호를 옮겨 가기 전에 읽어 둔다
        let guaranteed = row.is_guaranteed();
        let last_contact = row
            .last_contact
            .as_deref()
            .map(|code| {
                ContactResult::from_code(code).ok_or_else(|| decode_error("모르는 연락 결과입니다"))
            })
            .transpose()?;
        let withdrawn_from = row
            .withdrawn_from
            .as_deref()
            .map(withdrawn_from)
            .transpose()?;
        let member = ResultMember {
            application_id: row.application_id,
            member_name: Personal::new(row.name),
            member_no: Personal::new(row.member_no),
        };
        match row.status.as_str() {
            "selected" | "confirmed" => selected.push(SelectedEntry {
                member,
                guaranteed,
                waitlist_number: number,
            }),
            "waitlisted" => waiting.push(WaitingEntry {
                member,
                phone_number: Personal::new(row.phone_number),
                waitlist_number: number.ok_or_else(|| decode_error("대기 순번이 없습니다"))?,
                last_contact,
                contact_turn: false,
            }),
            "withdrawn" => withdrawn.push(WithdrawnEntry {
                member,
                withdrawn_from: withdrawn_from
                    .ok_or_else(|| decode_error("포기 기록이 없습니다"))?,
                waitlist_number: number,
            }),
            _ => return Err(decode_error("마감한 강좌에 선발 전 신청이 있습니다").into()),
        }
    }

    let open = open_seats(capacity, selected.len());
    let turns = contact_turns(
        &waiting
            .iter()
            .map(|entry| Waiting {
                application_id: entry.member.application_id,
                last_contact: entry.last_contact,
            })
            .collect::<Vec<_>>(),
        open,
    );
    for entry in &mut waiting {
        entry.contact_turn = turns.contains(&entry.member.application_id);
    }

    Ok(CourseResult {
        capacity,
        open_seats: open,
        selected,
        waitlisted: waiting,
        withdrawn,
    })
}

#[derive(sqlx::FromRow)]
struct ResultRow {
    application_id: i64,
    name: String,
    member_no: String,
    phone_number: String,
    status: String,
    /// 추첨 대상의 뽑힌 순서. 자리 보장과 추가 신청이면 없다.
    position: Option<i64>,
    /// 추가 신청을 대기로 받았을 때 정한 대기 순번.
    additional_number: Option<i64>,
    /// 추가 신청이다(BR-X2).
    additional: bool,
    last_contact: Option<String>,
    withdrawn_from: Option<String>,
}

impl ResultRow {
    /// 자리 보장으로 당첨됐다(BR-S4). 추첨 대상도 추가 신청도 아니다.
    fn is_guaranteed(&self) -> bool {
        self.position.is_none() && !self.additional
    }
}

fn withdrawn_from(status: &str) -> Result<WithdrawnFrom, sqlx::Error> {
    match status {
        "selected" => Ok(WithdrawnFrom::Selected),
        "waitlisted" => Ok(WithdrawnFrom::Waitlisted),
        _ => Err(decode_error("포기 전 상태가 범위 밖입니다")),
    }
}

fn course_capacity(capacity: i64) -> Result<Capacity, sqlx::Error> {
    Capacity::new(capacity).map_err(|_| decode_error("정원이 범위 밖입니다"))
}
