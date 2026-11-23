//! 게시용 회원별 명단(BR-P3, BR-P4): 회원 한 명이 한 줄. 회원이 자기 이름 하나만 찾으면
//! 모든 결과를 본다.
//!
//! 줄은 실제 이름의 가나다순이다(가려서 보여 줘도 찾기 쉽게). 결과는 강좌마다 칸을 따로 두고,
//! 자리 보장으로 당첨된 칸만 색칠한다. 포기한 신청은 넣지 않고, 포기만 남은 회원도 넣지 않는다.

use std::collections::HashMap;

use baeum_domain::{RosterColumn, RosterSpec};
use rust_xlsxwriter::{Workbook, XlsxError};

use super::{PostedFormats, cell, posted_column_width, posted_table_columns, print_on_a4};
use crate::db::rosters::{RosterCourse, RosterEntry, RosterResult};

/// 회원별 명단 시트 이름.
const SHEET_NAME: &str = "회원별 결과";
/// 결과 칸 너비.
const RESULT_WIDTH: f64 = 30.0;

/// 회원 한 명과 그 회원의 신청들(강좌 순).
struct MemberRow<'a> {
    results: Vec<(&'a RosterCourse, &'a RosterEntry)>,
}

impl MemberRow<'_> {
    /// 회원 정보 칸을 채울 때 쓰는 첫 신청.
    fn first(&self) -> (&RosterCourse, &RosterEntry) {
        self.results[0]
    }
}

/// 게시용 회원별 명단 Excel 파일의 내용을 만든다.
pub fn posted_by_member(spec: &RosterSpec, roster: &[RosterCourse]) -> Result<Vec<u8>, XlsxError> {
    let formats = PostedFormats::new();
    // 강좌 이름과 정원은 결과 칸 글자에, 결과·대기 순번도 결과 칸에 들어간다
    let member_columns: Vec<RosterColumn> = posted_table_columns(spec)
        .into_iter()
        .filter(|column| !matches!(column, RosterColumn::Result | RosterColumn::WaitlistNumber))
        .collect();
    let rows = member_rows(roster);
    let result_slots = rows.iter().map(|row| row.results.len()).max().unwrap_or(0);
    let member_count = u16::try_from(member_columns.len()).unwrap_or(u16::MAX);
    let last_col =
        (member_count + u16::try_from(result_slots).unwrap_or(u16::MAX)).saturating_sub(1);

    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();
    sheet.set_name(SHEET_NAME)?;
    sheet.merge_range(
        0,
        0,
        0,
        last_col.max(1),
        "선발 결과 (회원별)",
        &formats.title,
    )?;
    sheet.set_row_height(0, 36)?;
    for (col, column) in (0u16..).zip(&member_columns) {
        sheet.write_string_with_format(2, col, column.label(), &formats.header)?;
        sheet.set_column_width(col, posted_column_width(*column))?;
    }
    for (col, nth) in (member_count..).zip(1..=result_slots) {
        sheet.write_string_with_format(2, col, format!("결과 {nth}"), &formats.header)?;
        sheet.set_column_width(col, RESULT_WIDTH)?;
    }

    for (row, member) in (3u32..).zip(&rows) {
        let (course, entry) = member.first();
        for (col, column) in (0u16..).zip(&member_columns) {
            cell(spec, course, entry, *column).write(sheet, row, col, &formats.body)?;
        }
        for (col, nth) in (member_count..).zip(0..result_slots) {
            match member.results.get(nth) {
                Some((course, entry)) => {
                    let format = if entry.guaranteed {
                        &formats.priority
                    } else {
                        &formats.body
                    };
                    sheet.write_string_with_format(row, col, result_text(course, entry), format)?;
                }
                None => {
                    sheet.write_blank(row, col, &formats.body)?;
                }
            }
        }
        sheet.set_row_height(row, 24)?;
    }
    sheet.set_repeat_rows(2, 2)?;
    print_on_a4(sheet);
    workbook.save_to_buffer()
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

/// 결과 칸 글자. 예: `요가 초급(20) 당첨`, `서예(15) 대기 2번`
fn result_text(course: &RosterCourse, entry: &RosterEntry) -> String {
    let result = match (entry.result, entry.waitlist_number) {
        (RosterResult::Waitlisted, Some(number)) => format!("대기 {number}번"),
        (RosterResult::Waitlisted, None) => "대기".to_owned(),
        (RosterResult::Selected, _) => "당첨".to_owned(),
        (RosterResult::Withdrawn, _) => "포기".to_owned(),
    };
    format!("{}({}) {result}", course.name, course.capacity.value())
}

#[cfg(test)]
mod tests {
    use baeum_domain::{MaskChar, RosterChoice, RosterKind};

    use super::*;
    use crate::roster_xlsx::test_support::{read_sheet, sample_roster};

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
    fn br_p3_one_row_per_member_in_name_order() {
        let mut roster = sample_roster();
        // 홍길동이 서예에도 대기 2번으로 있다
        let mut again = roster[0].entries[0].clone();
        again.result = RosterResult::Waitlisted;
        again.waitlist_number = Some(2);
        again.guaranteed = false;
        roster[1].entries.push(again);

        let bytes = posted_by_member(&spec(&[], &[RosterColumn::Name]), &roster).unwrap();

        let (name, rows) = read_sheet(&bytes, 0);
        assert_eq!(name, "회원별 결과");
        assert_eq!(rows[0][0], "선발 결과 (회원별)");
        assert_eq!(
            rows[2..],
            [
                ["이름", "회원번호", "결과 1", "결과 2"],
                ["김○", "00034", "요가 초급(2) 대기 1번", ""],
                ["박○희", "00078", "서예(1) 당첨", ""],
                ["홍○동", "00012", "요가 초급(2) 당첨", "서예(1) 대기 2번"],
            ]
        );
    }

    #[test]
    fn br_p5_member_details_follow_name_and_are_masked() {
        let bytes = posted_by_member(
            &spec(&[RosterColumn::BirthDate, RosterColumn::Gender], &[]),
            &sample_roster(),
        )
        .unwrap();

        let (_, rows) = read_sheet(&bytes, 0);
        assert_eq!(rows[2], ["이름", "회원번호", "생년월일", "성별", "결과 1"]);
        assert_eq!(
            rows[5],
            ["홍길동", "00012", "1950년", "남", "요가 초급(2) 당첨"]
        );
    }
}
