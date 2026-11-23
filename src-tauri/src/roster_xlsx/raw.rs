//! 원본 명단(BR-P2): 한 시트에 신청 하나가 한 줄인 긴 표. 직원이 필터하고 다시 꾸며 쓴다.

use baeum_domain::{RosterKind, RosterSpec};
use rust_xlsxwriter::{Format, FormatBorder, Workbook, XlsxError};

use super::{FONT, cell, column_width, print_on_a4};
use crate::db::rosters::RosterCourse;

/// 원본 명단 시트 이름.
const SHEET_NAME: &str = "결과 명단";

/// 원본 명단 Excel 파일의 내용을 만든다.
///
/// 첫 줄은 머리글이고 고정한다. 자동 필터를 켜 둔다. 칸 순서는 `spec`의 순서다.
pub fn raw_roster(spec: &RosterSpec, roster: &[RosterCourse]) -> Result<Vec<u8>, XlsxError> {
    debug_assert_eq!(spec.kind(), RosterKind::Raw);
    let header = Format::new()
        .set_font_name(FONT)
        .set_bold()
        .set_border(FormatBorder::Thin)
        .set_background_color("#E8E8E8");
    let body = Format::new()
        .set_font_name(FONT)
        .set_border(FormatBorder::Thin);

    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();
    sheet.set_name(SHEET_NAME)?;
    let columns = spec.columns();
    for (col, column) in (0u16..).zip(columns) {
        sheet.write_string_with_format(0, col, column.label(), &header)?;
        sheet.set_column_width(col, column_width(*column))?;
    }

    let entries = roster
        .iter()
        .flat_map(|course| course.entries.iter().map(move |entry| (course, entry)));
    let mut last_row = 0;
    for (row, (course, entry)) in (1u32..).zip(entries) {
        for (col, column) in (0u16..).zip(columns) {
            cell(spec, course, entry, *column).write(sheet, row, col, &body)?;
        }
        last_row = row;
    }

    sheet.set_freeze_panes(1, 0)?;
    let last_col = u16::try_from(columns.len().saturating_sub(1)).unwrap_or(u16::MAX);
    sheet.autofilter(0, 0, last_row, last_col)?;
    print_on_a4(sheet);
    sheet.set_repeat_rows(0, 0)?;
    workbook.save_to_buffer()
}

#[cfg(test)]
mod tests {
    use baeum_domain::{MaskChar, RosterChoice, RosterColumn};

    use super::*;
    use crate::roster_xlsx::test_support::{read_sheet, sample_roster};

    fn spec(include: &[RosterColumn], mask: &[RosterColumn]) -> RosterSpec {
        RosterSpec::new(RosterChoice {
            kind: RosterKind::Raw,
            include,
            mask,
            mask_char: MaskChar::Circle,
        })
        .unwrap()
    }

    #[test]
    fn br_p2_writes_one_row_per_application_with_fixed_columns() {
        let bytes = raw_roster(&spec(&[], &[]), &sample_roster()).unwrap();

        let (name, rows) = read_sheet(&bytes, 0);
        assert_eq!(name, "결과 명단");
        assert_eq!(
            rows,
            [
                ["강좌", "결과", "이름", "회원번호"],
                ["요가 초급", "당첨", "홍길동", "00012"],
                ["요가 초급", "대기", "김철", "00034"],
                ["요가 초급", "포기", "남궁민수", "00056"],
                ["서예", "당첨", "박영희", "00078"],
            ]
        );
    }

    #[test]
    fn br_p6_p7_writes_chosen_columns_masked_or_not() {
        use RosterColumn::*;
        let chosen = spec(
            &[
                Capacity,
                WaitlistNumber,
                PhoneNumber,
                BirthDate,
                Gender,
                Guaranteed,
            ],
            &[Name, BirthDate],
        );

        let bytes = raw_roster(&chosen, &sample_roster()).unwrap();

        let (_, rows) = read_sheet(&bytes, 0);
        assert_eq!(
            rows[0],
            [
                "강좌",
                "정원",
                "결과",
                "대기 순번",
                "이름",
                "회원번호",
                "연락처",
                "생년월일",
                "성별",
                "우선"
            ]
        );
        assert_eq!(
            rows[1],
            [
                "요가 초급",
                "2",
                "당첨",
                "",
                "홍○동",
                "00012",
                "010-1234-5678",
                "1950년",
                "남",
                "우선"
            ]
        );
        assert_eq!(
            rows[2],
            [
                "요가 초급",
                "2",
                "대기",
                "1",
                "김○",
                "00034",
                "02-123-4567",
                "",
                "",
                ""
            ]
        );
    }
}
