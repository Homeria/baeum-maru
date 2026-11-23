//! 원본 명단(BR-P2): 강좌마다 시트 하나. 왼쪽에 당첨자, 오른쪽에 대기자를 나란히 둔다.
//!
//! 두 묶음은 서로 다른 사람이라 각자 둘째 줄부터 채운다. 묶음마다 바깥을 굵은 선으로 둘러
//! 구분한다. 포기한 신청은 넣지 않는다. 자리 보장으로 당첨된 사람의 당첨자 칸은 색칠한다(BR-P4).

use baeum_domain::{RosterColumn, RosterKind, RosterSpec};
use rust_xlsxwriter::{Format, FormatAlign, FormatBorder, Workbook, Worksheet, XlsxError};

use super::{Cell, FONT, PRIORITY_FILL, cell, column_width, print_on_a4, sheet_names};
use crate::db::rosters::{RosterCourse, RosterEntry, RosterResult};

/// 원본 명단의 칸 모양. 사무용이라 보통 크기 글씨, 가운데 정렬.
struct RawFormats {
    header: Format,
    body: Format,
    priority: Format,
}

impl RawFormats {
    fn new() -> Self {
        let body = Format::new()
            .set_font_name(FONT)
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_border(FormatBorder::Thin);
        RawFormats {
            header: body.clone().set_bold().set_background_color("#E8E8E8"),
            priority: body.clone().set_background_color(PRIORITY_FILL),
            body,
        }
    }
}

/// 당첨자 묶음이나 대기자 묶음 하나.
struct Group<'a> {
    /// 첫 칸 머리글. 예: `당첨자`
    title: &'static str,
    first_col: u16,
    /// 이름 다음의 칸들. 대기자 묶음은 끝에 대기 순번이 붙는다.
    columns: Vec<RosterColumn>,
    entries: Vec<&'a RosterEntry>,
}

/// 묶음 바깥 테두리 위치. 이 칸이 묶음의 어느 가장자리에 있는지.
#[derive(Clone, Copy)]
struct Edges {
    left: bool,
    right: bool,
    top: bool,
    bottom: bool,
}

/// 원본 명단 Excel 파일의 내용을 만든다. 첫 줄은 머리글이고 고정한다.
pub fn raw_roster(spec: &RosterSpec, roster: &[RosterCourse]) -> Result<Vec<u8>, XlsxError> {
    debug_assert_eq!(spec.kind(), RosterKind::Raw);
    let formats = RawFormats::new();
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

/// 칸의 머리글. 이름 칸은 묶음 이름(`당첨자`, `대기자`), 나머지는 칸 이름 그대로.
fn header_label(group: &Group<'_>, column: RosterColumn) -> &'static str {
    if column == RosterColumn::Name {
        group.title
    } else {
        column.label()
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

/// 묶음 바깥에 닿은 쪽만 굵은 선으로 바꾼 서식.
fn with_edges(base: &Format, edges: Edges) -> Format {
    let mut format = base.clone();
    if edges.left {
        format = format.set_border_left(FormatBorder::Medium);
    }
    if edges.right {
        format = format.set_border_right(FormatBorder::Medium);
    }
    if edges.top {
        format = format.set_border_top(FormatBorder::Medium);
    }
    if edges.bottom {
        format = format.set_border_bottom(FormatBorder::Medium);
    }
    format
}

fn write_course(
    sheet: &mut Worksheet,
    spec: &RosterSpec,
    people: &[RosterColumn],
    course: &RosterCourse,
    formats: &RawFormats,
) -> Result<(), XlsxError> {
    let of = |result: RosterResult| -> Vec<&RosterEntry> {
        course
            .entries
            .iter()
            .filter(|entry| entry.result == result)
            .collect()
    };
    let width = u16::try_from(people.len()).unwrap_or(u16::MAX);
    let mut waitlist_columns = people.to_vec();
    waitlist_columns.push(RosterColumn::WaitlistNumber);
    let groups = [
        Group {
            title: "당첨자",
            first_col: 0,
            columns: people.to_vec(),
            entries: of(RosterResult::Selected),
        },
        Group {
            title: "대기자",
            first_col: width,
            columns: waitlist_columns,
            entries: of(RosterResult::Waitlisted),
        },
    ];
    // 두 묶음의 높이를 맞춰 굵은 테두리가 같은 줄에서 닫히게
    let last_row = u32::try_from(
        groups
            .iter()
            .map(|group| group.entries.len())
            .max()
            .unwrap_or(0),
    )
    .unwrap_or(u32::MAX);

    for group in &groups {
        let last = group.columns.len().saturating_sub(1);
        for (index, (col, column)) in (group.first_col..).zip(&group.columns).enumerate() {
            let label = header_label(group, *column);
            let edges = |row: u32| Edges {
                left: index == 0,
                right: index == last,
                top: row == 0,
                bottom: row == 0 || row == last_row,
            };
            sheet.set_column_width(col, column_width(*column).max(header_width(label)))?;
            sheet.write_string_with_format(
                0,
                col,
                label,
                &with_edges(&formats.header, edges(0)),
            )?;
            for row in 1..=last_row {
                let entry = usize::try_from(row - 1)
                    .ok()
                    .and_then(|nth| group.entries.get(nth));
                let base = match entry {
                    Some(entry) if entry.guaranteed => &formats.priority,
                    _ => &formats.body,
                };
                let value = entry.map_or(Cell::Empty, |entry| cell(spec, course, entry, *column));
                value.write(sheet, row, col, &with_edges(base, edges(row)))?;
            }
        }
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
                ["당첨자", "회원번호", "대기자", "회원번호", "대기 순번"],
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
