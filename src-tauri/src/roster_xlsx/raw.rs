//! 원본 명단(BR-P2): 강좌마다 시트 하나. 왼쪽에 당첨자, 오른쪽에 대기자를 나란히 둔다.
//!
//! 두 묶음은 서로 다른 사람이라 각자 둘째 줄부터 채운다. 포기한 신청은 넣지 않는다.
//! 자리 보장으로 당첨된 사람의 당첨자 칸은 색칠한다(BR-P4).

use baeum_domain::{RosterColumn, RosterKind, RosterSpec};
use rust_xlsxwriter::{Format, FormatBorder, Workbook, Worksheet, XlsxError};

use super::{FONT, PRIORITY_FILL, cell, column_width, print_on_a4, sheet_names};
use crate::db::rosters::{RosterCourse, RosterEntry, RosterResult};

/// 원본 명단의 칸 모양. 사무용이라 보통 크기 글씨.
struct RawFormats {
    header: Format,
    body: Format,
    priority: Format,
}

/// 원본 명단 Excel 파일의 내용을 만든다. 첫 줄은 머리글이고 고정한다.
pub fn raw_roster(spec: &RosterSpec, roster: &[RosterCourse]) -> Result<Vec<u8>, XlsxError> {
    debug_assert_eq!(spec.kind(), RosterKind::Raw);
    let body = Format::new()
        .set_font_name(FONT)
        .set_border(FormatBorder::Thin);
    let formats = RawFormats {
        header: body.clone().set_bold().set_background_color("#E8E8E8"),
        priority: body.clone().set_background_color(PRIORITY_FILL),
        body,
    };
    let people = person_columns(spec);
    let names = sheet_names(roster);

    let mut workbook = Workbook::new();
    for (course, name) in roster.iter().zip(names) {
        let sheet = workbook.add_worksheet();
        sheet.set_name(name)?;
        write_course(sheet, spec, &people, course, &formats)?;
        sheet.set_freeze_panes(1, 0)?;
        sheet.set_repeat_rows(0, 0)?;
        print_on_a4(sheet);
    }
    workbook.save_to_buffer()
}

/// 당첨자·대기자 묶음마다 넣는 칸: 이름, 회원번호, 고른 칸(연락처, 생년월일, 성별).
fn person_columns(spec: &RosterSpec) -> Vec<RosterColumn> {
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

/// 묶음의 머리글. 이름 칸은 묶음 이름 그대로, 나머지는 앞에 붙인다. 예: `대기자 회원번호`
fn group_label(group: &str, column: RosterColumn) -> String {
    if column == RosterColumn::Name {
        group.to_owned()
    } else {
        format!("{group} {}", column.label())
    }
}

/// 머리글이 잘리지 않는 칸 너비. 한글 한 글자가 영문 두 글자쯤 된다.
fn header_width(label: &str) -> f64 {
    let wide: u32 = label
        .chars()
        .map(|c| if c.is_ascii() { 1 } else { 2 })
        .sum();
    f64::from(wide) + 2.0
}

fn write_course(
    sheet: &mut Worksheet,
    spec: &RosterSpec,
    people: &[RosterColumn],
    course: &RosterCourse,
    formats: &RawFormats,
) -> Result<(), XlsxError> {
    let width = u16::try_from(people.len()).unwrap_or(u16::MAX);
    let waitlist_col = width.saturating_mul(2);
    for (offset, group) in [(0, "당첨자"), (width, "대기자")] {
        for (col, column) in (offset..).zip(people) {
            let label = group_label(group, *column);
            sheet.set_column_width(col, column_width(*column).max(header_width(&label)))?;
            sheet.write_string_with_format(0, col, label, &formats.header)?;
        }
    }
    sheet.write_string_with_format(
        0,
        waitlist_col,
        RosterColumn::WaitlistNumber.label(),
        &formats.header,
    )?;
    sheet.set_column_width(waitlist_col, column_width(RosterColumn::WaitlistNumber))?;

    let of = |result: RosterResult| {
        course
            .entries
            .iter()
            .filter(move |entry| entry.result == result)
    };
    for (row, entry) in (1u32..).zip(of(RosterResult::Selected)) {
        let format = if entry.guaranteed {
            &formats.priority
        } else {
            &formats.body
        };
        write_person(sheet, spec, people, course, entry, row, 0, format)?;
    }
    for (row, entry) in (1u32..).zip(of(RosterResult::Waitlisted)) {
        write_person(
            sheet,
            spec,
            people,
            course,
            entry,
            row,
            width,
            &formats.body,
        )?;
        cell(spec, course, entry, RosterColumn::WaitlistNumber).write(
            sheet,
            row,
            waitlist_col,
            &formats.body,
        )?;
    }
    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "칸 하나를 쓰는 데 필요한 것을 그대로 받는다"
)]
fn write_person(
    sheet: &mut Worksheet,
    spec: &RosterSpec,
    people: &[RosterColumn],
    course: &RosterCourse,
    entry: &RosterEntry,
    row: u32,
    first_col: u16,
    format: &Format,
) -> Result<(), XlsxError> {
    for (col, column) in (first_col..).zip(people) {
        cell(spec, course, entry, *column).write(sheet, row, col, format)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use baeum_domain::{MaskChar, RosterChoice};

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
        // 포기한 남궁민수는 없다
        assert_eq!(
            rows,
            [
                [
                    "당첨자",
                    "당첨자 회원번호",
                    "대기자",
                    "대기자 회원번호",
                    "대기 순번"
                ],
                ["홍길동", "00012", "김철", "00034", "1"],
            ]
        );
        let (second, rows) = read_sheet(&bytes, 1);
        assert_eq!(second, "서예(1)");
        assert_eq!(rows[1], ["박영희", "00078", "김철", "00034", "1"]);
        assert_eq!(rows[2], ["", "", "김철", "00034", "2"]);
    }

    #[test]
    fn br_p6_p7_chosen_columns_go_in_each_group_masked_or_not() {
        use RosterColumn::*;
        let chosen = spec(&[PhoneNumber, BirthDate, Gender], &[Name, BirthDate]);

        let bytes = raw_roster(&chosen, &sample_roster()).unwrap();

        let (_, rows) = read_sheet(&bytes, 0);
        assert_eq!(
            rows,
            [
                [
                    "당첨자",
                    "당첨자 회원번호",
                    "당첨자 연락처",
                    "당첨자 생년월일",
                    "당첨자 성별",
                    "대기자",
                    "대기자 회원번호",
                    "대기자 연락처",
                    "대기자 생년월일",
                    "대기자 성별",
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
