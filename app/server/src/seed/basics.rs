//! 직원, 학기, 강의실, 과목, 강좌 더미 데이터.

use std::collections::HashMap;

use baeum_domain::{
    CourseInput, NewCourse, NewRoom, NewStaff, NewSubject, NewTerm, Role, RoomInput, ScheduleInput,
    StaffInput, SubjectInput, TermInput,
};
use sqlx::SqlitePool;
use time::Weekday::{Friday, Monday, Thursday, Tuesday, Wednesday};
use time::{Date, Duration, OffsetDateTime, Weekday};

use super::{SeedError, fail};
use crate::features::courses::repository as courses;
use crate::features::rooms::repository as rooms;
use crate::features::staff::repository as staff;
use crate::features::subjects::repository as subjects;
use crate::features::terms::repository as terms;
use crate::infra::secret::{self, AdminSecrets};

/// 개발용 고정 PIN. `docs/development.md`에도 적는다.
const ADMIN_PIN: &str = "123456";
const STAFF_PIN: &str = "1234";

/// 관리자 말고 넣는 직원. 이름으로 역할을 알아보게 했다.
const OTHER_STAFF: [(&str, Role); 3] = [
    ("직원", Role::Staff),
    ("임시직원", Role::TempStaff),
    ("조회전용", Role::ReadOnly),
];

/// (건물, 층, 이름). 이름은 건물이 달라도 겹치지 않게 해서 강좌가 이름으로 찾는다.
const ROOMS: [(&str, Option<i32>, &str); 9] = [
    ("본관", Some(-1), "다목적실"),
    ("본관", Some(1), "101호"),
    ("본관", Some(1), "102호"),
    ("본관", Some(2), "201호"),
    ("본관", Some(2), "202호"),
    ("본관", Some(3), "강당"),
    ("별관", Some(1), "체력단련실"),
    ("별관", Some(2), "음악실"),
    ("별관", None, "야외 마당"),
];

/// (이름, 분류).
const SUBJECTS: [(&str, Option<&str>); 9] = [
    ("요가", Some("건강")),
    ("라인댄스", Some("건강")),
    ("탁구", Some("건강")),
    ("노래교실", Some("음악")),
    ("하모니카", Some("음악")),
    ("서예", Some("미술")),
    ("수채화", Some("미술")),
    ("스마트폰 활용", None),
    ("영어 회화", None),
];

/// 강좌 하나. 수업 시간은 (요일, 시작, 끝, 강의실 이름).
pub struct CourseSpec {
    pub subject: &'static str,
    level: Option<&'static str>,
    section: Option<&'static str>,
    capacity: u16,
    schedules: &'static [(Weekday, &'static str, &'static str, &'static str)],
    /// 넣을 신청 수. 정원과 견줘 신청 없음, 미만, 같음, 조금 넘음, 두 배 가까이가 모두 나오게 했다.
    pub demand: usize,
}

/// 정원은 5명부터 30명까지. 주 2회 강좌, 같은 과목의 여러 수준·반이 섞여 있다.
pub const COURSES: [CourseSpec; 12] = [
    CourseSpec {
        subject: "요가",
        level: Some("초급"),
        section: Some("1반"),
        capacity: 20,
        schedules: &[
            (Monday, "09:00", "10:30", "다목적실"),
            (Wednesday, "09:00", "10:30", "다목적실"),
        ],
        demand: 40,
    },
    CourseSpec {
        subject: "요가",
        level: Some("초급"),
        section: Some("2반"),
        capacity: 20,
        schedules: &[
            (Tuesday, "09:00", "10:30", "다목적실"),
            (Thursday, "09:00", "10:30", "다목적실"),
        ],
        demand: 22,
    },
    CourseSpec {
        subject: "요가",
        level: Some("중급"),
        section: None,
        capacity: 15,
        schedules: &[(Friday, "09:00", "10:30", "다목적실")],
        demand: 15,
    },
    CourseSpec {
        subject: "라인댄스",
        level: None,
        section: None,
        capacity: 30,
        schedules: &[(Tuesday, "13:00", "14:30", "체력단련실")],
        demand: 45,
    },
    CourseSpec {
        subject: "탁구",
        level: Some("초급"),
        section: None,
        capacity: 12,
        schedules: &[
            (Wednesday, "14:00", "16:00", "체력단련실"),
            (Friday, "14:00", "16:00", "체력단련실"),
        ],
        demand: 8,
    },
    CourseSpec {
        subject: "노래교실",
        level: None,
        section: None,
        capacity: 30,
        schedules: &[(Thursday, "10:00", "12:00", "강당")],
        demand: 30,
    },
    CourseSpec {
        subject: "하모니카",
        level: Some("초급"),
        section: None,
        capacity: 10,
        schedules: &[(Monday, "13:00", "14:30", "음악실")],
        demand: 0,
    },
    CourseSpec {
        subject: "서예",
        level: None,
        section: None,
        capacity: 15,
        schedules: &[(Tuesday, "10:00", "12:00", "201호")],
        demand: 18,
    },
    CourseSpec {
        subject: "수채화",
        level: Some("초급"),
        section: None,
        capacity: 12,
        schedules: &[(Wednesday, "10:00", "12:00", "202호")],
        demand: 5,
    },
    CourseSpec {
        subject: "스마트폰 활용",
        level: Some("기초"),
        section: None,
        capacity: 20,
        schedules: &[(Monday, "10:00", "11:30", "101호")],
        demand: 33,
    },
    CourseSpec {
        subject: "스마트폰 활용",
        level: Some("심화"),
        section: None,
        capacity: 5,
        schedules: &[(Thursday, "13:00", "14:30", "101호")],
        demand: 9,
    },
    CourseSpec {
        subject: "영어 회화",
        level: None,
        section: None,
        capacity: 10,
        schedules: &[(Friday, "13:00", "14:30", "102호")],
        demand: 10,
    },
];

/// 넣은 것 가운데 다음 단계가 쓰는 id.
pub struct Basics {
    pub recovery_code: String,
    /// 신청을 받은 직원으로 쓴다.
    pub admin_id: i64,
    pub term_id: i64,
    /// [`COURSES`]와 같은 순서.
    pub course_ids: Vec<i64>,
}

pub async fn insert(
    pool: &SqlitePool,
    today: Date,
    now: OffsetDateTime,
) -> Result<Basics, SeedError> {
    let (admin_id, recovery_code) = insert_staff(pool, today, now).await?;
    let term_id = insert_term(pool, today, now).await?;
    let room_ids = insert_rooms(pool, now).await?;
    let subject_ids = insert_subjects(pool, now).await?;

    let mut course_ids = Vec::with_capacity(COURSES.len());
    for spec in &COURSES {
        let course = course_input(spec, &subject_ids, &room_ids)?;
        let id = courses::insert(pool, term_id, &course, now)
            .await
            .map_err(fail("강좌"))?;
        course_ids.push(id);
    }

    Ok(Basics {
        recovery_code,
        admin_id,
        term_id,
        course_ids,
    })
}

/// 관리자(처음 설정과 같은 길)와 나머지 역할 직원 하나씩. 관리자 id와 복구 코드를 돌려준다.
async fn insert_staff(
    pool: &SqlitePool,
    today: Date,
    now: OffsetDateTime,
) -> Result<(i64, String), SeedError> {
    let admin = NewStaff::parse(
        StaffInput {
            name: "관리자",
            role: Role::Admin,
            valid_until: None,
        },
        ADMIN_PIN,
    )
    .map_err(fail("관리자"))?;
    let secrets = AdminSecrets::new(admin.pin()).map_err(fail("관리자 PIN"))?;
    let admin_id = crate::features::setup::repository::setup(
        pool,
        admin.profile().name(),
        &secrets.pin_hash,
        &secrets.recovery_code_hash,
        now,
    )
    .await
    .map_err(fail("관리자"))?;

    let pin_hash = secret::hash(STAFF_PIN).map_err(fail("직원 PIN"))?;
    for (name, role) in OTHER_STAFF {
        // BR-K5: 임시 직원은 유효기간이 있어야 한다
        let valid_until = (role == Role::TempStaff).then(|| today + Duration::days(90));
        let new_staff = NewStaff::parse(
            StaffInput {
                name,
                role,
                valid_until,
            },
            STAFF_PIN,
        )
        .map_err(fail("직원"))?;
        staff::insert(pool, &new_staff, &pin_hash, now)
            .await
            .map_err(fail("직원"))?;
    }

    Ok((admin_id, secrets.recovery_code.as_str().to_owned()))
}

/// 오늘이 들어가는 학기 하나. 한 달 전에 시작해 두 달 뒤에 끝난다.
async fn insert_term(
    pool: &SqlitePool,
    today: Date,
    now: OffsetDateTime,
) -> Result<i64, SeedError> {
    let name = format!("{}년 더미 학기", today.year());
    let term = NewTerm::parse(TermInput {
        name: &name,
        starts_on: Some(today - Duration::days(30)),
        ends_on: Some(today + Duration::days(60)),
    })
    .map_err(fail("학기"))?;
    terms::insert(pool, &term, now).await.map_err(fail("학기"))
}

/// 강의실 이름으로 id를 찾는 표를 돌려준다.
async fn insert_rooms(
    pool: &SqlitePool,
    now: OffsetDateTime,
) -> Result<HashMap<&'static str, i64>, SeedError> {
    let mut ids = HashMap::new();
    for (building, floor, name) in ROOMS {
        let room = NewRoom::parse(RoomInput {
            name,
            building,
            floor,
        })
        .map_err(fail("강의실"))?;
        let id = rooms::insert(pool, &room, now)
            .await
            .map_err(fail("강의실"))?;
        ids.insert(name, id);
    }
    Ok(ids)
}

/// 과목 이름으로 id를 찾는 표를 돌려준다.
async fn insert_subjects(
    pool: &SqlitePool,
    now: OffsetDateTime,
) -> Result<HashMap<&'static str, i64>, SeedError> {
    let mut ids = HashMap::new();
    for (name, category) in SUBJECTS {
        let subject = NewSubject::parse(SubjectInput { name, category }).map_err(fail("과목"))?;
        let id = subjects::insert(pool, &subject, now)
            .await
            .map_err(fail("과목"))?;
        ids.insert(name, id);
    }
    Ok(ids)
}

fn course_input(
    spec: &CourseSpec,
    subject_ids: &HashMap<&'static str, i64>,
    room_ids: &HashMap<&'static str, i64>,
) -> Result<NewCourse, SeedError> {
    let schedules = spec
        .schedules
        .iter()
        .map(|&(weekday, start, end, room)| ScheduleInput {
            weekday,
            start,
            end,
            room_id: room_ids.get(room).copied(),
        })
        .collect();
    NewCourse::parse(CourseInput {
        subject_id: subject_ids.get(spec.subject).copied(),
        level: spec.level,
        section: spec.section,
        capacity: Some(i64::from(spec.capacity)),
        schedules,
    })
    .map_err(fail("강좌"))
}
