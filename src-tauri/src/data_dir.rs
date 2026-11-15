//! 데이터(DB, 백업, 로그)를 두는 폴더.
//!
//! ADR 0012: exe 옆에 `data` 폴더가 있으면 포터블판, 없으면 설치판이다.

use std::path::{Path, PathBuf};

/// 포터블판을 알아보는 폴더 이름.
const PORTABLE_DIR: &str = "data";

/// 이 앱이 데이터를 두는 곳.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataDir {
    /// exe 옆 `data` 폴더.
    Portable(PathBuf),
    /// 운영체제의 앱 데이터 폴더. Windows는 `%APPDATA%\<앱 식별자>`.
    Installed(PathBuf),
}

impl DataDir {
    /// `exe_dir`에 `data` 폴더가 있으면 포터블판, 없으면 `app_data_dir`을 쓴다.
    pub fn resolve(exe_dir: &Path, app_data_dir: PathBuf) -> DataDir {
        let portable = exe_dir.join(PORTABLE_DIR);
        if portable.is_dir() {
            DataDir::Portable(portable)
        } else {
            DataDir::Installed(app_data_dir)
        }
    }

    pub fn path(&self) -> &Path {
        match self {
            DataDir::Portable(path) | DataDir::Installed(path) => path,
        }
    }

    pub fn is_portable(&self) -> bool {
        matches!(self, DataDir::Portable(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portable_when_data_folder_is_next_to_exe() {
        let exe_dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(exe_dir.path().join("data")).unwrap();

        let dir = DataDir::resolve(exe_dir.path(), PathBuf::from("app-data"));

        assert_eq!(dir, DataDir::Portable(exe_dir.path().join("data")));
    }

    #[test]
    fn installed_without_data_folder() {
        let exe_dir = tempfile::tempdir().unwrap();

        let dir = DataDir::resolve(exe_dir.path(), PathBuf::from("app-data"));

        assert_eq!(dir, DataDir::Installed(PathBuf::from("app-data")));
    }

    #[test]
    fn installed_when_data_is_a_file() {
        let exe_dir = tempfile::tempdir().unwrap();
        std::fs::write(exe_dir.path().join("data"), "").unwrap();

        let dir = DataDir::resolve(exe_dir.path(), PathBuf::from("app-data"));

        assert!(!dir.is_portable());
    }
}
