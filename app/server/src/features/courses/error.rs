//! 강좌 기능의 오류. 같은 기능의 모든 층이 쓴다 (ADR 0023).

use crate::infra::db::is_unique_violation;

/// 강좌가 가리키는 것 중 DB에 없는 것. 그 사이 다른 직원이 지웠을 수 있다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum MissingReference {
    #[error("목록에 없는 과목입니다. 다시 골라 주세요")]
    Subject,
    /// `row`는 요청의 몇 번째 수업 시간인지(1부터)다.
    #[error("{row}번째 수업 시간: 목록에 없는 강의실입니다. 다시 골라 주세요")]
    Room { row: usize },
}

/// 강좌를 저장하지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum SaveCourseError {
    /// 강좌를 넣을 학기가 없다.
    #[error("학기를 찾지 못했습니다")]
    TermNotFound,
    /// 고른 과목이나 강의실이 없다. 없는 것을 모두 모았다.
    #[error("목록에 없는 과목이나 강의실을 골랐습니다")]
    MissingReferences(Vec<MissingReference>),
    /// BR-T3: 같은 학기 안에서 과목·수준·반이 모두 같은 강좌는 하나뿐이다.
    #[error("같은 학기에 같은 강좌가 있습니다")]
    Duplicate,
    #[error("강좌를 저장하지 못했습니다: {0}")]
    Db(sqlx::Error),
}

impl From<sqlx::Error> for SaveCourseError {
    fn from(error: sqlx::Error) -> Self {
        // UNIQUE는 courses_unique_in_term 하나뿐이다 (course_schedules에는 없다)
        if is_unique_violation(&error) {
            SaveCourseError::Duplicate
        } else {
            SaveCourseError::Db(error)
        }
    }
}
