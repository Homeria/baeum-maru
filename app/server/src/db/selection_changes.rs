//! 선발 마감 뒤 포기(BR-C3), 대기자 연락 결과(BR-W2~W5), 잘못 처리 바로잡기(BR-W8).
//!
//! 바꿀 때마다 선발 결과 변경 기록을 남긴다(BR-W7). 누가 할 수 있는지는 서버가 확인한다.
//! 읽고 확인한 뒤 쓰므로 모두 `BEGIN IMMEDIATE`로 한다 (ADR 0005).

use baeum_domain::{Capacity, ContactResult, WithdrawnFrom, open_seats};
use sqlx::{SqliteConnection, SqlitePool};
use time::OffsetDateTime;

use crate::infra::db::decode_error;
use crate::infra::db::timestamp_text;

/// 선발 결과를 바꾸지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum ChangeSelectionError {
    #[error("신청을 찾지 못했습니다")]
    ApplicationNotFound,
    /// 포기와 승격은 선발 마감 뒤에만 한다(BR-C3).
    #[error("선발을 마감하지 않은 강좌입니다")]
    NotClosed,
    /// 이 일을 할 수 없는 상태다. 예: 대기자가 아닌 신청에 연락 결과
    #[error("지금 상태에서는 할 수 없습니다")]
    WrongStatus,
    /// 빈자리가 없어 당첨으로 바꿀 수 없다(BR-W3, BR-W8).
    #[error("빈자리가 없습니다")]
    NoOpenSeat,
    #[error("선발 결과를 바꾸지 못했습니다: {0}")]
    Db(#[from] sqlx::Error),
}

/// 당첨이나 대기인 신청을 포기로 바꾼다. 사유는 선택이다(BR-C3).
pub async fn withdraw(
    pool: &SqlitePool,
    application_id: i64,
    reason: Option<&str>,
    staff_id: i64,
    now: OffsetDateTime,
) -> Result<(), ChangeSelectionError> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let target = target(&mut tx, application_id).await?;
    let from = match target.status.as_str() {
        "selected" => "selected",
        "waitlisted" => "waitlisted",
        _ => return Err(ChangeSelectionError::WrongStatus),
    };
    let change = Change {
        application_id,
        kind: "withdrawn",
        from,
        to: "withdrawn",
        reason,
    };
    apply(&mut tx, &change, staff_id, now).await?;
    tx.commit().await?;
    Ok(())
}

/// 대기자에게 연락한 결과를 남긴다(BR-W2).
///
/// 수락은 빈자리가 있을 때만 당첨으로 바꾼다(BR-W3). 거절은 포기(BR-W4), 연락 안 됨은
/// 대기 그대로다(BR-W5). 연락할 차례가 아닌 대기자도 받는다.
pub async fn record_contact(
    pool: &SqlitePool,
    application_id: i64,
    result: ContactResult,
    staff_id: i64,
    now: OffsetDateTime,
) -> Result<(), ChangeSelectionError> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let target = target(&mut tx, application_id).await?;
    if target.status != "waitlisted" {
        return Err(ChangeSelectionError::WrongStatus);
    }
    let to = match result {
        ContactResult::Accepted => {
            if target.open_seats(&mut tx).await? == 0 {
                return Err(ChangeSelectionError::NoOpenSeat);
            }
            "selected"
        }
        ContactResult::Declined => "withdrawn",
        ContactResult::Unreachable => "waitlisted",
    };
    let change = Change {
        application_id,
        kind: result.code(),
        from: "waitlisted",
        to,
        reason: None,
    };
    apply(&mut tx, &change, staff_id, now).await?;
    tx.commit().await?;
    Ok(())
}

/// 잘못 처리한 포기를 포기 전 상태로 되돌린다(BR-W8). 사유는 필수다.
///
/// 당첨이었으면 빈자리가 있을 때만 되돌린다. 대기였으면 원래 대기 순번으로 돌아간다
/// (대기 순번은 추첨 기록에서 나오므로 따로 할 일이 없다).
pub async fn correct_withdrawal(
    pool: &SqlitePool,
    application_id: i64,
    reason: &str,
    staff_id: i64,
    now: OffsetDateTime,
) -> Result<(), ChangeSelectionError> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let target = target(&mut tx, application_id).await?;
    if target.status != "withdrawn" {
        return Err(ChangeSelectionError::WrongStatus);
    }
    let from: Option<String> = sqlx::query_scalar(
        "SELECT from_status FROM selection_changes
         WHERE application_id = ? AND to_status = 'withdrawn'
         ORDER BY id DESC LIMIT 1",
    )
    .bind(application_id)
    .fetch_optional(&mut *tx)
    .await?;
    let withdrawn_from = match from.as_deref() {
        Some("selected") => WithdrawnFrom::Selected,
        Some("waitlisted") => WithdrawnFrom::Waitlisted,
        _ => return Err(decode_error("포기 기록이 없습니다").into()),
    };
    if !withdrawn_from.can_restore(target.open_seats(&mut tx).await?) {
        return Err(ChangeSelectionError::NoOpenSeat);
    }
    let change = Change {
        application_id,
        kind: "corrected",
        from: "withdrawn",
        to: match withdrawn_from {
            WithdrawnFrom::Selected => "selected",
            WithdrawnFrom::Waitlisted => "waitlisted",
        },
        reason: Some(reason),
    };
    apply(&mut tx, &change, staff_id, now).await?;
    tx.commit().await?;
    Ok(())
}

/// 바꿀 신청과 그 강좌.
struct Target {
    course_id: i64,
    capacity: i64,
    status: String,
}

impl Target {
    /// 강좌의 지금 빈자리(BR-W1).
    async fn open_seats(&self, conn: &mut SqliteConnection) -> Result<usize, sqlx::Error> {
        let selected: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM applications
             WHERE course_id = ? AND status IN ('selected', 'confirmed')",
        )
        .bind(self.course_id)
        .fetch_one(conn)
        .await?;
        let capacity =
            Capacity::new(self.capacity).map_err(|_| decode_error("정원이 범위 밖입니다"))?;
        let selected =
            usize::try_from(selected).map_err(|_| decode_error("인원이 범위 밖입니다"))?;
        Ok(open_seats(capacity, selected))
    }
}

/// 신청을 찾고 선발을 마감한 강좌인지 확인한다.
async fn target(
    conn: &mut SqliteConnection,
    application_id: i64,
) -> Result<Target, ChangeSelectionError> {
    let row: Option<(i64, i64, String, bool)> = sqlx::query_as(
        "SELECT c.id, c.capacity, a.status,
                EXISTS (SELECT 1 FROM selection_closures s WHERE s.course_id = c.id)
         FROM applications a JOIN courses c ON c.id = a.course_id
         WHERE a.id = ?",
    )
    .bind(application_id)
    .fetch_optional(conn)
    .await?;
    let Some((course_id, capacity, status, closed)) = row else {
        return Err(ChangeSelectionError::ApplicationNotFound);
    };
    if !closed {
        return Err(ChangeSelectionError::NotClosed);
    }
    Ok(Target {
        course_id,
        capacity,
        status,
    })
}

/// 신청 상태 하나의 바뀜. 모두 DB 글자다.
struct Change<'a> {
    application_id: i64,
    kind: &'a str,
    from: &'a str,
    to: &'a str,
    reason: Option<&'a str>,
}

/// 신청 상태를 바꾸고 기록을 남긴다(BR-W7).
async fn apply(
    conn: &mut SqliteConnection,
    change: &Change<'_>,
    staff_id: i64,
    now: OffsetDateTime,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE applications SET status = ? WHERE id = ?")
        .bind(change.to)
        .bind(change.application_id)
        .execute(&mut *conn)
        .await?;
    sqlx::query(
        "INSERT INTO selection_changes
             (application_id, kind, from_status, to_status, reason, changed_by, changed_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(change.application_id)
    .bind(change.kind)
    .bind(change.from)
    .bind(change.to)
    .bind(change.reason)
    .bind(staff_id)
    .bind(timestamp_text(now))
    .execute(&mut *conn)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use time::macros::datetime;

    use super::*;
    use crate::db::selection_results::{CourseResult, course_result};
    use crate::db::test_support::insert_drawn_course;
    use crate::infra::db::open_temp;

    fn now() -> OffsetDateTime {
        datetime!(2026-11-23 01:02:03 UTC)
    }

    /// 정원 2명 강좌에 신청 5건. 당첨 2명, 대기 3명.
    struct Fixture {
        _dir: tempfile::TempDir,
        pool: SqlitePool,
        course_id: i64,
        staff_id: i64,
    }

    async fn fixture(close: bool) -> Fixture {
        let (dir, pool) = open_temp().await;
        let drawn = insert_drawn_course(&pool, 2, 5, close).await;
        Fixture {
            _dir: dir,
            pool,
            course_id: drawn.course_id,
            staff_id: drawn.staff_id,
        }
    }

    impl Fixture {
        async fn result(&self) -> CourseResult {
            course_result(&self.pool, self.course_id).await.unwrap()
        }

        async fn withdraw(
            &self,
            id: i64,
            reason: Option<&str>,
        ) -> Result<(), ChangeSelectionError> {
            withdraw(&self.pool, id, reason, self.staff_id, now()).await
        }

        async fn contact(
            &self,
            id: i64,
            result: ContactResult,
        ) -> Result<(), ChangeSelectionError> {
            record_contact(&self.pool, id, result, self.staff_id, now()).await
        }

        async fn correct(&self, id: i64) -> Result<(), ChangeSelectionError> {
            correct_withdrawal(
                &self.pool,
                id,
                "다른 회원을 잘못 누름",
                self.staff_id,
                now(),
            )
            .await
        }

        async fn records(&self) -> Vec<(i64, String, String, String, Option<String>)> {
            sqlx::query_as(
                "SELECT application_id, kind, from_status, to_status, reason
                 FROM selection_changes ORDER BY id",
            )
            .fetch_all(&self.pool)
            .await
            .unwrap()
        }
    }

    #[tokio::test]
    async fn br_c3_withdraws_selected_with_record() {
        let f = fixture(true).await;
        let leaving = f.result().await.selected[0].member.application_id;

        f.withdraw(leaving, Some("본인 전화")).await.unwrap();

        let result = f.result().await;
        assert_eq!(result.open_seats, 1);
        assert_eq!(result.withdrawn[0].member.application_id, leaving);
        assert_eq!(
            f.records().await,
            [(
                leaving,
                "withdrawn".to_owned(),
                "selected".to_owned(),
                "withdrawn".to_owned(),
                Some("본인 전화".to_owned())
            )]
        );
    }

    #[tokio::test]
    async fn br_c3_not_closed_course_is_refused() {
        let f = fixture(false).await;
        let id: i64 = sqlx::query_scalar("SELECT MIN(id) FROM applications")
            .fetch_one(&f.pool)
            .await
            .unwrap();

        let error = f.withdraw(id, None).await.unwrap_err();
        let missing = f.withdraw(999, None).await.unwrap_err();

        assert!(matches!(error, ChangeSelectionError::NotClosed), "{error}");
        assert!(
            matches!(missing, ChangeSelectionError::ApplicationNotFound),
            "{missing}"
        );
    }

    #[tokio::test]
    async fn br_w3_accept_needs_open_seat() {
        let f = fixture(true).await;
        let before = f.result().await;
        let first = before.waitlisted[0].member.application_id;

        let full = f.contact(first, ContactResult::Accepted).await.unwrap_err();
        f.withdraw(before.selected[0].member.application_id, None)
            .await
            .unwrap();
        f.contact(first, ContactResult::Accepted).await.unwrap();

        assert!(matches!(full, ChangeSelectionError::NoOpenSeat), "{full}");
        let after = f.result().await;
        assert_eq!(after.open_seats, 0);
        assert!(
            after
                .selected
                .iter()
                .any(|s| s.member.application_id == first)
        );
    }

    #[tokio::test]
    async fn br_w4_w5_declined_withdraws_and_unreachable_stays() {
        let f = fixture(true).await;
        let before = f.result().await;
        let first = before.waitlisted[0].member.application_id;
        let second = before.waitlisted[1].member.application_id;

        f.contact(first, ContactResult::Unreachable).await.unwrap();
        f.contact(second, ContactResult::Declined).await.unwrap();

        let after = f.result().await;
        assert_eq!(after.waitlisted[0].member.application_id, first);
        assert_eq!(
            after.waitlisted[0].last_contact,
            Some(ContactResult::Unreachable)
        );
        assert_eq!(after.withdrawn[0].member.application_id, second);
        assert_eq!(after.withdrawn[0].withdrawn_from, WithdrawnFrom::Waitlisted);
        // 대기자가 아닌 신청에는 연락 결과를 남기지 않는다
        let selected = before.selected[0].member.application_id;
        let wrong = f
            .contact(selected, ContactResult::Declined)
            .await
            .unwrap_err();
        assert!(
            matches!(wrong, ChangeSelectionError::WrongStatus),
            "{wrong}"
        );
    }

    #[tokio::test]
    async fn br_w8_corrects_waitlisted_back_to_same_number() {
        let f = fixture(true).await;
        let second = f.result().await.waitlisted[1].member.application_id;
        f.withdraw(second, None).await.unwrap();

        f.correct(second).await.unwrap();

        let after = f.result().await;
        assert_eq!(after.waitlisted[1].member.application_id, second);
        assert_eq!(after.waitlisted[1].waitlist_number, 2);
        let records = f.records().await;
        assert_eq!(records[1].1, "corrected");
        assert_eq!(records[1].3, "waitlisted");
        assert_eq!(records[1].4.as_deref(), Some("다른 회원을 잘못 누름"));
    }

    #[tokio::test]
    async fn br_w8_selected_comes_back_only_while_seat_is_open() {
        let f = fixture(true).await;
        let before = f.result().await;
        let leaving = before.selected[0].member.application_id;
        let other = before.selected[1].member.application_id;
        f.withdraw(leaving, None).await.unwrap();
        f.withdraw(other, None).await.unwrap();
        // 빈자리 2개 중 하나를 대기 1번이 채운다
        f.contact(
            before.waitlisted[0].member.application_id,
            ContactResult::Accepted,
        )
        .await
        .unwrap();

        f.correct(leaving).await.unwrap();
        let full = f.correct(other).await.unwrap_err();
        let not_withdrawn = f.correct(leaving).await.unwrap_err();

        assert!(matches!(full, ChangeSelectionError::NoOpenSeat), "{full}");
        assert!(
            matches!(not_withdrawn, ChangeSelectionError::WrongStatus),
            "{not_withdrawn}"
        );
        assert_eq!(f.result().await.open_seats, 0);
    }
}
