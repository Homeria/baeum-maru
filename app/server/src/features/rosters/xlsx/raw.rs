//! 원본 명단(BR-P2): 강좌마다 시트 하나. 맨 위에 강좌 이름과 정원을 제목으로 쓰고, 왼쪽에
//! 당첨자, 오른쪽에 대기자를 나란히 둔다.
//!
//! 사무용이라 보통 크기 글씨이고 안내 줄은 없다. 포기한 신청은 넣지 않는다.

use baeum_domain::{RosterKind, RosterSpec};
use rust_xlsxwriter::{Workbook, XlsxError};

use super::side_by_side::{table_width, write_side_by_side};
use super::{Heading, TableFormats, course_title, print_on_a4, sheet_names};
use crate::features::rosters::repository::RosterCourse;

/// 원본 명단의 글씨 크기. Excel 기본과 같다.
const FONT_SIZE: f64 = 11.0;
/// 제목 글씨 크기. 게시용보다 작게.
const TITLE_SIZE: f64 = 16.0;
/// 표 머리글 줄. 0번 제목, 1번은 비운다.
const HEADER_ROW: u32 = 2;

/// 원본 명단 Excel 파일의 내용을 만든다. 머리글 줄까지 고정한다.
pub fn raw_roster(spec: &RosterSpec, roster: &[RosterCourse]) -> Result<Vec<u8>, XlsxError> {
    debug_assert_eq!(spec.kind(), RosterKind::Raw);
    let heading = Heading::new(TITLE_SIZE);
    let formats = TableFormats::new(FONT_SIZE);
    let width = table_width(spec);
    let names = sheet_names(roster);

    let mut workbook = Workbook::new();
    for (course, name) in roster.iter().zip(names) {
        let sheet = workbook.add_worksheet();
        sheet.set_name(name)?;
        heading.write(sheet, width, &course_title(course), &[])?;
        write_side_by_side(sheet, spec, course, &formats, HEADER_ROW, 1.0)?;
        sheet.set_freeze_panes(HEADER_ROW + 1, 0)?;
        sheet.set_repeat_rows(HEADER_ROW, HEADER_ROW)?;
        print_on_a4(sheet);
    }
    workbook.save_to_buffer()
}

#[cfg(test)]
mod tests {
    use baeum_domain::{MaskChar, RosterChoice, RosterColumn};

    use super::*;
    use crate::features::rosters::xlsx::test_support::{read_sheet, sample_roster};

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
    fn br_p2_one_sheet_per_course_with_selected_and_waitlisted_side_by_side() {
        let mut roster = sample_roster();
        // 서예에 대기자 두 명을 더 둔다. 당첨자(한 명)보다 길다
        let waiting = roster[0].entries[1].clone();
        for number in 1..=2 {
            let mut more = waiting.clone();
            more.waitlist_number = Some(number);
            roster[1].entries.push(more);
        }

        let bytes = raw_roster(&spec(&[], &[]), &roster).unwrap();

        let (name, rows) = read_sheet(&bytes, 0);
        assert_eq!(name, "요가 초급(2)");
        assert_eq!(rows[0][0], "요가 초급 (정원 2명)");
        // 포기한 남궁민수는 없다
        assert_eq!(
            rows[2..],
            [
                ["당첨자", "회원번호", "대기자", "회원번호", "대기 순번"],
                ["홍길동", "00012", "김철", "00034", "1"],
            ]
        );
        let (second, rows) = read_sheet(&bytes, 1);
        assert_eq!(second, "서예(1)");
        assert_eq!(rows[3], ["박영희", "00078", "김철", "00034", "1"]);
        assert_eq!(rows[4], ["", "", "김철", "00034", "2"]);
    }

    #[test]
    fn br_p6_p7_chosen_columns_go_in_each_group_masked_or_not() {
        use RosterColumn::*;
        let chosen = spec(&[PhoneNumber, BirthDate, Gender], &[Name, BirthDate]);

        let bytes = raw_roster(&chosen, &sample_roster()).unwrap();

        let (_, rows) = read_sheet(&bytes, 0);
        assert_eq!(
            rows[2..],
            [
                [
                    "당첨자",
                    "회원번호",
                    "연락처",
                    "생년월일",
                    "성별",
                    "대기자",
                    "회원번호",
                    "연락처",
                    "생년월일",
                    "성별",
                    "대기 순번"
                ],
                [
                    "홍○동",
                    "00012",
                    "010-1234-5678",
                    "1950년",
                    "남",
                    "김○",
                    "00034",
                    "02-123-4567",
                    "",
                    "",
                    "1"
                ],
            ]
        );
    }
}
