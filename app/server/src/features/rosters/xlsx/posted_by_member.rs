//! 게시용 회원별 명단(BR-P3, BR-P4): 회원 한 명이 한 줄. 회원이 자기 이름 하나만 찾으면
//! 모든 결과를 본다.
//!
//! 줄은 실제 이름의 가나다순이고 줄 번호를 붙인다(가려서 보여 줘도 찾기 쉽게). 당첨 강좌와
//! 대기 강좌는 각각 한 칸에 강좌마다 줄을 바꿔 적는다. 한 칸에 여러 강좌가 있어 칸을 칠하지
//! 않고, 자리 보장으로 당첨된 줄의 글자색만 바꾼다. 포기한 신청과 포기만 남은 회원은 넣지 않는다.

use std::collections::HashMap;

use baeum_domain::{RosterColumn, RosterSpec};
use rust_xlsxwriter::{Format, FormatAlign, Workbook, Worksheet, XlsxError};

use super::{
    FONT, Heading, PRIORITY_TEXT, TableFormats, cell, column_width, print_on_a4, text_width,
};
use crate::features::rosters::repository::{RosterCourse, RosterEntry, RosterResult};

/// 회원별 명단 시트 이름.
const SHEET_NAME: &str = "회원별 결과";
/// 게시용 표의 글씨 크기.
const FONT_SIZE: f64 = 14.0;
/// 글씨가 커서 칸도 원본보다 넓게.
const WIDTH_SCALE: f64 = 1.4;
/// 제목 아래 안내 줄.
const NOTES: [&str; 2] = [
    "이름을 찾으면 신청한 모든 강좌의 당첨·대기 결과를 한 번에 볼 수 있습니다.",
    "※ 동명이인은 회원번호를 함께 확인해 주세요. 대기 순번은 강좌별 대기자 명단 순서입니다.",
];
/// 표 머리글 줄. 0번 제목, 1~2번 안내.
const HEADER_ROW: u32 = 3;
/// 결과 칸 한 줄의 높이.
const LINE_HEIGHT: f64 = 21.0;

/// 회원 한 명과 그 회원의 신청들(강좌 순).
struct MemberRow<'a> {
    results: Vec<(&'a RosterCourse, &'a RosterEntry)>,
}

impl MemberRow<'_> {
    /// 회원 정보 칸을 채울 때 쓰는 첫 신청.
    fn first(&self) -> (&RosterCourse, &RosterEntry) {
        self.results[0]
    }

    /// `result`인 신청들의 결과 칸 줄. 자리 보장 당첨이면 `true`가 붙는다.
    fn lines(&self, result: RosterResult) -> Vec<(String, bool)> {
        self.results
            .iter()
            .filter(|(_, entry)| entry.result == result)
            .map(|(course, entry)| (result_line(course, entry), entry.guaranteed))
            .collect()
    }
}

/// 회원별 명단의 칸 모양.
struct MemberFormats {
    table: TableFormats,
    /// 이름 칸. 찾기 쉽게 굵게.
    name: Format,
    /// 당첨·대기 칸. 여러 줄이라 왼쪽 정렬, 줄바꿈.
    result: Format,
    /// 결과 칸 안의 보통 줄 글자.
    line: Format,
    /// 결과 칸 안의 자리 보장 당첨 줄 글자(BR-P4).
    priority_line: Format,
}

impl MemberFormats {
    fn new() -> Self {
        let table = TableFormats::new(FONT_SIZE);
        let line = Format::new()
            .set_font_name(FONT)
            .set_font_size(FONT_SIZE)
            .set_bold();
        MemberFormats {
            name: table.body.clone().set_bold(),
            result: table
                .body
                .clone()
                .set_bold()
                .set_align(FormatAlign::Left)
                // 칸 왼쪽 선에 붙지 않게 한 칸 들여쓴다
                .set_indent(1)
                .set_text_wrap(),
            priority_line: line.clone().set_font_color(PRIORITY_TEXT),
            line,
            table,
        }
    }
}

/// 게시용 회원별 명단 Excel 파일의 내용을 만든다. `term_name`은 제목에 쓴다.
pub fn posted_by_member(
    spec: &RosterSpec,
    roster: &[RosterCourse],
    term_name: &str,
) -> Result<Vec<u8>, XlsxError> {
    let formats = MemberFormats::new();
    let member_columns = member_columns(spec);
    let rows = member_rows(roster);
    let member_count = u16::try_from(member_columns.len()).unwrap_or(u16::MAX);
    // 번호 + 회원 칸 + 당첨 + 대기
    let selected_col = member_count + 1;
    let waitlisted_col = selected_col + 1;

    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();
    sheet.set_name(SHEET_NAME)?;
    let title = format!("{term_name} 회원별 선발 결과");
    Heading::new(20.0).write(sheet, waitlisted_col + 1, &title, &NOTES)?;

    let header = &formats.table.header;
    sheet.write_string_with_format(HEADER_ROW, 0, "번호", header)?;
    sheet.set_column_width(0, 8.0)?;
    for (col, column) in (1u16..).zip(&member_columns) {
        sheet.write_string_with_format(HEADER_ROW, col, column.label(), header)?;
        sheet.set_column_width(col, column_width(*column) * WIDTH_SCALE)?;
    }
    for (col, label, result) in [
        (selected_col, "당첨 강좌", RosterResult::Selected),
        (waitlisted_col, "대기 강좌(순번)", RosterResult::Waitlisted),
    ] {
        sheet.write_string_with_format(HEADER_ROW, col, label, header)?;
        let widest = rows
            .iter()
            .flat_map(|row| row.lines(result))
            .map(|(line, _)| text_width(&line))
            .fold(text_width(label), f64::max);
        sheet.set_column_width(col, widest * WIDTH_SCALE)?;
    }

    for ((row, number), member) in (HEADER_ROW + 1..).zip(1u32..).zip(&rows) {
        let (course, entry) = member.first();
        sheet.write_number_with_format(row, 0, number, &formats.table.body)?;
        for (col, column) in (1u16..).zip(&member_columns) {
            let format = if *column == RosterColumn::Name {
                &formats.name
            } else {
                &formats.table.body
            };
            cell(spec, course, entry, *column).write(sheet, row, col, format)?;
        }
        let selected = member.lines(RosterResult::Selected);
        let waitlisted = member.lines(RosterResult::Waitlisted);
        write_lines(sheet, row, selected_col, &selected, &formats)?;
        write_lines(sheet, row, waitlisted_col, &waitlisted, &formats)?;
        let tallest = selected.len().max(waitlisted.len()).max(1);
        let height = LINE_HEIGHT * f64::from(u32::try_from(tallest).unwrap_or(u32::MAX)) + 6.0;
        sheet.set_row_height(row, height)?;
    }
    sheet.set_repeat_rows(HEADER_ROW, HEADER_ROW)?;
    print_on_a4(sheet);
    workbook.save_to_buffer()
}

/// 번호 다음에 넣는 회원 칸: 이름, 회원번호, 고른 칸(연락처, 생년월일, 성별).
fn member_columns(spec: &RosterSpec) -> Vec<RosterColumn> {
    spec.columns()
        .iter()
        .copied()
        .filter(|column| {
            matches!(
                column,
                RosterColumn::Name
                    | RosterColumn::MemberNo
                    | RosterColumn::PhoneNumber
                    | RosterColumn::BirthDate
                    | RosterColumn::Gender
            )
        })
        .collect()
}

/// 결과 칸 하나에 줄들을 쓴다. 없으면 `-`. 자리 보장 당첨 줄이 있으면 그 줄만 글자색을 바꾼다.
fn write_lines(
    sheet: &mut Worksheet,
    row: u32,
    col: u16,
    lines: &[(String, bool)],
    formats: &MemberFormats,
) -> Result<(), XlsxError> {
    if lines.is_empty() {
        sheet.write_string_with_format(row, col, "-", &formats.result)?;
    } else if lines.iter().any(|(_, guaranteed)| *guaranteed) {
        // 줄마다 조각 하나. 마지막 줄 말고는 끝에 줄바꿈을 붙인다
        let texts: Vec<String> = lines
            .iter()
            .enumerate()
            .map(|(index, (line, _))| {
                if index + 1 < lines.len() {
                    format!("{line}\n")
                } else {
                    line.clone()
                }
            })
            .collect();
        let segments: Vec<(&Format, &str)> = lines
            .iter()
            .zip(&texts)
            .map(|((_, guaranteed), text)| {
                let format = if *guaranteed {
                    &formats.priority_line
                } else {
                    &formats.line
                };
                (format, text.as_str())
            })
            .collect();
        sheet.write_rich_string_with_format(row, col, &segments, &formats.result)?;
    } else {
        let text: Vec<&str> = lines.iter().map(|(line, _)| line.as_str()).collect();
        sheet.write_string_with_format(row, col, text.join("\n"), &formats.result)?;
    }
    Ok(())
}

/// 회원마다 포기가 아닌 신청을 모은다. 실제 이름, 회원번호 순.
fn member_rows(roster: &[RosterCourse]) -> Vec<MemberRow<'_>> {
    let mut by_member: HashMap<i64, MemberRow<'_>> = HashMap::new();
    for course in roster {
        for entry in &course.entries {
            if entry.result == RosterResult::Withdrawn {
                continue;
            }
            by_member
                .entry(entry.member.member_id)
                .or_insert_with(|| MemberRow {
                    results: Vec::new(),
                })
                .results
                .push((course, entry));
        }
    }
    let mut rows: Vec<MemberRow<'_>> = by_member.into_values().collect();
    rows.sort_by(|a, b| {
        let (a, b) = (&a.first().1.member, &b.first().1.member);
        // 한글 음절은 유니코드에서 가나다순으로 놓여 있다
        (a.name.expose(), a.member_no.expose()).cmp(&(b.name.expose(), b.member_no.expose()))
    });
    rows
}

/// 결과 칸의 한 줄. 예: `당첨 | 요가 초급`, `대기 2번 | 서예`
fn result_line(course: &RosterCourse, entry: &RosterEntry) -> String {
    let result = match (entry.result, entry.waitlist_number) {
        (RosterResult::Waitlisted, Some(number)) => format!("대기 {number}번"),
        (RosterResult::Waitlisted, None) => "대기".to_owned(),
        (RosterResult::Selected, _) => "당첨".to_owned(),
        (RosterResult::Withdrawn, _) => "포기".to_owned(),
    };
    format!("{result} | {}", course.name)
}

#[cfg(test)]
mod tests {
    use baeum_domain::{MaskChar, RosterChoice, RosterKind};

    use super::*;
    use crate::features::rosters::xlsx::test_support::{read_sheet, sample_roster};

    fn spec(include: &[RosterColumn], mask: &[RosterColumn]) -> RosterSpec {
        RosterSpec::new(RosterChoice {
            kind: RosterKind::PostedByMember,
            include,
            mask,
            mask_char: MaskChar::Circle,
        })
        .unwrap()
    }

    #[test]
    fn br_p3_one_row_per_member_in_name_order_with_result_lines() {
        let mut roster = sample_roster();
        // 홍길동이 서예에도 대기 2번으로 있다
        let mut again = roster[0].entries[0].clone();
        again.result = RosterResult::Waitlisted;
        again.waitlist_number = Some(2);
        again.guaranteed = false;
        roster[1].entries.push(again);

        let bytes =
            posted_by_member(&spec(&[], &[RosterColumn::Name]), &roster, "2027년 1학기").unwrap();

        let (name, rows) = read_sheet(&bytes, 0);
        assert_eq!(name, "회원별 결과");
        assert_eq!(rows[0][0], "2027년 1학기 회원별 선발 결과");
        assert!(rows[2][0].starts_with("※ 동명이인은"));
        assert_eq!(
            rows[3..],
            [
                ["번호", "이름", "회원번호", "당첨 강좌", "대기 강좌(순번)"],
                ["1", "김○", "00034", "-", "대기 1번 | 요가 초급"],
                ["2", "박○희", "00078", "당첨 | 서예", "-"],
                ["3", "홍○동", "00012", "당첨 | 요가 초급", "대기 2번 | 서예"],
            ]
        );
    }

    #[test]
    fn several_courses_go_on_separate_lines_in_one_cell() {
        let mut roster = sample_roster();
        // 박영희가 요가 초급에도 당첨(자리 보장 아님)
        let mut again = roster[1].entries[0].clone();
        again.guaranteed = false;
        roster[0].entries.push(again);
        // 홍길동(자리 보장)이 서예에도 당첨
        let mut guaranteed = roster[0].entries[0].clone();
        guaranteed.guaranteed = false;
        roster[1].entries.push(guaranteed);

        let bytes = posted_by_member(&spec(&[], &[]), &roster, "2027년 1학기").unwrap();

        let (_, rows) = read_sheet(&bytes, 0);
        assert_eq!(rows[5][3], "당첨 | 요가 초급\n당첨 | 서예");
        // 자리 보장 줄이 섞여도(글자색만 다름) 글자는 같다
        assert_eq!(rows[6][3], "당첨 | 요가 초급\n당첨 | 서예");
    }

    #[test]
    fn br_p5_member_details_follow_member_no_and_are_masked() {
        let bytes = posted_by_member(
            &spec(&[RosterColumn::BirthDate, RosterColumn::Gender], &[]),
            &sample_roster(),
            "2027년 1학기",
        )
        .unwrap();

        let (_, rows) = read_sheet(&bytes, 0);
        assert_eq!(
            rows[3],
            [
                "번호",
                "이름",
                "회원번호",
                "생년월일",
                "성별",
                "당첨 강좌",
                "대기 강좌(순번)"
            ]
        );
        assert_eq!(
            rows[6],
            [
                "3",
                "홍길동",
                "00012",
                "1950년",
                "남",
                "당첨 | 요가 초급",
                "-"
            ]
        );
    }
}
