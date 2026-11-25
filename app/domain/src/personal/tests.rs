use super::*;

#[derive(Debug)]
struct Holder {
    id: i64,
    name: Personal<String>,
}

#[test]
fn debug_hides_value_inside_other_structs() {
    let holder = Holder {
        id: 7,
        name: Personal::new(String::from("회원1")),
    };

    let printed = format!("{holder:?}");

    assert!(printed.contains("id: 7"));
    assert!(printed.contains("name: ***"));
    assert!(!printed.contains("회원1"));
    assert_eq!(holder.id, 7);
    assert_eq!(holder.name.expose(), "회원1");
}

#[test]
fn pretty_debug_also_hides_value() {
    let name = Personal::new(String::from("회원1"));

    assert_eq!(format!("{name:#?}"), "***");
}

#[test]
fn into_inner_returns_value() {
    let phone = Personal::new(String::from("01000000001"));

    assert_eq!(phone.into_inner(), "01000000001");
}
