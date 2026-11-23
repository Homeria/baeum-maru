//! 결과 명단(BR-P1~P11)을 Excel 파일로 만든다.
//!
//! 무엇을 넣고 가릴지는 [`RosterSpec`]이 정한다. 여기서는 그대로 칸을 채우고 모양만 정한다.
//! 회원번호와 연락처는 앞자리 0이 사라지지 않게 늘 글자로 쓴다(실험 02).

#![cfg_attr(not(test), expect(dead_code, reason = "명단 내보내기 API를 붙이기 전"))]

pub mod raw;

use baeum_domain::{
    Gender, MaskChar, RosterColumn, RosterSpec, format_phone_number, mask_member_no, mask_name,
    mask_phone_number, masked_birth_date,
};
use rust_xlsxwriter::{Format, Worksheet, XlsxError};

use crate::db::rosters::{RosterCourse, RosterEntry, RosterResult};

#[cfg(test)]
mod test_support;

/// 모든 칸의 글꼴. Excel 기본 글꼴(Calibri)에는 한글이 없어 프로그램에 따라 빈칸이 된다.
/// 한국어 Windows 기본 글꼴을 직접 지정한다(실험 02).
const FONT: &str = "맑은 고딕";

/// 칸 하나에 쓸 값.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Cell {
    Text(String),
    Number(u32),
    Empty,
}

impl Cell {
    fn write(
        &self,
        sheet: &mut Worksheet,
        row: u32,
        col: u16,
        format: &Format,
    ) -> Result<(), XlsxError> {
        match self {
            Cell::Text(text) => sheet.write_string_with_format(row, col, text, format)?,
            Cell::Number(number) => sheet.write_number_with_format(row, col, *number, format)?,
            Cell::Empty => sheet.write_blank(row, col, format)?,
        };
        Ok(())
    }
}

/// `entry`의 `column` 칸 값. 가림은 `spec`을 따른다(BR-P7).
fn cell(
    spec: &RosterSpec,
    course: &RosterCourse,
    entry: &RosterEntry,
    column: RosterColumn,
) -> Cell {
    let masked = spec.is_masked(column);
    let mask = spec.mask_char();
    let member = &entry.member;
    match column {
        RosterColumn::CourseName => Cell::Text(course.name.clone()),
        RosterColumn::Capacity => Cell::Number(u32::from(course.capacity.value())),
        RosterColumn::Result => Cell::Text(result_label(entry.result).to_owned()),
        RosterColumn::WaitlistNumber => entry
            .waitlist_number
            .and_then(|number| u32::try_from(number).ok())
            .map_or(Cell::Empty, Cell::Number),
        RosterColumn::Name => {
            Cell::Text(masked_text(member.name.expose(), masked, mask, mask_name))
        }
        RosterColumn::MemberNo => Cell::Text(masked_text(
            member.member_no.expose(),
            masked,
            mask,
            mask_member_no,
        )),
        RosterColumn::PhoneNumber => {
            let digits = member.phone_number.expose();
            Cell::Text(if masked {
                mask_phone_number(digits, mask)
            } else {
                format_phone_number(digits)
            })
        }
        RosterColumn::BirthDate => match &member.birth_date {
            None => Cell::Empty,
            Some(date) if masked => Cell::Text(masked_birth_date(date.expose().year())),
            Some(date) => Cell::Text(date.expose().to_string()),
        },
        RosterColumn::Gender => match &member.gender {
            None => Cell::Empty,
            Some(gender) => Cell::Text(gender_label(*gender.expose()).to_owned()),
        },
        RosterColumn::Guaranteed => {
            if entry.guaranteed {
                Cell::Text("우선".to_owned())
            } else {
                Cell::Empty
            }
        }
    }
}

fn masked_text(
    text: &str,
    masked: bool,
    mask: MaskChar,
    hide: fn(&str, MaskChar) -> String,
) -> String {
    if masked {
        hide(text, mask)
    } else {
        text.to_owned()
    }
}

fn result_label(result: RosterResult) -> &'static str {
    match result {
        RosterResult::Selected => "당첨",
        RosterResult::Waitlisted => "대기",
        RosterResult::Withdrawn => "포기",
    }
}

fn gender_label(gender: Gender) -> &'static str {
    match gender {
        Gender::Male => "남",
        Gender::Female => "여",
    }
}

/// 칸 너비(글자 수쯤). 한글은 영문보다 넓어 넉넉히 둔다.
fn column_width(column: RosterColumn) -> f64 {
    match column {
        RosterColumn::CourseName => 24.0,
        RosterColumn::PhoneNumber => 16.0,
        RosterColumn::Name | RosterColumn::MemberNo | RosterColumn::BirthDate => 13.0,
        RosterColumn::WaitlistNumber => 10.0,
        RosterColumn::Capacity
        | RosterColumn::Result
        | RosterColumn::Gender
        | RosterColumn::Guaranteed => 8.0,
    }
}

/// A4, 가로 폭을 한 쪽에 맞춰 인쇄한다(실험 02).
fn print_on_a4(sheet: &mut Worksheet) {
    sheet.set_paper_size(9); // 9 = A4
    sheet.set_print_fit_to_pages(1, 0); // 가로 1쪽, 세로는 필요한 만큼
    sheet.set_print_center_horizontally(true);
}
