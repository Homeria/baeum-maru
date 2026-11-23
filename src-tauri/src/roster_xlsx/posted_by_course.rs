//! 게시용 강좌별 명단(BR-P3, BR-P4): 강좌마다 시트 하나. 당첨자 다음에 대기자를 순번대로 둔다.
//!
//! 포기한 신청은 넣지 않는다. 자리 보장으로 당첨된 줄은 색칠한다.

use baeum_domain::RosterSpec;
use rust_xlsxwriter::{Workbook, XlsxError};

use super::{
    PostedFormats, cell, posted_column_width, posted_table_columns, print_on_a4, sheet_names,
};
use crate::db::rosters::{RosterCourse, RosterResult};

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

#[cfg(test)]
mod tests {
    use baeum_domain::{MaskChar, RosterChoice, RosterColumn, RosterKind};

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
}
