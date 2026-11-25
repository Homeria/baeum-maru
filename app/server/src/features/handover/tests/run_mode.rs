//! 실행 모드 파일 테스트.

use crate::infra::db;

use crate::features::handover::service::run_mode::*;

#[test]
fn br_h1_handover_states_are_not_chosen_by_hand() {
    assert!(!RunMode::HandingOver.is_choosable());
    assert!(!RunMode::HandedOver.is_choosable());
    assert!(!can_change(RunMode::Host, RunMode::HandedOver));
    assert!(!can_change(RunMode::Remote, RunMode::HandingOver));
}

#[test]
fn br_h2_handed_over_pc_goes_back_to_host_only_by_cancel() {
    assert!(can_change(RunMode::HandedOver, RunMode::Remote));
    assert!(!can_change(RunMode::HandedOver, RunMode::Host));
    assert!(!can_change(RunMode::HandingOver, RunMode::Host));
}

#[test]
fn remembers_handed_over() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(db::file(dir.path()), "").unwrap();

    save(dir.path(), RunMode::HandedOver).unwrap();

    assert_eq!(resolve(dir.path()), Some(RunMode::HandedOver));
}

#[test]
fn undecided_on_fresh_pc() {
    let dir = tempfile::tempdir().unwrap();

    assert_eq!(resolve(dir.path()), None);
}

#[test]
fn pc_with_db_is_host_without_asking() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(db::file(dir.path()), "").unwrap();

    assert_eq!(resolve(dir.path()), Some(RunMode::Host));
}

#[test]
fn remembers_chosen_mode() {
    let dir = tempfile::tempdir().unwrap();

    save(dir.path(), RunMode::Remote).unwrap();

    assert_eq!(resolve(dir.path()), Some(RunMode::Remote));
}

#[test]
fn remote_stays_remote_with_leftover_db() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(db::file(dir.path()), "").unwrap();

    save(dir.path(), RunMode::Remote).unwrap();

    assert_eq!(resolve(dir.path()), Some(RunMode::Remote));
}

#[test]
fn creates_missing_data_folder() {
    let dir = tempfile::tempdir().unwrap();
    let data_dir = dir.path().join("새 폴더");

    save(&data_dir, RunMode::Host).unwrap();

    assert_eq!(resolve(&data_dir), Some(RunMode::Host));
}

#[test]
fn unreadable_file_falls_back_to_db() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(FILE), "망가진 내용").unwrap();

    assert_eq!(resolve(dir.path()), None);

    std::fs::write(db::file(dir.path()), "").unwrap();
    assert_eq!(resolve(dir.path()), Some(RunMode::Host));
}
