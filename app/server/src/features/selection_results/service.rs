//! 선발 결과 읽기 중 트랜잭션이 필요한 것. 포기·연락 기록은 `service/changes.rs`.

pub(crate) mod changes;

use sqlx::SqlitePool;

use super::repository::{self, ClosedCourse, CourseResultError};
use crate::infra::db::decode_error;

/// 학기에서 선발을 마감한 강좌들의 인원. 강좌 id 순서.
///
/// 강좌마다 선발 결과를 읽어 센다. 다음 추가 신청의 대기 순번은 추첨 기록에서 계산해야 해서다.
/// 강좌들을 같은 때의 모습으로 읽도록 트랜잭션 하나에서 읽는다.
pub async fn closed_courses(
    pool: &SqlitePool,
    term_id: i64,
) -> Result<Vec<ClosedCourse>, sqlx::Error> {
    let mut conn = pool.begin().await?;
    let course_ids = repository::closed_course_ids(&mut conn, term_id).await?;
    let mut courses = Vec::with_capacity(course_ids.len());
    for course_id in course_ids {
        let result = repository::course_result_in(&mut conn, course_id)
            .await
            .map_err(|error| match error {
                CourseResultError::Db(error) => error,
                // 같은 연결에서 방금 마감한 강좌로 읽었다
                CourseResultError::CourseNotFound | CourseResultError::NotClosed => {
                    decode_error("마감한 강좌의 선발 결과가 없습니다")
                }
            })?;
        courses.push(ClosedCourse {
            course_id,
            capacity: result.capacity,
            selected: result.selected.len(),
            waitlisted: result.waitlisted.len(),
            open_seats: result.open_seats,
            next_additional: repository::next_additional(&result),
        });
    }
    Ok(courses)
}
