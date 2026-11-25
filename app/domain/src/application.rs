//! 신청 규칙(BR-A1, BR-A2, BR-A4)과 규칙 예외(BR-A6, BR-A7) 판단.
//!
//! 최대 신청 개수(BR-A3)와 학기별 켜기·끄기는 아직 없다. A1, A2는 늘 켠다.

use crate::course::Schedule;

/// 신청을 받을 때 확인하는 규칙.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ApplicationRule {
    /// BR-A4: 같은 회원이 같은 강좌에 유효한 신청을 두 건 가질 수 없다.
    NoDuplicate,
    /// BR-A1: 수업 시간이 하나라도 겹치는 강좌는 함께 신청할 수 없다.
    NoTimeOverlap,
    /// BR-A2: 같은 학기에 같은 과목은 하나만 신청할 수 있다.
    NoSameSubject,
}

impl ApplicationRule {
    /// 모든 규칙.
    pub const ALL: [ApplicationRule; 3] = [
        ApplicationRule::NoDuplicate,
        ApplicationRule::NoTimeOverlap,
        ApplicationRule::NoSameSubject,
    ];

    /// DB와 API에서 쓰는 이름.
    pub fn code(self) -> &'static str {
        match self {
            ApplicationRule::NoDuplicate => "no_duplicate",
            ApplicationRule::NoTimeOverlap => "no_time_overlap",
            ApplicationRule::NoSameSubject => "no_same_subject",
        }
    }

    /// [`ApplicationRule::code`]의 이름에서 규칙을 찾는다. 모르는 이름이면 `None`.
    pub fn from_code(code: &str) -> Option<Self> {
        ApplicationRule::ALL
            .into_iter()
            .find(|rule| rule.code() == code)
    }

    /// 직원이 사유를 적으면 이 규칙을 어긴 신청도 받을 수 있는지(BR-A6).
    /// BR-A4는 예외를 줄 수 없다(BR-A7).
    pub fn can_override(self) -> bool {
        !matches!(self, ApplicationRule::NoDuplicate)
    }
}

/// 사유 칸이 형식에 맞지 않는다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ReasonError {
    /// 사유가 너무 길다.
    #[error("사유는 100자까지 씁니다")]
    TooLong,
}

/// 규칙 예외(BR-A6)나 신청 삭제(BR-D3)의 사유. 1~100자이고 앞뒤 공백은 지운다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reason(String);

impl Reason {
    const MAX_LEN: usize = 100;

    /// 사유를 확인한다. 비었거나 공백뿐이면 `Ok(None)`.
    ///
    /// # Errors
    ///
    /// 100자를 넘으면 [`ReasonError::TooLong`].
    pub fn parse(input: &str) -> Result<Option<Self>, ReasonError> {
        let trimmed = input.trim();
        match trimmed.chars().count() {
            0 => Ok(None),
            n if n > Self::MAX_LEN => Err(ReasonError::TooLong),
            _ => Ok(Some(Reason(trimmed.to_owned()))),
        }
    }

    /// 사유 글자.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 규칙을 판단하는 데 필요한 강좌 정보.
#[derive(Debug, Clone, Copy)]
pub struct CourseForRules<'a> {
    /// 강좌 DB id.
    pub course_id: i64,
    /// 과목 DB id.
    pub subject_id: i64,
    /// 수업 시간. 주 2회 강좌는 두 줄이다.
    pub schedules: &'a [Schedule],
}

/// 신청하려는 강좌가 어긴 규칙 하나.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuleViolation {
    /// 신청하려는 강좌.
    pub course_id: i64,
    /// 어긴 규칙.
    pub rule: ApplicationRule,
    /// 부딪힌 강좌. 이미 신청한 강좌이거나 이번에 함께 고른 강좌다.
    pub other_course_id: i64,
}

/// 회원이 이미 유효한 신청을 낸 강좌 `applied`에 더해 `chosen` 강좌들을 신청할 때
/// 어기는 규칙을 모두 찾는다. 하나도 없으면 빈 목록이다.
///
/// - 고른 강좌마다 이미 신청한 강좌, 그다음 앞서 고른 강좌와 비교한다. 이번에 고른
///   강좌끼리 부딪히면 뒤에 고른 강좌에 붙인다.
/// - 같은 강좌면 BR-A4만 알린다. 같은 강좌는 시간도 과목도 같아서 다른 규칙은 덧붙이지 않는다.
///
/// # Examples
///
/// ```
/// use baeum_domain::{ApplicationRule, ClassTime, CourseForRules, Schedule, check_application};
/// use time::Weekday;
///
/// let time = |text| ClassTime::parse(text).unwrap();
/// let monday = [Schedule::new(Weekday::Monday, time("10:00"), time("11:00"), None).unwrap()];
/// let yoga = CourseForRules { course_id: 1, subject_id: 10, schedules: &monday };
/// let hangul = CourseForRules { course_id: 2, subject_id: 20, schedules: &monday };
///
/// let violations = check_application(&[yoga], &[hangul]);
///
/// assert_eq!(violations.len(), 1);
/// assert_eq!(violations[0].rule, ApplicationRule::NoTimeOverlap);
/// assert_eq!(violations[0].other_course_id, 1);
/// ```
pub fn check_application(
    applied: &[CourseForRules<'_>],
    chosen: &[CourseForRules<'_>],
) -> Vec<RuleViolation> {
    let mut violations = Vec::new();
    for (index, course) in chosen.iter().enumerate() {
        for other in applied.iter().chain(&chosen[..index]) {
            for rule in broken_rules(course, other) {
                violations.push(RuleViolation {
                    course_id: course.course_id,
                    rule,
                    other_course_id: other.course_id,
                });
            }
        }
    }
    violations
}

/// 두 강좌를 함께 신청할 때 어기는 규칙.
fn broken_rules(course: &CourseForRules<'_>, other: &CourseForRules<'_>) -> Vec<ApplicationRule> {
    if course.course_id == other.course_id {
        return vec![ApplicationRule::NoDuplicate];
    }
    let mut rules = Vec::new();
    let overlaps = course
        .schedules
        .iter()
        .any(|schedule| other.schedules.iter().any(|o| schedule.overlaps(*o)));
    if overlaps {
        rules.push(ApplicationRule::NoTimeOverlap);
    }
    if course.subject_id == other.subject_id {
        rules.push(ApplicationRule::NoSameSubject);
    }
    rules
}

#[cfg(test)]
mod tests;
