//! 게시용 강좌별 명단(BR-P3, BR-P4): 강좌마다 시트 하나. 맨 위에 강좌 이름과 정원을 제목으로
//! 쓰고, 원본처럼 당첨자와 대기자를 나란히 둔다.
//!
//! 붙여 두고 어르신들이 보므로 글씨를 크게 한다. 포기한 신청은 넣지 않는다.

use baeum_domain::RosterSpec;
use rust_xlsxwriter::{Workbook, XlsxError};

use super::side_by_side::{table_width, write_side_by_side};
use super::{PostedHeading, TableFormats, print_on_a4, sheet_names};
use crate::db::rosters::RosterCourse;

/// 게시용 표의 글씨 크기.
const FONT_SIZE: f64 = 14.0;
/// 글씨가 커서 칸도 원본보다 넓게.
const WIDTH_SCALE: f64 = 1.4;
/// 제목 아래 안내 줄.
const NOTE: &str =
    "※ 동명이인은 회원번호를 함께 확인해 주세요. 대기 순번은 이 강좌의 대기자 명단 순서입니다.";
/// 표 머리글 줄. 0번 제목, 1번 안내, 2번은 비운다.
const HEADER_ROW: u32 = 3;

/// 게시용 강좌별 명단 Excel 파일의 내용을 만든다.
pub fn posted_by_course(spec: &RosterSpec, roster: &[RosterCourse]) -> Result<Vec<u8>, XlsxError> {
    let heading = PostedHeading::new();
    let formats = TableFormats::new(FONT_SIZE);
    let width = table_width(spec);
    let names = sheet_names(roster);

    let mut workbook = Workbook::new();
    for (course, name) in roster.iter().zip(names) {
        let sheet = workbook.add_worksheet();
        sheet.set_name(name)?;
        let title = format!("{} (정원 {}명)", course.name, course.capacity.value());
        heading.write(sheet, width, &title, NOTE)?;
        write_side_by_side(sheet, spec, course, &formats, HEADER_ROW, WIDTH_SCALE)?;
        sheet.set_repeat_rows(HEADER_ROW, HEADER_ROW)?;
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
        assert!(rows[1][0].starts_with("※ 동명이인은"));
        assert_eq!(
            rows[3..],
            [
                ["당첨자", "회원번호", "대기자", "회원번호", "대기 순번"],
                ["홍○동", "00012", "김○", "00034", "1"],
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
        assert_eq!(rows[3][..3], ["당첨자", "회원번호", "연락처"]);
        assert_eq!(rows[4][2], "010-○○○○-5678");
        assert_eq!(rows[4][5], "02-○○○-4567");
    }
}
