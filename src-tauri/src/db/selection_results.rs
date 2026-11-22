//! 선발 결과: 마감한 강좌의 당첨자, 대기자, 포기한 사람(BR-C3, BR-W1~W8).
//!
//! 대기 순번은 추첨 기록의 순서에서 계산한다(BR-W6). 연락할 차례도 저장하지 않고 그때그때
//! 정한다(BR-W1).

#![cfg_attr(not(test), expect(dead_code, reason = "선발 결과 API를 붙이기 전"))]

use baeum_domain::{
    Capacity, ContactResult, Personal, Waiting, WithdrawnFrom, contact_turns, open_seats,
};
use sqlx::SqlitePool;

use super::lotteries::{active_run, decode_error};

/// 마감한 강좌 하나의 인원.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClosedCourse {
    pub course_id: i64,
    pub capacity: Capacity,
    pub selected: usize,
    pub waitlisted: usize,
    pub open_seats: usize,
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
    /// 추첨 때 받은 대기 순번. 1부터(BR-W6).
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
    /// 자리 보장(id 순) 다음에 뽑힌 순서.
    pub selected: Vec<SelectedEntry>,
    /// 대기 순번 순서.
    pub waitlisted: Vec<WaitingEntry>,
    /// 뽑힌 순서.
    pub withdrawn: Vec<WithdrawnEntry>,
}

/// 선발 결과를 읽지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum CourseResultError {
    #[error("강좌를 찾지 못했습니다")]
    CourseNotFound,
    /// 포기와 승격은 선발 마감 뒤에만 한다(BR-C3).
    #[error("선발을 마감하지 않은 강좌입니다")]
    NotClosed,
    #[error("선발 결과를 읽지 못했습니다: {0}")]
    Db(#[from] sqlx::Error),
}

/// 학기에서 선발을 마감한 강좌들의 인원. 강좌 id 순서.
pub async fn closed_courses(
    pool: &SqlitePool,
    term_id: i64,
) -> Result<Vec<ClosedCourse>, sqlx::Error> {
    let rows: Vec<(i64, i64, i64, i64)> = sqlx::query_as(
        "SELECT c.id, c.capacity,
                (SELECT COUNT(*) FROM applications a
                 WHERE a.course_id = c.id AND a.status IN ('selected', 'confirmed')),
                (SELECT COUNT(*) FROM applications a
                 WHERE a.course_id = c.id AND a.status = 'waitlisted')
         FROM courses c JOIN selection_closures s ON s.course_id = c.id
         WHERE c.term_id = ?
         ORDER BY c.id",
    )
    .bind(term_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|(course_id, capacity, selected, waitlisted)| {
            let capacity = course_capacity(capacity)?;
            let selected = count(selected)?;
            Ok(ClosedCourse {
                course_id,
                capacity,
                selected,
                waitlisted: count(waitlisted)?,
                open_seats: open_seats(capacity, selected),
            })
        })
        .collect()
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
    let run = match (closed, active_run(&mut conn, course_id).await?) {
        (true, Some(run)) => run,
        _ => return Err(CourseResultError::NotClosed),
    };
    let capacity = course_capacity(capacity)?;

    // 자리 보장(id 순, 순서 없음) 다음에 추첨 대상(뽑힌 순서)
    let rows: Vec<ResultRow> = sqlx::query_as(
        "SELECT a.id AS application_id, m.name, m.member_no, m.phone_number, a.status,
                NULL AS position,
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
         SELECT a.id, m.name, m.member_no, m.phone_number, a.status, e.position,
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
         ORDER BY sort_group, sort_key",
    )
    .bind(run.id)
    .fetch_all(&mut *conn)
    .await?;

    // BR-W6: 추첨 때 당첨된 수. 그 뒤의 뽑힌 순서가 대기 순번이다
    let guaranteed = rows.iter().filter(|row| row.position.is_none()).count();
    let drawn = rows.len() - guaranteed;
    let drawn_selected = drawn.min(run.capacity()?.saturating_sub(guaranteed));
    let waitlist_number = |row: &ResultRow| -> Result<Option<usize>, sqlx::Error> {
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
                guaranteed: row.position.is_none(),
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
    /// 추첨 대상의 뽑힌 순서. 자리 보장이면 없다.
    position: Option<i64>,
    last_contact: Option<String>,
    withdrawn_from: Option<String>,
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

fn count(value: i64) -> Result<usize, sqlx::Error> {
    usize::try_from(value).map_err(|_| decode_error("인원이 범위 밖입니다"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_temp;
    use crate::db::test_support::{
        insert_course, insert_drawn_course, insert_selection_change, insert_subject,
    };

    /// 정원 2명 강좌에 신청 5건을 추첨했다. 마감하면 당첨 2명, 대기 3명.
    struct Fixture {
        _dir: tempfile::TempDir,
        pool: SqlitePool,
        term_id: i64,
        course_id: i64,
        staff_id: i64,
    }

    async fn fixture(close: bool) -> Fixture {
        let (dir, pool) = open_temp().await;
        let drawn = insert_drawn_course(&pool, 2, 5, close).await;
        Fixture {
            _dir: dir,
            pool,
            term_id: drawn.term_id,
            course_id: drawn.course_id,
            staff_id: drawn.staff_id,
        }
    }

    impl Fixture {
        async fn result(&self) -> CourseResult {
            course_result(&self.pool, self.course_id).await.unwrap()
        }

        /// 상태를 바꾸고 그 기록을 남긴다. 저장 함수가 생기기 전이라 DB에 바로 넣는다.
        async fn change(&self, application_id: i64, kind: &str, from: &str, to: &str) {
            sqlx::query("UPDATE applications SET status = ? WHERE id = ?")
                .bind(to)
                .bind(application_id)
                .execute(&self.pool)
                .await
                .unwrap();
            insert_selection_change(
                &self.pool,
                application_id,
                kind,
                from,
                to,
                None,
                self.staff_id,
            )
            .await
            .unwrap();
        }
    }

    fn ids<'a>(members: impl IntoIterator<Item = &'a ResultMember>) -> Vec<i64> {
        members.into_iter().map(|m| m.application_id).collect()
    }

    #[tokio::test]
    async fn br_w6_waitlist_numbers_follow_draw_order() {
        let f = fixture(true).await;

        let result = f.result().await;

        assert_eq!(result.selected.len(), 2);
        assert_eq!(result.open_seats, 0);
        let numbers: Vec<usize> = result
            .waitlisted
            .iter()
            .map(|w| w.waitlist_number)
            .collect();
        assert_eq!(numbers, [1, 2, 3]);
        assert!(result.waitlisted.iter().all(|w| !w.contact_turn));
        assert!(
            result.waitlisted[0]
                .phone_number
                .expose()
                .starts_with("010")
        );
        assert!(result.withdrawn.is_empty());
    }

    #[tokio::test]
    async fn br_w1_withdrawal_opens_seat_and_first_waiting_is_turn() {
        let f = fixture(true).await;
        let before = f.result().await;
        let leaving = before.selected[0].member.application_id;

        f.change(leaving, "withdrawn", "selected", "withdrawn")
            .await;

        let result = f.result().await;
        assert_eq!(result.open_seats, 1);
        let turns: Vec<bool> = result.waitlisted.iter().map(|w| w.contact_turn).collect();
        assert_eq!(turns, [true, false, false]);
        assert_eq!(ids(result.withdrawn.iter().map(|w| &w.member)), [leaving]);
        assert_eq!(result.withdrawn[0].withdrawn_from, WithdrawnFrom::Selected);
        assert_eq!(result.withdrawn[0].waitlist_number, None);
    }

    #[tokio::test]
    async fn br_w5_unreachable_passes_turn_and_promoted_keeps_number() {
        let f = fixture(true).await;
        let before = f.result().await;
        let first = before.waitlisted[0].member.application_id;
        let second = before.waitlisted[1].member.application_id;
        f.change(
            before.selected[0].member.application_id,
            "withdrawn",
            "selected",
            "withdrawn",
        )
        .await;

        f.change(first, "unreachable", "waitlisted", "waitlisted")
            .await;
        let skipped = f.result().await;
        f.change(second, "accepted", "waitlisted", "selected").await;
        let promoted = f.result().await;

        assert_eq!(
            skipped.waitlisted[0].last_contact,
            Some(ContactResult::Unreachable)
        );
        assert!(!skipped.waitlisted[0].contact_turn);
        assert!(skipped.waitlisted[1].contact_turn);
        assert_eq!(promoted.open_seats, 0);
        let promoted_entry = promoted
            .selected
            .iter()
            .find(|s| s.member.application_id == second)
            .unwrap();
        assert_eq!(promoted_entry.waitlist_number, Some(2));
        // 남은 대기자의 번호는 그대로다
        let numbers: Vec<usize> = promoted
            .waitlisted
            .iter()
            .map(|w| w.waitlist_number)
            .collect();
        assert_eq!(numbers, [1, 3]);
    }

    #[tokio::test]
    async fn br_c3_drawn_but_not_closed_is_refused() {
        let f = fixture(false).await;

        let error = course_result(&f.pool, f.course_id).await.unwrap_err();
        let missing = course_result(&f.pool, 999).await.unwrap_err();

        assert!(matches!(error, CourseResultError::NotClosed), "{error}");
        assert!(
            matches!(missing, CourseResultError::CourseNotFound),
            "{missing}"
        );
    }

    #[tokio::test]
    async fn closed_courses_lists_only_closed_with_counts() {
        let f = fixture(true).await;
        let subject_id = insert_subject(&f.pool, "서예", "서예", None).await.unwrap();
        insert_course(&f.pool, f.term_id, subject_id, None, None)
            .await
            .unwrap();

        let courses = closed_courses(&f.pool, f.term_id).await.unwrap();

        assert_eq!(courses.len(), 1);
        assert_eq!(courses[0].course_id, f.course_id);
        assert_eq!((courses[0].selected, courses[0].waitlisted), (2, 3));
        assert_eq!(courses[0].open_seats, 0);
    }
}
