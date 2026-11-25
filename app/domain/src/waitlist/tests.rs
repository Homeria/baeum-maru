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

#[test]
fn br_x2_open_seat_without_waiting_is_selected_at_once() {
    assert_eq!(
        additional_outcome(2, 0, Some(3)),
        AdditionalOutcome::Selected
    );
    assert_eq!(additional_outcome(1, 0, None), AdditionalOutcome::Selected);
}

#[test]
fn br_x2_waiting_members_go_first_even_with_open_seat() {
    assert_eq!(
        additional_outcome(1, 2, Some(5)),
        AdditionalOutcome::Waitlisted(6)
    );
}

#[test]
fn br_x2_full_course_waits_after_last_number_used() {
    // 대기자가 모두 빠졌어도 쓴 번호는 다시 쓰지 않는다 (BR-W6)
    assert_eq!(
        additional_outcome(0, 0, Some(3)),
        AdditionalOutcome::Waitlisted(4)
    );
    assert_eq!(
        additional_outcome(0, 0, None),
        AdditionalOutcome::Waitlisted(1)
    );
}
