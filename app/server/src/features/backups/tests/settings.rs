//! 백업 설정 파일 테스트.

use crate::features::backups::service::settings::*;

#[tokio::test]
async fn missing_or_broken_file_means_off() {
    let dir = tempfile::tempdir().unwrap();
    let path = file(dir.path());
    assert_eq!(load(&path).await, BackupSettings::default());

    tokio::fs::write(&path, b"{broken").await.unwrap();

    assert_eq!(load(&path).await.second_location, None);
}

#[tokio::test]
async fn saved_settings_read_back() {
    let dir = tempfile::tempdir().unwrap();
    let path = file(dir.path());
    let settings = BackupSettings {
        second_location: Some(dir.path().join("외장 디스크")),
    };

    save(&path, &settings).await.unwrap();

    assert_eq!(load(&path).await, settings);
}
