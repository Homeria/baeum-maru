//! 백업 파일 이름. 예: `20261123-100200_daily.db`, `20261123-143005_before-lottery-c12.db`
//!
//! 시각은 이 PC의 지역 시각이다. 직원이 폴더를 열어 봐도 언제 것인지 알 수 있게 한다.

use std::fmt;

use time::PrimitiveDateTime;
use time::format_description::BorrowedFormatItem;
use time::macros::format_description;

/// 파일 이름의 시각 부분.
const TIME_FORMAT: &[BorrowedFormatItem<'static>] =
    format_description!("[year][month][day]-[hour][minute][second]");
/// 백업 파일의 확장자. DB 파일과 같다.
const EXTENSION: &str = ".db";

/// 백업을 만든 때(ADR 0011). 종류마다 남기는 개수가 다르다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BackupKind {
    /// 하루 한 번.
    Daily,
    /// 관리자가 "지금 백업"을 눌렀다.
    Manual,
    /// 앱을 끌 때.
    Exit,
    /// 추첨 직전.
    BeforeLottery { course_id: i64 },
    /// 선발 마감 직전.
    BeforeClose { course_id: i64 },
    /// DB 구조 변경 직전. 지우지 않는다.
    BeforeMigration,
    /// 복원 직전. 복원 자체를 되돌릴 수 있게 한다.
    BeforeRestore,
    /// 호스트 넘기기 직전(BR-H1). 넘기기 파일과 같은 내용이다.
    BeforeHandover,
}

impl BackupKind {
    /// 파일 이름에 쓰는 글자. 예: `before-lottery-c12`
    fn code(self) -> String {
        match self {
            BackupKind::Daily => "daily".to_owned(),
            BackupKind::Manual => "manual".to_owned(),
            BackupKind::Exit => "exit".to_owned(),
            BackupKind::BeforeLottery { course_id } => format!("before-lottery-c{course_id}"),
            BackupKind::BeforeClose { course_id } => format!("before-close-c{course_id}"),
            BackupKind::BeforeMigration => "before-migration".to_owned(),
            BackupKind::BeforeRestore => "before-restore".to_owned(),
            BackupKind::BeforeHandover => "before-handover".to_owned(),
        }
    }

    /// [`BackupKind::code`]를 거꾸로 읽는다. 모르는 글자면 `None`.
    fn parse(code: &str) -> Option<Self> {
        let course = |prefix: &str| -> Option<i64> {
            code.strip_prefix(prefix)?.parse().ok().filter(|id| *id > 0)
        };
        match code {
            "daily" => Some(BackupKind::Daily),
            "manual" => Some(BackupKind::Manual),
            "exit" => Some(BackupKind::Exit),
            "before-migration" => Some(BackupKind::BeforeMigration),
            "before-restore" => Some(BackupKind::BeforeRestore),
            "before-handover" => Some(BackupKind::BeforeHandover),
            _ => course("before-lottery-c")
                .map(|course_id| BackupKind::BeforeLottery { course_id })
                .or_else(|| {
                    course("before-close-c").map(|course_id| BackupKind::BeforeClose { course_id })
                }),
        }
    }
}

/// 백업 파일 하나가 언제, 왜 만들어졌는지. 시각이 앞서는 순으로 정렬된다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BackupName {
    /// 이 PC의 지역 시각, 초까지.
    pub taken_at: PrimitiveDateTime,
    pub kind: BackupKind,
}

impl BackupName {
    /// 파일 이름을 읽는다. 백업 파일 이름이 아니면(쓰는 중인 임시 파일 등) `None`.
    pub fn parse(file_name: &str) -> Option<Self> {
        let stem = file_name.strip_suffix(EXTENSION)?;
        let (time, code) = stem.split_once('_')?;
        Some(BackupName {
            taken_at: PrimitiveDateTime::parse(time, TIME_FORMAT).ok()?,
            kind: BackupKind::parse(code)?,
        })
    }
}

/// 파일 이름. 예: `20261123-100200_daily.db`
impl fmt::Display for BackupName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let time = self.taken_at.format(TIME_FORMAT).map_err(|_| fmt::Error)?;
        write!(f, "{time}_{}{EXTENSION}", self.kind.code())
    }
}

#[cfg(test)]
mod tests {
    use time::macros::datetime;

    use super::*;

    fn name(kind: BackupKind) -> BackupName {
        BackupName {
            taken_at: datetime!(2026-11-23 09:05:07),
            kind,
        }
    }

    #[test]
    fn file_name_shows_local_time_and_kind() {
        assert_eq!(
            name(BackupKind::Daily).to_string(),
            "20261123-090507_daily.db"
        );
        assert_eq!(
            name(BackupKind::BeforeLottery { course_id: 12 }).to_string(),
            "20261123-090507_before-lottery-c12.db"
        );
    }

    #[test]
    fn file_name_reads_back_for_every_kind() {
        let kinds = [
            BackupKind::Daily,
            BackupKind::Manual,
            BackupKind::Exit,
            BackupKind::BeforeLottery { course_id: 3 },
            BackupKind::BeforeClose { course_id: 45 },
            BackupKind::BeforeMigration,
            BackupKind::BeforeRestore,
        ];
        for kind in kinds {
            let written = name(kind);
            assert_eq!(BackupName::parse(&written.to_string()), Some(written));
        }
    }

    #[test]
    fn other_files_are_not_backups() {
        for file in [
            "20261123-090507_daily.db.part",
            "20261123-090507_weekly.db",
            "20261123-090507_before-lottery-c.db",
            "20261123-090507_before-close-c0.db",
            "2026-11-23_daily.db",
            "baeum-maru.db",
            "메모.txt",
        ] {
            assert_eq!(BackupName::parse(file), None, "{file}");
        }
    }
}
