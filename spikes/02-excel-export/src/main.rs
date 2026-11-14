//! 버린 코드: Excel 출력·가져오기 실험 (3단계 기술 실험 02)
//!
//! 1. 게시용 결과 명단 (이름 가림, 회원번호, 합격/대기 순번)
//! 2. 출석부 (강사가 종이에 체크하는 양식)
//! 3. 회원 명단 가져오기 (기관이 쓰던 엑셀을 읽을 때 생기는 함정 확인)
//!
//! 이름과 연락처는 모두 지어낸 가짜 데이터다.

use std::error::Error;

use calamine::{Data, Reader, Xlsx, open_workbook};
use chrono::NaiveDate;
use rust_xlsxwriter::{ExcelDateTime, Format, FormatAlign, FormatBorder, Workbook, Worksheet};

const RESULT_FILE: &str = "결과명단.xlsx";
const ATTENDANCE_FILE: &str = "출석부.xlsx";
const IMPORT_FILE: &str = "회원명단_가져오기.xlsx";

/// 가짜 신청자 한 명.
struct Applicant {
    member_no: String,
    name: String,
}

/// 가짜 강좌 하나와 그 신청자들 (앞에서부터 정원만큼 합격, 나머지는 대기).
struct Course {
    title: String,
    capacity: usize,
    applicants: Vec<Applicant>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let courses = fake_courses();

    write_result_roster(&courses)?;
    println!("만듦: {RESULT_FILE}");

    write_attendance_sheet(&courses)?;
    println!("만듦: {ATTENDANCE_FILE}");

    write_import_sample()?;
    println!("만듦: {IMPORT_FILE} (가져오기 시험용)");
    println!();

    read_import_sample()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// 가짜 데이터
// ---------------------------------------------------------------------------

fn fake_courses() -> Vec<Course> {
    let surnames = ['김', '이', '박', '최', '정', '강', '조', '윤', '장', '임'];
    let given = ["가람", "나래", "다솜", "라온", "마루", "바다", "사랑", "아름", "자람", "하늘"];

    let mut next_no = 100;
    let mut make = |count: usize| -> Vec<Applicant> {
        (0..count)
            .map(|i| {
                next_no += 1;
                Applicant {
                    member_no: format!("{next_no:05}"), // 앞자리 0을 살린 회원번호
                    name: format!("{}{}", surnames[(i * 3) % 10], given[(i * 7 + next_no) % 10]),
                }
            })
            .collect()
    };

    let mut courses = vec![
        Course { title: "스마트폰 초급 1반".into(), capacity: 20, applicants: make(25) },
        Course { title: "요가".into(), capacity: 15, applicants: make(12) },
    ];
    // 이름 길이별 가림 확인용: 외자, 네 글자 이름
    courses[1].applicants.push(Applicant { member_no: "00999".into(), name: "남궁하늘".into() });
    courses[1].applicants.push(Applicant { member_no: "01000".into(), name: "윤솔".into() });
    courses
}

/// BR-R1: 이름 가운데 글자를 가린다. 홍길동 → 홍○동, 윤솔 → 윤○, 남궁하늘 → 남○○늘
fn mask_name(name: &str) -> String {
    // 한글은 한 글자가 3바이트라 name.len()은 글자 수가 아니다. 글자 단위로 나눈다.
    let chars: Vec<char> = name.trim().chars().collect();
    match chars.len() {
        0 => String::new(),
        1 => chars[0].to_string(),
        2 => format!("{}○", chars[0]),
        n => {
            let mut masked = String::new();
            masked.push(chars[0]);
            for _ in 1..n - 1 {
                masked.push('○');
            }
            masked.push(chars[n - 1]);
            masked
        }
    }
}

// ---------------------------------------------------------------------------
// 1. 게시용 결과 명단
// ---------------------------------------------------------------------------

fn write_result_roster(courses: &[Course]) -> Result<(), Box<dyn Error>> {
    let mut workbook = Workbook::new();
    let title = base_format().set_bold().set_font_size(16).set_align(FormatAlign::Center);
    let header = base_format()
        .set_bold()
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::Center)
        .set_background_color("#E8E8E8");
    let cell = base_format().set_border(FormatBorder::Thin).set_align(FormatAlign::Center);

    for course in courses {
        let sheet = workbook.add_worksheet();
        sheet.set_name(sheet_name(&course.title))?;
        print_setup(sheet, false)?;

        sheet.merge_range(0, 0, 0, 3, &format!("{} 선발 결과", course.title), &title)?;
        for (col, text) in ["순번", "회원번호", "이름", "결과"].iter().enumerate() {
            sheet.write_with_format(2, col as u16, *text, &header)?;
        }
        // 인쇄할 때 쪽마다 머리글(3번째 줄)을 반복한다.
        sheet.set_repeat_rows(2, 2)?;

        for (i, a) in course.applicants.iter().enumerate() {
            let row = 3 + i as u32;
            let result = if i < course.capacity {
                "합격".to_string()
            } else {
                format!("대기 {}번", i - course.capacity + 1)
            };
            sheet.write_with_format(row, 0, (i + 1) as u32, &cell)?;
            // 회원번호는 숫자가 아니라 글자로 쓴다. 숫자로 쓰면 00101 → 101이 된다.
            sheet.write_with_format(row, 1, a.member_no.as_str(), &cell)?;
            sheet.write_with_format(row, 2, mask_name(&a.name).as_str(), &cell)?;
            sheet.write_with_format(row, 3, result.as_str(), &cell)?;
        }
        sheet.set_column_width(0, 8)?;
        sheet.set_column_width(1, 14)?;
        sheet.set_column_width(2, 14)?;
        sheet.set_column_width(3, 14)?;
    }
    workbook.save(RESULT_FILE)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// 2. 출석부
// ---------------------------------------------------------------------------

fn write_attendance_sheet(courses: &[Course]) -> Result<(), Box<dyn Error>> {
    const SESSIONS: u16 = 15; // 한 학기 수업 횟수
    let mut workbook = Workbook::new();
    let title = base_format().set_bold().set_font_size(14);
    let header = base_format()
        .set_bold()
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::Center)
        .set_background_color("#E8E8E8");
    let cell = base_format().set_border(FormatBorder::Thin).set_align(FormatAlign::Center);

    for course in courses {
        let sheet = workbook.add_worksheet();
        sheet.set_name(sheet_name(&course.title))?;
        print_setup(sheet, true)?;

        let last_col = 2 + SESSIONS;
        sheet.merge_range(0, 0, 0, last_col, &format!("{} 출석부", course.title), &title)?;
        sheet.write_with_format(1, 0, "강사: ____________    강의실: ____________", &base_format())?;

        sheet.write_with_format(3, 0, "번호", &header)?;
        sheet.write_with_format(3, 1, "회원번호", &header)?;
        sheet.write_with_format(3, 2, "이름", &header)?;
        for s in 1..=SESSIONS {
            sheet.write_with_format(3, 2 + s, format!("{s}회"), &header)?;
        }
        sheet.set_repeat_rows(3, 3)?;

        // 출석부는 기관 안에서만 쓰니 이름을 가리지 않는다. 합격자만 넣는다.
        for (i, a) in course.applicants.iter().take(course.capacity).enumerate() {
            let row = 4 + i as u32;
            sheet.set_row_height(row, 20)?;
            sheet.write_with_format(row, 0, (i + 1) as u32, &cell)?;
            sheet.write_with_format(row, 1, a.member_no.as_str(), &cell)?;
            sheet.write_with_format(row, 2, a.name.as_str(), &cell)?;
            for s in 1..=SESSIONS {
                sheet.write_blank(row, 2 + s, &cell)?;
            }
        }
        sheet.set_column_width(0, 5)?;
        sheet.set_column_width(1, 9)?;
        sheet.set_column_width(2, 10)?;
        for s in 1..=SESSIONS {
            sheet.set_column_width(2 + s, 5)?;
        }
    }
    workbook.save(ATTENDANCE_FILE)?;
    Ok(())
}

/// 모든 칸의 기본 글꼴. 엑셀 기본 글꼴(Calibri)에는 한글이 없어서, 프로그램에 따라
/// 한글이 빈칸으로 나올 수 있다. 한국어 Windows 기본 글꼴인 맑은 고딕을 직접 지정한다.
/// 실험용으로 `SPIKE_FONT` 환경 변수로 바꿔 볼 수 있다.
fn base_format() -> Format {
    let font = std::env::var("SPIKE_FONT").unwrap_or_else(|_| "맑은 고딕".to_string());
    Format::new().set_font_name(font)
}

/// A4, 가로 폭을 한 쪽에 맞춰 인쇄.
fn print_setup(sheet: &mut Worksheet, landscape: bool) -> Result<(), Box<dyn Error>> {
    sheet.set_paper_size(9); // 9 = A4
    if landscape {
        sheet.set_landscape();
    }
    sheet.set_print_fit_to_pages(1, 0); // 가로 1쪽, 세로는 필요한 만큼
    sheet.set_print_center_horizontally(true);
    Ok(())
}

/// 시트 이름은 31자 이하, 일부 기호(: \ / ? * [ ])를 쓸 수 없다.
fn sheet_name(title: &str) -> String {
    title
        .chars()
        .filter(|c| !matches!(c, ':' | '\\' | '/' | '?' | '*' | '[' | ']'))
        .take(31)
        .collect()
}

// ---------------------------------------------------------------------------
// 3. 회원 명단 가져오기
// ---------------------------------------------------------------------------

/// 기관이 쓰던 엑셀에서 흔히 보이는 함정을 일부러 넣은 파일을 만든다.
fn write_import_sample() -> Result<(), Box<dyn Error>> {
    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();
    sheet.set_name("회원")?;
    let date = Format::new().set_num_format("yyyy-mm-dd");

    for (col, text) in ["회원번호", "이름", "연락처", "생년월일"].iter().enumerate() {
        sheet.write(0, col as u16, *text)?;
    }

    // 1: 정상 (회원번호 글자, 생년월일은 엑셀 날짜)
    sheet.write(1, 0, "00123")?;
    sheet.write(1, 1, "김가람")?;
    sheet.write(1, 2, "010-1234-5678")?;
    sheet.write_datetime_with_format(1, 3, ExcelDateTime::from_ymd(1950, 3, 4)?, &date)?;
    // 2: 회원번호·연락처가 숫자로 저장됨 (앞자리 0 사라짐), 이름 뒤 공백, 생년월일이 글자
    sheet.write(2, 0, 124)?;
    sheet.write(2, 1, "이나래 ")?;
    sheet.write(2, 2, 1012345678)?;
    sheet.write(2, 3, "1948.11.02")?;
    // 3: 연락처 하이픈 없음, 생년월일 8자리 글자
    sheet.write(3, 0, "00125")?;
    sheet.write(3, 1, "박다솜")?;
    sheet.write(3, 2, "01098765432")?;
    sheet.write(3, 3, "19520715")?;
    // 4: 빈 줄
    // 5: 이름 없음, 없는 날짜
    sheet.write(5, 0, "00126")?;
    sheet.write(5, 2, "010-0000-0000")?;
    sheet.write(5, 3, "1949-02-30")?;
    // 6: 생년월일이 날짜 서식 없는 숫자 (엑셀 내부 날짜 번호)
    sheet.write(6, 0, "00127")?;
    sheet.write(6, 1, "최라온")?;
    sheet.write(6, 2, "010-2222-3333")?;
    sheet.write(6, 3, 18000)?;

    workbook.save(IMPORT_FILE)?;
    Ok(())
}

fn read_import_sample() -> Result<(), Box<dyn Error>> {
    let mut workbook: Xlsx<_> = open_workbook(IMPORT_FILE)?;
    let range = workbook.worksheet_range("회원")?;

    println!("== 가져오기 결과 ==");
    for (i, row) in range.rows().enumerate().skip(1) {
        let line = i + 1; // 엑셀에서 보이는 줄 번호
        if row.iter().all(|c| matches!(c, Data::Empty)) {
            println!("{line}줄: 빈 줄 → 건너뜀");
            continue;
        }

        let mut warnings = Vec::new();
        let member_no = read_member_no(&row[0], &mut warnings);
        let name = cell_text(&row[1]).trim().to_string();
        if name.is_empty() {
            warnings.push("이름 없음".to_string());
        }
        let phone = read_phone(&row[2], &mut warnings);
        let birth = read_birth(&row[3], &mut warnings);

        println!(
            "{line}줄: 회원번호={member_no} 이름={name} 연락처={phone} 생년월일={}",
            birth.map(|d| d.to_string()).unwrap_or_else(|| "?".into())
        );
        for w in warnings {
            println!("      ⚠ {w}");
        }
    }
    Ok(())
}

/// 셀 원본 종류를 사람이 읽을 수 있게.
fn cell_text(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        other => other.to_string(),
    }
}

fn read_member_no(cell: &Data, warnings: &mut Vec<String>) -> String {
    match cell {
        Data::String(s) => s.trim().to_string(),
        Data::Int(n) => {
            warnings.push(format!("회원번호가 숫자로 저장됨({n}). 앞자리 0이 사라졌을 수 있음"));
            n.to_string()
        }
        Data::Float(f) if f.fract() == 0.0 => {
            warnings.push(format!("회원번호가 숫자로 저장됨({f}). 앞자리 0이 사라졌을 수 있음"));
            format!("{}", *f as i64)
        }
        other => {
            warnings.push(format!("회원번호를 읽을 수 없음: {other:?}"));
            String::new()
        }
    }
}

fn read_phone(cell: &Data, warnings: &mut Vec<String>) -> String {
    let raw = match cell {
        Data::String(s) => s.clone(),
        Data::Int(n) => {
            warnings.push("연락처가 숫자로 저장됨. 앞자리 0을 붙여 복원".into());
            format!("0{n}")
        }
        Data::Float(f) if f.fract() == 0.0 => {
            warnings.push("연락처가 숫자로 저장됨. 앞자리 0을 붙여 복원".into());
            format!("0{}", *f as i64)
        }
        other => {
            warnings.push(format!("연락처를 읽을 수 없음: {other:?}"));
            return String::new();
        }
    };
    // 숫자만 남겨서 010-1234-5678 모양으로 맞춘다.
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    match digits.len() {
        11 => format!("{}-{}-{}", &digits[0..3], &digits[3..7], &digits[7..]),
        10 => format!("{}-{}-{}", &digits[0..3], &digits[3..6], &digits[6..]),
        _ => {
            warnings.push(format!("연락처 자릿수가 이상함: {raw}"));
            raw
        }
    }
}

fn read_birth(cell: &Data, warnings: &mut Vec<String>) -> Option<NaiveDate> {
    match cell {
        Data::DateTime(dt) => dt.as_datetime().map(|t| t.date()),
        Data::String(s) => {
            let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
            let parsed = if digits.len() == 8 {
                let y = digits[0..4].parse().ok()?;
                let m = digits[4..6].parse().ok()?;
                let d = digits[6..8].parse().ok()?;
                NaiveDate::from_ymd_opt(y, m, d)
            } else {
                None
            };
            if parsed.is_none() {
                warnings.push(format!("생년월일을 날짜로 읽을 수 없음: {s}"));
            }
            parsed
        }
        Data::Float(_) | Data::Int(_) => {
            warnings.push(format!(
                "생년월일이 날짜 서식 없는 숫자({cell}). 엑셀 날짜 번호로 보고 변환했지만 확인 필요"
            ));
            let serial = match cell {
                Data::Float(f) => *f as i64,
                Data::Int(n) => *n,
                _ => unreachable!(),
            };
            // 엑셀 날짜 번호는 1899-12-30을 0으로 센다.
            NaiveDate::from_ymd_opt(1899, 12, 30)?.checked_add_days(chrono::Days::new(serial as u64))
        }
        Data::Empty => {
            warnings.push("생년월일 없음".into());
            None
        }
        other => {
            warnings.push(format!("생년월일을 읽을 수 없음: {other:?}"));
            None
        }
    }
}
