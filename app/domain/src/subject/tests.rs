use super::*;

#[test]
fn name_trims_spaces() {
    assert_eq!(SubjectName::parse(" 요가 ").unwrap().as_str(), "요가");
}

#[test]
fn name_rejects_empty_and_too_long() {
    assert_eq!(SubjectName::parse("  "), Err(SubjectError::EmptyName));
    assert!(SubjectName::parse(&"가".repeat(30)).is_ok());
    assert_eq!(
        SubjectName::parse(&"가".repeat(31)),
        Err(SubjectError::NameTooLong)
    );
}

#[test]
fn name_keeps_inner_spaces_as_typed() {
    assert_eq!(
        SubjectName::parse("한글 교실").unwrap().as_str(),
        "한글 교실"
    );
}

#[test]
fn br_j1_key_ignores_every_space() {
    let key = |name: &str| SubjectName::parse(name).unwrap().key();

    assert_eq!(key("한글교실"), key("한글 교실"));
    assert_eq!(key("스마트폰 초급 반"), "스마트폰초급반");
}

#[test]
fn br_j1_key_tells_different_names_apart() {
    let key = |name: &str| SubjectName::parse(name).unwrap().key();

    assert_ne!(key("스마트폰"), key("스마트폰 활용"));
}

#[test]
fn category_is_optional() {
    assert_eq!(CategoryName::parse(""), Ok(None));
    assert_eq!(CategoryName::parse("   "), Ok(None));
}

#[test]
fn category_trims_and_limits_length() {
    assert_eq!(
        CategoryName::parse(" 평생교육 ").unwrap().unwrap().as_str(),
        "평생교육"
    );
    assert!(CategoryName::parse(&"가".repeat(20)).unwrap().is_some());
    assert_eq!(
        CategoryName::parse(&"가".repeat(21)),
        Err(SubjectError::CategoryTooLong)
    );
}

#[test]
fn parse_keeps_category() {
    let subject = NewSubject::parse(SubjectInput {
        name: "요가",
        category: Some("취미여가"),
    })
    .unwrap();

    assert_eq!(subject.name().as_str(), "요가");
    assert_eq!(
        subject.category().map(CategoryName::as_str),
        Some("취미여가")
    );
}

#[test]
fn missing_or_blank_category_is_none() {
    for category in [None, Some(""), Some("  ")] {
        let subject = NewSubject::parse(SubjectInput {
            name: "요가",
            category,
        })
        .unwrap();

        assert_eq!(subject.category(), None, "{category:?}");
    }
}

#[test]
fn collects_every_error_in_field_order() {
    let long = "가".repeat(21);
    let errors = NewSubject::parse(SubjectInput {
        name: "",
        category: Some(&long),
    })
    .unwrap_err();

    assert_eq!(
        errors,
        [SubjectError::EmptyName, SubjectError::CategoryTooLong]
    );
}
