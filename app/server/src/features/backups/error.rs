//! 백업 기능의 오류. 같은 기능의 모든 층과 넘기기가 쓴다 (ADR 0023).

use crate::infra::backup_file::store::BackupError;
use crate::infra::db::DbError;

/// 복원할 수 없는 파일이다. 경로는 담지 않는다.
#[derive(Debug, thiserror::Error)]
pub enum InspectError {
    #[error("백업 파일을 열지 못했습니다. 파일이 깨졌거나 DB 파일이 아닙니다")]
    Broken(#[source] sqlx::Error),
    #[error("백업 파일이 깨졌습니다. 다른 백업을 골라 주세요")]
    Damaged,
    #[error("배움마루 백업 파일이 아닙니다")]
    NotBaeumMaru,
    #[error("더 새 버전의 배움마루에서 만든 백업입니다. 앱을 새 버전으로 바꾼 뒤 복원해 주세요")]
    NewerVersion,
}

/// 복원할 파일을 고르지 못했다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SourceError {
    /// 파일 이름과 경로 중 하나만 채워야 한다. 화면 코드의 잘못이다.
    #[error("파일 이름과 경로 중 하나만 보내야 합니다")]
    Malformed,
    #[error("백업 목록에 없는 파일입니다")]
    NotInList,
    #[error("파일 경로는 드라이브부터 적어야 합니다")]
    NotAbsolute,
    #[error("백업 파일이 없습니다. 외장 디스크나 USB가 꽂혀 있는지 확인해 주세요")]
    Missing { field: &'static str },
}

impl SourceError {
    /// 오류를 붙일 요청 칸. [`SourceError::Malformed`]는 칸이 없다.
    pub fn field(self) -> Option<&'static str> {
        match self {
            SourceError::Malformed => None,
            SourceError::NotInList => Some("fileName"),
            SourceError::NotAbsolute => Some("path"),
            SourceError::Missing { field } => Some(field),
        }
    }
}

/// 복원을 준비하지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum PrepareError {
    #[error(transparent)]
    Inspect(#[from] InspectError),
    /// 복원 직전 백업을 만들지 못하면 복원하지 않는다.
    #[error("지금 상태를 백업하지 못해 복원하지 않았습니다: {0}")]
    Backup(#[source] BackupError),
    #[error("백업 파일을 받아 두지 못했습니다: {0}")]
    Io(#[from] std::io::Error),
}

/// 깨진 DB를 되돌리지 못했다. 깨진 DB는 제자리에 그대로 있다.
#[derive(Debug, thiserror::Error)]
pub enum BrokenRestoreError {
    #[error(transparent)]
    Inspect(#[from] InspectError),
    #[error("백업 파일을 옮기지 못했습니다: {0}")]
    Io(#[from] std::io::Error),
    #[error("되돌린 DB를 열지 못했습니다: {0}")]
    Open(#[from] DbError),
}

/// 두 번째 위치를 바꾸지 못했다.
#[derive(Debug, thiserror::Error)]
pub enum SecondLocationError {
    #[error("폴더 경로는 드라이브부터 적어야 합니다")]
    NotAbsolute,
    #[error("없는 폴더입니다. 외장 디스크나 USB가 꽂혀 있는지 확인해 주세요")]
    NotFolder,
    #[error("첫 번째 백업 폴더와 같은 곳은 고를 수 없습니다. 다른 디스크의 폴더를 골라 주세요")]
    SameAsFirst,
    #[error("이 폴더에 파일을 쓸 수 없습니다. 다른 폴더를 골라 주세요")]
    NotWritable(#[source] std::io::Error),
    #[error("백업 설정을 저장하지 못했습니다: {0}")]
    Save(#[source] std::io::Error),
}
