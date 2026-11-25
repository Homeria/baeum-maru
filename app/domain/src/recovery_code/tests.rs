use super::*;

#[test]
fn uses_every_letter_and_no_confusing_ones() {
    let bytes: [u8; 16] = std::array::from_fn(|i| u8::try_from(i * 2).unwrap());
    let code = RecoveryCode::from_random(bytes).to_string();

    assert_eq!(code, "2468-ACEG-JLNQ-SUWY");
    for confusing in ['0', '1', 'O', 'I'] {
        assert!(!ALPHABET.contains(&(confusing as u8)), "{confusing}");
    }
}

#[test]
fn high_bits_are_ignored() {
    assert_eq!(
        RecoveryCode::from_random([0b1110_0000; 16]),
        RecoveryCode::from_random([0; 16])
    );
}

#[test]
fn parse_accepts_lowercase_and_spaces() {
    let code = RecoveryCode::from_random([7; 16]);
    let typed = code.to_string().to_lowercase().replace('-', " ");

    assert_eq!(RecoveryCode::parse(&typed), Some(code));
}

#[test]
fn parse_rejects_wrong_shape() {
    for bad in [
        "",
        "2222-2222-2222",
        "2222-2222-2222-2220",
        "2222-2222-2222-22222",
    ] {
        assert_eq!(RecoveryCode::parse(bad), None, "{bad}");
    }
}

#[test]
fn debug_hides_code() {
    let code = RecoveryCode::from_random([0; 16]);
    assert_eq!(format!("{code:?}"), "RecoveryCode(***)");
}
