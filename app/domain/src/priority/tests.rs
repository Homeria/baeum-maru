use super::*;

#[test]
fn name_trims_spaces() {
    let name = PriorityReasonName::parse("  반장 ").unwrap();
    assert_eq!(name.as_str(), "반장");
}

#[test]
fn empty_name_is_refused() {
    assert_eq!(
        PriorityReasonName::parse("   "),
        Err(PriorityError::EmptyName)
    );
}

#[test]
fn name_up_to_30_chars() {
    assert!(PriorityReasonName::parse(&"가".repeat(30)).is_ok());
    assert_eq!(
        PriorityReasonName::parse(&"가".repeat(31)),
        Err(PriorityError::NameTooLong)
    );
}

#[test]
fn br_s10_names_differing_only_in_spaces_share_key() {
    let a = PriorityReasonName::parse("지난 학기 반장").unwrap();
    let b = PriorityReasonName::parse("지난학기반장").unwrap();
    assert_eq!(a.key(), b.key());
}

#[test]
fn method_code_round_trips() {
    for method in PriorityMethod::ALL {
        assert_eq!(PriorityMethod::from_code(method.code()), Some(method));
    }
    assert_eq!(PriorityMethod::from_code("quota"), None);
}
