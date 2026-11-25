use super::*;

const TEXT: &str = "00112233445566778899aabbccddeeff";

fn identity(byte: u8, generation: u32) -> HostIdentity {
    HostIdentity {
        bundle_id: DataBundleId::from_bytes([byte; 16]),
        generation,
    }
}

#[test]
fn bundle_id_reads_and_writes_lowercase_hex() {
    let id = DataBundleId::parse(TEXT).unwrap();

    assert_eq!(id.to_string(), TEXT);
}

#[test]
fn bundle_id_refuses_other_text() {
    for text in [
        "",
        "0011",
        &TEXT.to_uppercase(),
        &format!("{}zz", &TEXT[..30]),
    ] {
        assert_eq!(DataBundleId::parse(text), Err(DataBundleIdError), "{text}");
    }
}

#[test]
fn br_h5_first_host_is_remembered() {
    assert_eq!(check_host(None, identity(1, 3)), HostCheck::First);
}

#[test]
fn br_h5_same_or_newer_generation_connects() {
    assert_eq!(
        check_host(Some(identity(1, 2)), identity(1, 2)),
        HostCheck::Same
    );
    assert_eq!(
        check_host(Some(identity(1, 2)), identity(1, 3)),
        HostCheck::Newer
    );
    assert!(HostCheck::Newer.can_connect());
}

#[test]
fn br_h5_older_generation_or_other_bundle_is_refused() {
    let older = check_host(Some(identity(1, 3)), identity(1, 2));
    let other = check_host(Some(identity(1, 3)), identity(2, 9));

    assert_eq!(older, HostCheck::Older);
    assert_eq!(other, HostCheck::OtherBundle);
    assert!(!older.can_connect());
    assert!(!other.can_connect());
}
