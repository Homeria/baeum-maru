//! 게시용 강좌별 명단(BR-P3, BR-P4): 강좌마다 시트 하나. 당첨자 다음에 대기자를 순번대로 둔다.
//!
//! 포기한 신청은 넣지 않는다. 자리 보장으로 당첨된 줄은 색칠한다.

use baeum_domain::RosterSpec;
use rust_xlsxwriter::{Workbook, XlsxError};

use super::{PostedFormats, cell, posted_column_width, posted_table_columns, print_on_a4};
use crate::db::rosters::{RosterCourse, RosterResult};

/// Excel 시트 이름의 최대 글자 수.
const SHEET_NAME_MAX: usize = 31;

/// 게시용 강좌별 명단 Excel 파일의 내용을 만든다.
pub fn posted_by_course(spec: &RosterSpec, roster: &[RosterCourse]) -> Result<Vec<u8>, XlsxError> {
    let formats = PostedFormats::new();
    let columns = posted_table_columns(spec);
    let last_col = u16::try_from(columns.len().saturating_sub(1)).unwrap_or(u16::MAX);
    let names = sheet_names(roster);

    let mut workbook = Workbook::new();
    for (course, name) in roster.iter().zip(names) {
        let sheet = workbook.add_worksheet();
        sheet.set_name(name)?;
        let title = format!("{} (정원 {}명)", course.name, course.capacity.value());
        if last_col == 0 {
            sheet.write_string_with_format(0, 0, &title, &formats.title)?;
        } else {
            sheet.merge_range(0, 0, 0, last_col, &title, &formats.title)?;
        }
        sheet.set_row_height(0, 36)?;
        for (col, column) in (0u16..).zip(&columns) {
            sheet.write_string_with_format(2, col, column.label(), &formats.header)?;
            sheet.set_column_width(col, posted_column_width(*column))?;
        }
        let shown = course
            .entries
            .iter()
            .filter(|entry| entry.result != RosterResult::Withdrawn);
        for (row, entry) in (3u32..).zip(shown) {
            let format = if entry.guaranteed {
                &formats.priority
            } else {
                &formats.body
            };
            for (col, column) in (0u16..).zip(&columns) {
                cell(spec, course, entry, *column).write(sheet, row, col, format)?;
            }
            sheet.set_row_height(row, 24)?;
        }
        sheet.set_repeat_rows(2, 2)?;
        print_on_a4(sheet);
    }
    workbook.save_to_buffer()
}

/// 강좌마다 시트 이름 `강좌 이름(정원)`.
///
/// Excel 규칙에 맞춘다: 31자 이하, `: \ / ? * [ ]`는 쓸 수 없다. 길면 강좌 이름을 자른다.
/// 겹치면(대소문자 무시) 이름 뒤에 `-2`, `-3`을 붙인다.
fn sheet_names(roster: &[RosterCourse]) -> Vec<String> {
    let mut taken: Vec<String> = Vec::with_capacity(roster.len());
    let mut names = Vec::with_capacity(roster.len());
    for course in roster {
        let base: String = course
            .name
            .chars()
            .filter(|c| !matches!(c, ':' | '\\' | '/' | '?' | '*' | '[' | ']'))
            .collect();
        let base = base.trim_matches(|c: char| c == '\'' || c.is_whitespace());
        let capacity = format!("({})", course.capacity.value());
        let name = (1..)
            .map(|n| {
                let suffix = if n == 1 {
                    capacity.clone()
                } else {
                    format!("-{n}{capacity}")
                };
                let room = SHEET_NAME_MAX.saturating_sub(suffix.chars().count());
                let head: String = base.chars().take(room).collect();
                format!("{}{suffix}", head.trim_end())
            })
            .find(|candidate| !taken.contains(&candidate.to_lowercase()))
            .unwrap_or_default();
        taken.push(name.to_lowercase());
        names.push(name);
    }
    names
}

#[cfg(test)]
mod tests {
    use baeum_domain::{Capacity, MaskChar, RosterChoice, RosterColumn, RosterKind};

    use super::*;
    use crate::roster_xlsx::test_support::{read_sheet, sample_roster};

    fn spec(include: &[RosterColumn], mask: &[RosterColumn]) -> RosterSpec {
        RosterSpec::new(RosterChoice {
            kind: RosterKind::PostedByCourse,
            include,
            mask,
            mask_char: MaskChar::Circle,
        })
        .unwrap()
    }

    #[test]
    fn br_p3_one_sheet_per_course_without_withdrawn() {
        let bytes = posted_by_course(&spec(&[], &[RosterColumn::Name]), &sample_roster()).unwrap();

        let (name, rows) = read_sheet(&bytes, 0);
        assert_eq!(name, "요가 초급(2)");
        assert_eq!(rows[0][0], "요가 초급 (정원 2명)");
        assert_eq!(
            rows[2..],
            [
                ["결과", "대기 순번", "이름", "회원번호"],
                ["당첨", "", "홍○동", "00012"],
                ["대기", "1", "김○", "00034"],
            ]
        );
        let (second, _) = read_sheet(&bytes, 1);
        assert_eq!(second, "서예(1)");
    }

    #[test]
    fn br_p5_chosen_phone_is_masked() {
        let bytes =
            posted_by_course(&spec(&[RosterColumn::PhoneNumber], &[]), &sample_roster()).unwrap();

        let (_, rows) = read_sheet(&bytes, 0);
        assert_eq!(rows[2][4], "연락처");
        assert_eq!(rows[3][4], "010-○○○○-5678");
    }

    #[test]
    fn sheet_names_follow_excel_rules() {
        let course = |name: &str| RosterCourse {
            course_id: 1,
            name: name.to_owned(),
            capacity: Capacity::new(20).unwrap(),
            entries: Vec::new(),
        };
        let roster = [
            course("스마트폰: 카톡/사진 [기초]"),
            course("요가"),
            course("요가"),
            course("아주 긴 강좌 이름이 들어가는 경우를 위한 시험용 강좌 이름 초급 1반"),
        ];

        let names = sheet_names(&roster);

        assert_eq!(names[0], "스마트폰 카톡사진 기초(20)");
        assert_eq!(names[1], "요가(20)");
        assert_eq!(names[2], "요가-2(20)");
        assert_eq!(names[3].chars().count(), 31);
        assert!(names[3].ends_with("(20)"));
    }
}
