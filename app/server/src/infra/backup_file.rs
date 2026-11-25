//! 백업 파일 층(ADR 0011, 0023): 백업 파일의 이름, 만들기, 보관 개수 정리.
//!
//! DB 구조를 모르고 파일만 다룬다. 구조 변경 직전 백업(`db`)과 복원·두 번째 위치(`backup`)가
//! 모두 이것을 부르므로, 여기서는 `db`와 `backup`을 부르지 않는다.

pub mod name;
pub mod retention;
pub mod store;

use std::path::{Path, PathBuf};

/// 데이터 폴더 안의 백업 폴더 이름(첫 번째 위치).
const DIR_NAME: &str = "backups";

/// 데이터 폴더 `data_dir` 안의 백업 폴더.
pub fn dir(data_dir: &Path) -> PathBuf {
    data_dir.join(DIR_NAME)
}
