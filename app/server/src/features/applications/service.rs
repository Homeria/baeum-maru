//! 신청 저장(규칙 확인과 규칙 예외, 추가 신청)과 지우기(삭제 기록). 트랜잭션을 열고 규칙
//! 확인(`baeum-domain`)과 repository를 엮는다. 우선 배정 사유 바꾸기는 `service/priorities.rs`.

pub(crate) mod priorities;

use std::collections::BTreeSet;

use baeum_domain::{AdditionalOutcome, ApplicationRule, CourseForRules, check_application};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::error::{DeleteApplicationError, SubmitError};
use super::repository;
use super::repository::priorities as priority_sql;
use crate::infra::clock;
use crate::infra::db::timestamp_text;

/// 회원 한 명이 한 학기 강좌 여러 개에 내는 신청.
#[derive(Debug, Clone, Copy)]
pub struct NewApplications<'a> {
    pub term_id: i64,
    pub member_id: i64,
    /// 고른 순서. 이번에 고른 강좌끼리 부딪히면 뒤 강좌에 붙인다.
    pub course_ids: &'a [i64],
    /// 규칙 예외 사유(BR-A6). 없으면 규칙을 어길 때 저장하지 않는다(BR-A5).
    pub override_reason: Option<&'a str>,
    /// 접수한 직원.
    pub staff_id: i64,
    /// 강좌마다 붙일 우선 배정 사유(BR-S12). 없는 강좌는 빼도 된다.
    pub priorities: &'a [CoursePriorities<'a>],
}

/// 고른 강좌 하나에 붙일 우선 배정 사유들.
#[derive(Debug, Clone, Copy)]
pub struct CoursePriorities<'a> {
    /// [`NewApplications::course_ids`]에 있는 강좌.
    pub course_id: i64,
    pub reason_ids: &'a [i64],
}

/// 저장한 신청 하나.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Submitted {
    pub id: i64,
    pub course_id: i64,
    /// 추가 신청(BR-X2)이면 그 결과. 보통 신청(접수됨)이면 `None`.
    pub additional: Option<AdditionalOutcome>,
}

/// 신청을 저장하고 고른 순서대로 돌려준다.
///
/// 선발 마감한 강좌는 추가 신청으로 받아 바로 당첨이나 대기로 정한다(BR-X1, BR-X2). 같은
/// 트랜잭션에서 세므로 여러 PC가 동시에 받아도 늦게 저장한 쪽이 뒤에 선다.
///
/// 규칙을 어기면 사유가 있고 모두 예외를 줄 수 있는 규칙일 때만 저장하고, 어긴 규칙마다 규칙 예외를 남긴다.
/// 이미 낸 신청을 읽고 확인한 뒤 저장하므로 `BEGIN IMMEDIATE`로 한다 (ADR 0005).
pub async fn submit(
    pool: &SqlitePool,
    new: NewApplications<'_>,
    now: OffsetDateTime,
) -> Result<Vec<Submitted>, SubmitError> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    if !repository::member_exists(&mut tx, new.member_id).await? {
        return Err(SubmitError::MemberNotFound);
    }

    let courses = repository::term_courses(&mut tx, new.term_id).await?;
    let missing: Vec<i64> = new
        .course_ids
        .iter()
        .copied()
        .filter(|id| !courses.contains_key(id))
        .collect();
    if !missing.is_empty() {
        return Err(SubmitError::CourseNotFound(missing));
    }
    let drawn_ids = repository::drawn_course_ids(&mut tx, new.term_id).await?;
    let closed_ids = repository::closed_course_ids(&mut tx, new.term_id).await?;
    let (closed, drawn): (Vec<i64>, Vec<i64>) = new
        .course_ids
        .iter()
        .copied()
        .filter(|id| drawn_ids.contains(id))
        .partition(|id| closed_ids.contains(id));
    if !drawn.is_empty() {
        return Err(SubmitError::CourseDrawn(drawn));
    }
    let for_additional: Vec<i64> = new
        .priorities
        .iter()
        .filter(|priority| !priority.reason_ids.is_empty())
        .map(|priority| priority.course_id)
        .filter(|id| closed.contains(id))
        .collect();
    if !for_additional.is_empty() {
        return Err(SubmitError::PriorityForAdditional(for_additional));
    }
    if !closed.is_empty() {
        let ends_on = repository::term_ends_on(&mut tx, new.term_id).await?;
        if clock::local_date(now) > ends_on {
            return Err(SubmitError::TermEnded(closed));
        }
    }
    let unchosen: Vec<i64> = new
        .priorities
        .iter()
        .map(|priority| priority.course_id)
        .filter(|id| !new.course_ids.contains(id))
        .collect();
    if !unchosen.is_empty() {
        return Err(SubmitError::PriorityForUnchosenCourse(unchosen));
    }
    let reason_ids: BTreeSet<i64> = new
        .priorities
        .iter()
        .flat_map(|priority| priority.reason_ids.iter().copied())
        .collect();
    let missing = priority_sql::missing_reasons(&mut tx, &reason_ids).await?;
    if !missing.is_empty() {
        return Err(SubmitError::PriorityReasonNotFound(missing));
    }

    let applied_ids = repository::applied_course_ids(&mut tx, new.term_id, new.member_id).await?;
    // 고른 강좌는 위에서, 신청한 강좌는 같은 학기 강좌라서 `courses`에 반드시 있다
    let for_rules = |id: &i64| courses[id].for_rules(*id);
    let applied: Vec<CourseForRules<'_>> = applied_ids.iter().map(for_rules).collect();
    let chosen: Vec<CourseForRules<'_>> = new.course_ids.iter().map(for_rules).collect();
    let violations = check_application(&applied, &chosen);

    let reason = match new.override_reason {
        _ if violations.is_empty() => None,
        Some(reason) if violations.iter().all(|v| v.rule.can_override()) => Some(reason),
        _ => return Err(SubmitError::RulesBroken(violations)),
    };

    let now = timestamp_text(now);
    let mut submitted = Vec::with_capacity(new.course_ids.len());
    for &course_id in new.course_ids {
        let additional = if closed_ids.contains(&course_id) {
            Some(repository::additional_outcome_for(&mut tx, course_id).await?)
        } else {
            None
        };
        let id = repository::insert_application(
            &mut tx,
            new.member_id,
            course_id,
            additional,
            new.staff_id,
            &now,
        )
        .await?;
        if let Some(outcome) = additional {
            repository::insert_additional(&mut tx, id, outcome).await?;
        }
        if let Some(reason) = reason {
            let mut rules: Vec<ApplicationRule> = violations
                .iter()
                .filter(|v| v.course_id == course_id)
                .map(|v| v.rule)
                .collect();
            rules.sort();
            rules.dedup();
            for rule in rules {
                repository::insert_rule_override(&mut tx, id, rule, reason, new.staff_id, &now)
                    .await?;
            }
        }
        let target = priority_sql::ChangeTarget {
            application_id: id,
            member_id: new.member_id,
            course_id,
            staff_id: new.staff_id,
            now: &now,
        };
        // 같은 사유가 두 번 와도 한 번만 붙인다
        let reasons: BTreeSet<i64> = new
            .priorities
            .iter()
            .filter(|priority| priority.course_id == course_id)
            .flat_map(|priority| priority.reason_ids.iter().copied())
            .collect();
        for reason_id in reasons {
            priority_sql::attach(&mut tx, &target, reason_id).await?;
        }
        submitted.push(Submitted {
            id,
            course_id,
            additional,
        });
    }

    tx.commit().await?;
    Ok(submitted)
}

/// 접수됨 신청을 지우고 삭제 기록을 남긴다(BR-D1~D3). 규칙 예외도 함께 지워진다.
///
/// 상태를 읽고 지우므로 `BEGIN IMMEDIATE`로 한다.
pub async fn delete(
    pool: &SqlitePool,
    id: i64,
    staff_id: i64,
    reason: Option<&str>,
    now: OffsetDateTime,
) -> Result<(), DeleteApplicationError> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let row = repository::deletion_target(&mut tx, id).await?;
    let Some((member_id, course_id, status)) = row else {
        return Err(DeleteApplicationError::NotFound);
    };
    if status != "submitted" {
        return Err(DeleteApplicationError::NotSubmitted);
    }

    repository::delete_with_record(&mut tx, id, member_id, course_id, staff_id, reason, now)
        .await?;

    tx.commit().await?;
    Ok(())
}
