//! 명단 Excel 테스트 도우미: 지어낸 명단과, 만든 파일을 다시 읽는 함수.

use std::io::Cursor;

use baeum_domain::{Capacity, Gender, Personal};
use calamine::{Reader, Xlsx};
use time::macros::date;

use crate::features::rosters::repository::{RosterCourse, RosterEntry, RosterMember, RosterResult};

fn member(member_id: i64, name: &str, member_no: &str, phone: &str) -> RosterMember {
    RosterMember {
        member_id,
        name: Personal::new(name.to_owned()),
        member_no: Personal::new(member_no.to_owned()),
        phone_number: Personal::new(phone.to_owned()),
        birth_date: None,
        gender: None,
    }
}

/// 강좌 둘. 요가 초급(정원 2): 우선 당첨 1, 대기 1번 1, 포기 1. 서예(정원 1): 당첨 1.
/// 박영희는 요가 대기자 김철과 다른 사람이다. 모두 지어낸 이름이다.
pub fn sample_roster() -> Vec<RosterCourse> {
    let mut first = member(1, "홍길동", "00012", "01012345678");
    first.birth_date = Some(Personal::new(date!(1950 - 03 - 15)));
    first.gender = Some(Personal::new(Gender::Male));
    vec![
        RosterCourse {
            course_id: 10,
            name: "요가 초급".to_owned(),
            capacity: Capacity::new(2).unwrap(),
            entries: vec![
                RosterEntry {
                    result: RosterResult::Selected,
                    waitlist_number: None,
                    guaranteed: true,
                    member: first,
                },
                RosterEntry {
                    result: RosterResult::Waitlisted,
                    waitlist_number: Some(1),
                    guaranteed: false,
                    member: member(2, "김철", "00034", "021234567"),
                },
                RosterEntry {
                    result: RosterResult::Withdrawn,
                    waitlist_number: None,
                    guaranteed: false,
                    member: member(3, "남궁민수", "00056", "0311234567"),
                },
            ],
        },
        RosterCourse {
            course_id: 11,
            name: "서예".to_owned(),
            capacity: Capacity::new(1).unwrap(),
            entries: vec![RosterEntry {
                result: RosterResult::Selected,
                waitlist_number: None,
                guaranteed: false,
                member: member(4, "박영희", "00078", "01098765432"),
            }],
        },
    ]
}

/// 만든 Excel의 `index`번째 시트 이름과 모든 칸(글자로 바꿈).
pub fn read_sheet(bytes: &[u8], index: usize) -> (String, Vec<Vec<String>>) {
    let mut workbook = Xlsx::new(Cursor::new(bytes.to_vec())).unwrap();
    let name = workbook.sheet_names()[index].clone();
    let range = workbook.worksheet_range_at(index).unwrap().unwrap();
    let rows = range
        .rows()
        .map(|row| row.iter().map(ToString::to_string).collect())
        .collect();
    (name, rows)
}
