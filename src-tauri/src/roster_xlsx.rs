//! 결과 명단(BR-P1~P11)을 Excel 파일로 만든다.
//!
//! 무엇을 넣고 가릴지는 [`RosterSpec`]이 정한다. 여기서는 그대로 칸을 채우고 모양만 정한다.
//! 회원번호와 연락처는 앞자리 0이 사라지지 않게 늘 글자로 쓴다(실험 02).

mod posted_by_course;
mod posted_by_member;
mod raw;
mod side_by_side;

use baeum_domain::{
    Gender, MaskChar, RosterColumn, RosterKind, RosterSpec, format_phone_number, mask_member_no,
    mask_name, mask_phone_number, masked_birth_date,
};
use rust_xlsxwriter::{Format, FormatAlign, FormatBorder, Worksheet, XlsxError};

use crate::db::rosters::{RosterCourse, RosterEntry, RosterResult};

#[cfg(test)]
pub mod test_support;

/// 명단 Excel 파일의 내용을 만든다. 종류는 `spec`을 따른다(BR-P1).
/// `term_name`은 회원별 명단의 제목에 쓴다. 예: `2027년 1학기`
///
/// # Errors
///
/// Excel 파일을 만들지 못하면 [`XlsxError`].
pub fn build(
    spec: &RosterSpec,
    roster: &[RosterCourse],
    term_name: &str,
) -> Result<Vec<u8>, XlsxError> {
    match spec.kind() {
        RosterKind::Raw => raw::raw_roster(spec, roster),
        RosterKind::PostedByCourse => posted_by_course::posted_by_course(spec, roster),
        RosterKind::PostedByMember => posted_by_member::posted_by_member(spec, roster, term_name),
    }
}

/// Excel 시트 이름의 최대 글자 수.
const SHEET_NAME_MAX: usize = 31;

/// 모든 칸의 글꼴. Excel 기본 글꼴(Calibri)에는 한글이 없어 프로그램에 따라 빈칸이 된다.
/// 한국어 Windows 기본 글꼴을 직접 지정한다(실험 02).
const FONT: &str = "맑은 고딕";

// 색은 여기에만 둔다. 나중에 테마를 고를 수 있게 되면 이 값들을 바꾼다.

/// 머리글 바탕색.
const HEADER_FILL: &str = "#E8E8E8";
/// 자리 보장 당첨을 칠하는 색(BR-P4). 흑백으로 인쇄해도 회색으로 구별되게 진하게 둔다.
const PRIORITY_FILL: &str = "#FFD966";
/// 칸을 칠할 수 없는 곳(회원별 명단의 여러 줄 칸)에서 자리 보장 당첨 줄의 글자색(BR-P4).
const PRIORITY_TEXT: &str = "#C55A11";

/// 표의 머리글, 내용, 자리 보장 칸 모양. 가운데 정렬, 가는 테두리.
struct TableFormats {
    header: Format,
    body: Format,
    /// 자리 보장으로 당첨된 칸(BR-P4).
    priority: Format,
}

impl TableFormats {
    fn new(font_size: f64) -> Self {
        let body = Format::new()
            .set_font_name(FONT)
            .set_font_size(font_size)
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_border(FormatBorder::Thin);
        TableFormats {
            header: body.clone().set_bold().set_background_color(HEADER_FILL),
            priority: body.clone().set_background_color(PRIORITY_FILL),
            body,
        }
    }
}

/// 게시용 명단 맨 위의 제목과 안내 줄 모양.
struct PostedHeading {
    title: Format,
    note: Format,
}

impl PostedHeading {
    fn new() -> Self {
        let base = Format::new()
            .set_font_name(FONT)
            .set_align(FormatAlign::VerticalCenter);
        PostedHeading {
            title: base
                .clone()
                .set_bold()
                .set_font_size(20)
                .set_align(FormatAlign::Center),
            note: base.set_font_size(12).set_align(FormatAlign::Left),
        }
    }

    /// 0번 줄에 제목, 그 아래 줄마다 안내를 표 너비(`width`칸)만큼 합쳐 쓴다.
    fn write(
        &self,
        sheet: &mut Worksheet,
        width: u16,
        title: &str,
        notes: &[&str],
    ) -> Result<(), XlsxError> {
        let lines = std::iter::once((title, &self.title, 36.0))
            .chain(notes.iter().map(|note| (*note, &self.note, 22.0)));
        for (row, (text, format, height)) in (0u32..).zip(lines) {
            if width <= 1 {
                sheet.write_string_with_format(row, 0, text, format)?;
            } else {
                sheet.merge_range(row, 0, row, width - 1, text, format)?;
            }
            sheet.set_row_height(row, height)?;
        }
        Ok(())
    }
}

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

/// 강좌마다 시트 이름 `강좌 이름(정원)`.
///
/// Excel 규칙에 맞춘다: 31자 이하, `: \ / ? * [ ]`는 쓸 수 없다. 길면 강좌 이름을 자른다.
/// 겹치면(대소문자 무시) 이름 뒤에 `-2`, `-3`을 붙인다.
fn sheet_names(roster: &[RosterCourse]) -> Vec<String> {
    let mut taken: Vec<String> = Vec::with_capacity(roster.len());
    let mut names = Vec::with_capacity(roster.len());
    for course in roster {
        let base: String = course
            .name
            .chars()
            .filter(|c| !matches!(c, ':' | '\\' | '/' | '?' | '*' | '[' | ']'))
            .collect();
        let base = base.trim_matches(|c: char| c == '\'' || c.is_whitespace());
        let capacity = format!("({})", course.capacity.value());
        let name = (1..)
            .map(|n| {
                let suffix = if n == 1 {
                    capacity.clone()
                } else {
                    format!("-{n}{capacity}")
                };
                let room = SHEET_NAME_MAX.saturating_sub(suffix.chars().count());
                let head: String = base.chars().take(room).collect();
                format!("{}{suffix}", head.trim_end())
            })
            .find(|candidate| !taken.contains(&candidate.to_lowercase()))
            .unwrap_or_default();
        taken.push(name.to_lowercase());
        names.push(name);
    }
    names
}

/// 글자가 잘리지 않는 칸 너비. 한글 한 글자가 영문 두 글자쯤 된다.
fn text_width(text: &str) -> f64 {
    let wide: u32 = text.chars().map(|c| if c.is_ascii() { 1 } else { 2 }).sum();
    f64::from(wide) + 2.0
}

/// A4, 가로 폭을 한 쪽에 맞춰 인쇄한다(실험 02).
fn print_on_a4(sheet: &mut Worksheet) {
    sheet.set_paper_size(9); // 9 = A4
    sheet.set_print_fit_to_pages(1, 0); // 가로 1쪽, 세로는 필요한 만큼
    sheet.set_print_center_horizontally(true);
}

#[cfg(test)]
mod tests {
    use baeum_domain::{Capacity, MaskChar, RosterChoice};

    use super::*;
    use test_support::{read_sheet, sample_roster};

    #[test]
    fn br_p1_builds_each_kind() {
        // 첫 시트의 첫 칸으로 종류를 알아본다
        let first_cell = |kind: RosterKind| {
            let spec = RosterSpec::new(RosterChoice {
                kind,
                include: &[],
                mask: &[],
                mask_char: MaskChar::Circle,
            })
            .unwrap();
            read_sheet(&build(&spec, &sample_roster(), "2027년 1학기").unwrap(), 0).1[0][0].clone()
        };

        assert_eq!(first_cell(RosterKind::Raw), "당첨자");
        assert_eq!(
            first_cell(RosterKind::PostedByCourse),
            "요가 초급 (정원 2명)"
        );
        assert_eq!(
            first_cell(RosterKind::PostedByMember),
            "2027년 1학기 회원별 선발 결과"
        );
    }

    #[test]
    fn sheet_names_follow_excel_rules() {
        let course = |name: &str| RosterCourse {
            course_id: 1,
            name: name.to_owned(),
            capacity: Capacity::new(20).unwrap(),
            entries: Vec::new(),
        };
        let roster = [
            course("스마트폰: 카톡/사진 [기초]"),
            course("요가"),
            course("요가"),
            course("아주 긴 강좌 이름이 들어가는 경우를 위한 시험용 강좌 이름 초급 1반"),
        ];

        let names = sheet_names(&roster);

        assert_eq!(names[0], "스마트폰 카톡사진 기초(20)");
        assert_eq!(names[1], "요가(20)");
        assert_eq!(names[2], "요가-2(20)");
        assert_eq!(names[3].chars().count(), 31);
        assert!(names[3].ends_with("(20)"));
    }
}
