//! 이 PC의 지역 날짜 (ADR 0015).
//!
//! 유효기간과 생년월일은 기관의 달력 날짜라 UTC가 아니라 이 PC의 날짜로 본다.

use time::{Date, OffsetDateTime, UtcOffset};

/// `now`가 이 PC의 지역 시간으로 며칠인지.
///
/// 운영체제에서 지역 시간대를 얻지 못하면 경고를 남기고 UTC 날짜로 본다.
pub fn local_date(now: OffsetDateTime) -> Date {
    match UtcOffset::local_offset_at(now) {
        Ok(offset) => now.to_offset(offset).date(),
        Err(_) => {
            tracing::warn!("지역 시간대를 얻지 못해 UTC 날짜로 봄");
            now.date()
        }
    }
}

/// 지금 이 PC의 지역 날짜.
pub fn local_today() -> Date {
    local_date(OffsetDateTime::now_utc())
}

#[cfg(test)]
mod tests {
    use time::macros::datetime;

    use super::*;

    #[test]
    fn local_date_is_the_date_in_local_offset() {
        let now = datetime!(2026-11-15 23:30 UTC);
        let offset = UtcOffset::local_offset_at(now).unwrap_or(UtcOffset::UTC);

        assert_eq!(local_date(now), now.to_offset(offset).date());
    }
}
