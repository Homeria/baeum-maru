//! 당첨자와 대기자를 나란히 두는 표(BR-P2, BR-P3). 원본 명단과 게시용 강좌별이 같이 쓴다.
//!
//! 왼쪽은 당첨자, 오른쪽은 대기자 묶음이고 각자 머리글 다음 줄부터 채운다. 묶음마다 바깥을
//! 굵은 선으로 둘러 구분한다. 자리 보장으로 당첨된 사람의 당첨자 칸은 색칠한다(BR-P4).

use baeum_domain::{RosterColumn, RosterSpec};
use rust_xlsxwriter::{Format, FormatBorder, Worksheet, XlsxError};

use super::{Cell, TableFormats, cell, column_width};
use crate::db::rosters::{RosterCourse, RosterEntry, RosterResult};

/// 당첨자 묶음이나 대기자 묶음 하나.
struct Group<'a> {
    /// 첫 칸 머리글. 예: `당첨자`
    title: &'static str,
    first_col: u16,
    /// 이름부터의 칸들. 대기자 묶음은 끝에 대기 순번이 붙는다.
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

/// 표의 칸 수. 제목을 표 너비만큼 합칠 때 쓴다.
pub(super) fn table_width(spec: &RosterSpec) -> u16 {
    let people = u16::try_from(person_columns(spec).len()).unwrap_or(u16::MAX);
    // 두 묶음 + 대기 순번
    people.saturating_mul(2).saturating_add(1)
}

/// `first_row` 줄에 머리글, 그 아래에 당첨자와 대기자를 쓴다. 칸 너비는 `width_scale`배.
pub(super) fn write_side_by_side(
    sheet: &mut Worksheet,
    spec: &RosterSpec,
    course: &RosterCourse,
    formats: &TableFormats,
    first_row: u32,
    width_scale: f64,
) -> Result<(), XlsxError> {
    let people = person_columns(spec);
    let of = |result: RosterResult| -> Vec<&RosterEntry> {
        course
            .entries
            .iter()
            .filter(|entry| entry.result == result)
            .collect()
    };
    let width = u16::try_from(people.len()).unwrap_or(u16::MAX);
    let mut waitlist_columns = people.clone();
    waitlist_columns.push(RosterColumn::WaitlistNumber);
    let groups = [
        Group {
            title: "당첨자",
            first_col: 0,
            columns: people,
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
    let tallest = groups
        .iter()
        .map(|group| group.entries.len())
        .max()
        .unwrap_or(0);
    let last_row = first_row.saturating_add(u32::try_from(tallest).unwrap_or(u32::MAX));

    for group in &groups {
        let last = group.columns.len().saturating_sub(1);
        for (index, (col, column)) in (group.first_col..).zip(&group.columns).enumerate() {
            let label = header_label(group, *column);
            let edges = |row: u32| Edges {
                left: index == 0,
                right: index == last,
                top: row == first_row,
                bottom: row == first_row || row == last_row,
            };
            let col_width = column_width(*column).max(header_width(label));
            sheet.set_column_width(col, col_width * width_scale)?;
            sheet.write_string_with_format(
                first_row,
                col,
                label,
                &with_edges(&formats.header, edges(first_row)),
            )?;
            for (nth, row) in (first_row + 1..=last_row).enumerate() {
                let entry = group.entries.get(nth);
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
