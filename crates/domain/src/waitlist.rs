//! 포기 뒤의 빈자리, 대기자 연락할 차례, 잘못 처리 바로잡기(BR-C3, BR-W1~W8).

use crate::Capacity;

/// 대기자에게 연락한 결과(BR-W2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContactResult {
    /// 수락. 빈자리가 있으면 당첨이 된다(BR-W3).
    Accepted,
    /// 거절. 포기가 된다(BR-W4).
    Declined,
    /// 연락 안 됨. 대기 순번을 그대로 두고 다음 순번이 차례가 된다(BR-W5).
    Unreachable,
}

impl ContactResult {
    /// 모든 결과. 화면 순서.
    pub const ALL: [ContactResult; 3] = [
        ContactResult::Accepted,
        ContactResult::Declined,
        ContactResult::Unreachable,
    ];

    /// DB와 JSON에 쓰는 이름. 예: `unreachable`
    pub fn code(self) -> &'static str {
        match self {
            ContactResult::Accepted => "accepted",
            ContactResult::Declined => "declined",
            ContactResult::Unreachable => "unreachable",
        }
    }

    /// [`ContactResult::code`]의 이름에서 결과를 찾는다. 모르는 이름이면 `None`.
    pub fn from_code(code: &str) -> Option<Self> {
        ContactResult::ALL
            .into_iter()
            .find(|result| result.code() == code)
    }
}

/// 포기하기 전 상태. 잘못 처리 바로잡기(BR-W8)가 이 상태로 되돌린다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WithdrawnFrom {
    /// 당첨이었다.
    Selected,
    /// 대기였다.
    Waitlisted,
}

impl WithdrawnFrom {
    /// 바로잡기로 이 상태로 돌아갈 수 있는지.
    ///
    /// BR-W8: 당첨이었으면 빈자리가 있어야 한다. 대기였으면 원래 순번으로 늘 돌아간다.
    pub fn can_restore(self, open_seats: usize) -> bool {
        match self {
            WithdrawnFrom::Selected => open_seats > 0,
            WithdrawnFrom::Waitlisted => true,
        }
    }
}

/// 빈자리 수. 정원에서 당첨 수를 뺀다(BR-W1). 당첨이 정원보다 많으면 0.
pub fn open_seats(capacity: Capacity, selected: usize) -> usize {
    usize::from(capacity.value()).saturating_sub(selected)
}

/// 대기자 한 명. 연락할 차례를 정할 때 쓴다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Waiting {
    /// 신청 id.
    pub application_id: i64,
    /// 마지막으로 남긴 연락 결과. 연락한 적이 없으면 `None`.
    pub last_contact: Option<ContactResult>,
}

/// 지금 연락할 차례인 대기자들의 신청 id. 대기 순번 순서다.
///
/// `waiting`은 대기 순번 순서여야 한다. 앞에서부터 빈자리 수만큼 고르고, 마지막 연락이
/// "연락 안 됨"인 대기자는 건너뛴다(BR-W1, BR-W5). 차례는 저장하지 않고 이렇게 그때그때 정한다.
pub fn contact_turns(waiting: &[Waiting], open_seats: usize) -> Vec<i64> {
    waiting
        .iter()
        .filter(|entry| entry.last_contact != Some(ContactResult::Unreachable))
        .take(open_seats)
        .map(|entry| entry.application_id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn waiting(id: i64, last_contact: Option<ContactResult>) -> Waiting {
        Waiting {
            application_id: id,
            last_contact,
        }
    }

    #[test]
    fn br_w1_open_seats_never_go_below_zero() {
        let capacity = Capacity::new(20).unwrap();

        assert_eq!(open_seats(capacity, 19), 1);
        assert_eq!(open_seats(capacity, 20), 0);
        assert_eq!(open_seats(capacity, 21), 0);
    }

    #[test]
    fn br_w1_turns_start_from_front_as_many_as_open_seats() {
        let list = [waiting(11, None), waiting(12, None), waiting(13, None)];

        assert_eq!(contact_turns(&list, 2), [11, 12]);
        assert_eq!(contact_turns(&list, 0), Vec::<i64>::new());
        assert_eq!(contact_turns(&list, 5), [11, 12, 13]);
    }

    #[test]
    fn br_w5_unreachable_is_skipped_but_keeps_place() {
        let list = [
            waiting(11, Some(ContactResult::Unreachable)),
            waiting(12, None),
            waiting(13, None),
        ];

        assert_eq!(contact_turns(&list, 1), [12]);
    }

    #[test]
    fn br_w8_selected_needs_open_seat_but_waitlisted_always_returns() {
        assert!(WithdrawnFrom::Selected.can_restore(1));
        assert!(!WithdrawnFrom::Selected.can_restore(0));
        assert!(WithdrawnFrom::Waitlisted.can_restore(0));
    }

    #[test]
    fn contact_result_code_round_trips() {
        for result in ContactResult::ALL {
            assert_eq!(ContactResult::from_code(result.code()), Some(result));
        }
        assert_eq!(ContactResult::from_code("maybe"), None);
    }
}
