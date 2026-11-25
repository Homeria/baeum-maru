//! 실행 모드(ADR 0021): 이 PC의 앱이 호스트로 켜는지, 다른 PC의 호스트에 접속하는지.
//!
//! 처음 켤 때 한 번 고르고 데이터 폴더에 기억한다. 이미 DB가 있는 PC는 묻지 않고 호스트로 켠다.

use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::infra::db;

/// 실행 모드를 기억하는 파일. 데이터 폴더 안에 둔다.
const FILE: &str = "run-mode.json";

/// 이 PC의 앱이 켜는 방식.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum RunMode {
    /// 이 PC가 DB를 갖고 서버를 띄운다.
    Host,
    /// 다른 PC의 호스트에 접속한다. 이 PC의 DB는 열지 않는다.
    Remote,
    /// 호스트를 넘기는 중이다(BR-H1). 서버 없이 DB만 열어 넘기기 파일을 만든다.
    HandingOver,
    /// 호스트를 넘겼다(BR-H1). DB를 열지 않고, 넘기기 취소나 접속 모드로 바꾸기만 한다.
    HandedOver,
}

impl RunMode {
    /// 직원이 처음 고르기와 모드 바꾸기로 고를 수 있는 모드. 넘기기 상태는 넘기기 기능으로만 들어간다.
    pub fn is_choosable(self) -> bool {
        matches!(self, RunMode::Host | RunMode::Remote)
    }
}

/// 파일에 적는 모양. 나중에 칸이 늘어도 읽을 수 있게 객체로 둔다.
#[derive(Debug, Serialize, Deserialize)]
struct Stored {
    mode: RunMode,
}

/// 기억한 실행 모드를 읽는다. 아직 고르지 않았으면 `None`.
///
/// 파일이 없어도 DB가 있으면 호스트다. 이 기능 전부터 쓰던 PC다.
/// 파일을 읽지 못하면 없는 것처럼 다룬다. 접속 모드 PC라도 DB가 없으면 다시 묻기만 한다.
pub fn resolve(data_dir: &Path) -> Option<RunMode> {
    match std::fs::read(data_dir.join(FILE)) {
        Ok(bytes) => match serde_json::from_slice::<Stored>(&bytes) {
            Ok(stored) => return Some(stored.mode),
            Err(error) => tracing::warn!(%error, "실행 모드 파일을 읽지 못했습니다"),
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => tracing::warn!(%error, "실행 모드 파일을 열지 못했습니다"),
    }
    db::file(data_dir).exists().then_some(RunMode::Host)
}

/// 실행 모드를 기억한다. 데이터 폴더가 없으면 만든다.
pub fn save(data_dir: &Path, mode: RunMode) -> io::Result<()> {
    std::fs::create_dir_all(data_dir)?;
    let text = serde_json::to_vec(&Stored { mode }).map_err(io::Error::other)?;
    std::fs::write(data_dir.join(FILE), text)
}

/// 켤 때 정한 실행 모드. 앱에 맡겨 두고 명령이 꺼내 쓴다.
#[derive(Debug, Clone)]
pub struct RunModeState {
    data_dir: PathBuf,
    mode: Option<RunMode>,
}

impl RunModeState {
    pub fn new(data_dir: &Path, mode: Option<RunMode>) -> Self {
        RunModeState {
            data_dir: data_dir.to_owned(),
            mode,
        }
    }

    /// 켤 때 정한 실행 모드. 아직 고르지 않았으면 `None`.
    pub fn mode(&self) -> Option<RunMode> {
        self.mode
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// 접속 모드인데 이 PC에 예전 DB가 남아 있다. 지우지 않고 쓰지도 않는다.
    pub fn has_leftover_data(&self) -> bool {
        self.mode == Some(RunMode::Remote) && db::file(&self.data_dir).exists()
    }
}

/// `from`에서 `to`로 실행 모드 바꾸기로 바꿀 수 있는지.
///
/// 넘긴 PC를 다시 호스트로 쓰려면 PIN을 넣는 넘기기 취소(BR-H2)로만 한다. 넘기는 중인 PC는
/// 그 화면에서 넘기지 않고 돌아가기로만 나온다.
pub fn can_change(from: RunMode, to: RunMode) -> bool {
    matches!(
        (from, to),
        (RunMode::Host, RunMode::Remote)
            | (RunMode::Remote, RunMode::Host)
            | (RunMode::HandedOver, RunMode::Remote)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
