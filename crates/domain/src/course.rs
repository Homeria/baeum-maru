//! 강좌와 수업 시간 항목, 형식 확인.

use chrono::Weekday;

/// 강좌 항목이 형식에 맞지 않는다.
///
/// 수업 시간 오류의 `row`는 화면의 몇 번째 줄인지(1부터)다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CourseError {
    /// 과목을 고르지 않았다.
    #[error("과목을 골라 주세요")]
    MissingSubject,
    /// 수준 이름이 너무 길다.
    #[error("수준은 20자까지 씁니다")]
    LevelTooLong,
    /// 반 이름이 너무 길다.
    #[error("반은 20자까지 씁니다")]
    SectionTooLong,
    /// 정원을 넣지 않았다.
    #[error("정원을 입력해 주세요")]
    MissingCapacity,
    /// 정원이 고를 수 있는 범위 밖이다.
    #[error("정원은 1명부터 200명까지입니다")]
    CapacityOutOfRange,
    /// BR-T4: 수업 시간이 한 줄도 없다.
    #[error("수업 시간을 한 줄 이상 넣어 주세요")]
    NoSchedule,
    /// 시각을 넣지 않았거나 모양이 틀렸다.
    #[error("{row}번째 수업 시간: 시작·종료 시각을 입력해 주세요")]
    InvalidTime {
        /// 몇 번째 줄인지.
        row: usize,
    },
    /// BR-T4: 종료 시각이 시작 시각보다 늦지 않다.
    #[error("{row}번째 수업 시간: 종료 시각은 시작 시각보다 늦어야 합니다")]
    EndNotAfterStart {
        /// 몇 번째 줄인지.
        row: usize,
    },
    /// BR-T4: 한 강좌 안의 두 줄이 겹친다.
    #[error("{first}번째와 {second}번째 수업 시간이 겹칩니다")]
    SchedulesOverlap {
        /// 앞 줄.
        first: usize,
        /// 뒤 줄.
        second: usize,
    },
}

/// 수준 이름. 예: 초급. 1~20자이고 앞뒤 공백은 지운다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelName(String);

impl LevelName {
    /// 가장 긴 글자 수.
    pub const MAX_CHARS: usize = 20;

    /// 입력한 글자에서 수준을 만든다. 수준은 선택이라 비었으면 `None`.
    ///
    /// # Errors
    ///
    /// 20자를 넘으면 [`CourseError::LevelTooLong`].
    pub fn parse(input: &str) -> Result<Option<Self>, CourseError> {
        optional_text(input, Self::MAX_CHARS, CourseError::LevelTooLong).map(|text| text.map(Self))
    }

    /// 수준 글자.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 반 이름. 예: 1반. 1~20자이고 앞뒤 공백은 지운다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionName(String);

impl SectionName {
    /// 가장 긴 글자 수.
    pub const MAX_CHARS: usize = 20;

    /// 입력한 글자에서 반을 만든다. 반은 선택이라 비었으면 `None`.
    ///
    /// # Errors
    ///
    /// 20자를 넘으면 [`CourseError::SectionTooLong`].
    pub fn parse(input: &str) -> Result<Option<Self>, CourseError> {
        optional_text(input, Self::MAX_CHARS, CourseError::SectionTooLong)
            .map(|text| text.map(Self))
    }

    /// 반 글자.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 선택 글자 칸: 앞뒤 공백을 지우고, 비었으면 `None`, 너무 길면 `too_long`.
fn optional_text(
    input: &str,
    max_chars: usize,
    too_long: CourseError,
) -> Result<Option<String>, CourseError> {
    let trimmed = input.trim();
    match trimmed.chars().count() {
        0 => Ok(None),
        n if n > max_chars => Err(too_long),
        _ => Ok(Some(trimmed.to_owned())),
    }
}

/// 정원. 1명부터 200명까지.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capacity(u16);

impl Capacity {
    /// 가장 적은 정원.
    pub const MIN: u16 = 1;
    /// 가장 많은 정원.
    pub const MAX: u16 = 200;

    /// 숫자에서 정원을 만든다.
    ///
    /// # Errors
    ///
    /// 1~200 밖이면 [`CourseError::CapacityOutOfRange`].
    pub fn new(value: i64) -> Result<Self, CourseError> {
        let value = u16::try_from(value).map_err(|_| CourseError::CapacityOutOfRange)?;
        if !(Self::MIN..=Self::MAX).contains(&value) {
            return Err(CourseError::CapacityOutOfRange);
        }
        Ok(Capacity(value))
    }

    /// 사람 수.
    pub fn value(self) -> u16 {
        self.0
    }
}

/// 하루 안의 시각. 자정부터 센 분으로 담는다 (10:00은 600).
///
/// 분으로 두면 겹침 비교가 숫자 비교 하나로 끝난다.
///
/// # Examples
///
/// ```
/// use baeum_domain::ClassTime;
///
/// let time = ClassTime::parse("10:30").unwrap();
/// assert_eq!(time.minutes(), 630);
/// assert_eq!(time.to_string(), "10:30");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClassTime(u16);

impl ClassTime {
    /// `HH:MM` 글자(00:00~23:59)에서 시각을 만든다. 모양이 틀리면 `None`.
    pub fn parse(input: &str) -> Option<Self> {
        let (hour, minute) = input.trim().split_once(':')?;
        if hour.len() != 2 || minute.len() != 2 {
            return None;
        }
        let hour: u16 = hour.parse().ok()?;
        let minute: u16 = minute.parse().ok()?;
        (hour < 24 && minute < 60).then_some(ClassTime(hour * 60 + minute))
    }

    /// 자정부터 센 분에서 시각을 만든다. 하루(1440분)를 넘으면 `None`.
    pub fn from_minutes(minutes: u16) -> Option<Self> {
        (minutes < 24 * 60).then_some(ClassTime(minutes))
    }

    /// 자정부터 센 분.
    pub fn minutes(self) -> u16 {
        self.0
    }
}

/// `HH:MM` 모양으로 쓴다.
impl std::fmt::Display for ClassTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}:{:02}", self.0 / 60, self.0 % 60)
    }
}

/// 수업 시간 한 줄: 요일, 시작·종료 시각, 강의실.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Schedule {
    weekday: Weekday,
    start: ClassTime,
    end: ClassTime,
    room_id: Option<i64>,
}

impl Schedule {
    /// 요일과 시각으로 수업 시간을 만든다. 강의실은 선택.
    ///
    /// BR-T4: 종료가 시작보다 늦지 않으면 `None`. 줄 번호를 붙인 오류는 [`NewCourse::parse`]가 만든다.
    pub fn new(
        weekday: Weekday,
        start: ClassTime,
        end: ClassTime,
        room_id: Option<i64>,
    ) -> Option<Self> {
        (start < end).then_some(Schedule {
            weekday,
            start,
            end,
            room_id,
        })
    }

    /// 요일.
    pub fn weekday(self) -> Weekday {
        self.weekday
    }

    /// 시작 시각.
    pub fn start(self) -> ClassTime {
        self.start
    }

    /// 종료 시각.
    pub fn end(self) -> ClassTime {
        self.end
    }

    /// 강의실 DB id. 없으면 `None`.
    pub fn room_id(self) -> Option<i64> {
        self.room_id
    }

    /// 두 수업 시간이 겹치는지. 같은 요일이고 시간이 조금이라도 겹치면 `true`.
    ///
    /// 끝나는 시각과 시작하는 시각이 같으면(10:00~11:00, 11:00~12:00) 겹치지 않는다.
    /// BR-T4(한 강좌 안)와 신청의 시간 겹침 금지(BR-A1)가 함께 쓴다.
    pub fn overlaps(self, other: Schedule) -> bool {
        self.weekday == other.weekday && self.start < other.end && other.start < self.end
    }
}

/// 화면에서 받은 수업 시간 한 줄. 시각은 확인 전 글자 그대로다.
#[derive(Debug, Clone, Copy)]
pub struct ScheduleInput<'a> {
    /// 요일.
    pub weekday: Weekday,
    /// 시작 시각 (`HH:MM`).
    pub start: &'a str,
    /// 종료 시각 (`HH:MM`).
    pub end: &'a str,
    /// 강의실 DB id. 선택.
    pub room_id: Option<i64>,
}

/// 화면에서 받은 강좌 입력. 확인 전이다. 학기는 주소에서 따로 받는다.
#[derive(Debug, Clone)]
pub struct CourseInput<'a> {
    /// 과목 DB id. 고르지 않았으면 `None`.
    pub subject_id: Option<i64>,
    /// 수준. 선택.
    pub level: Option<&'a str>,
    /// 반. 선택.
    pub section: Option<&'a str>,
    /// 정원. 넣지 않았으면 `None`.
    pub capacity: Option<i64>,
    /// 수업 시간 줄들. 화면 순서대로.
    pub schedules: Vec<ScheduleInput<'a>>,
}

/// 확인을 통과한 강좌. DB에 저장하기 전의 모양이다.
///
/// BR-T3(같은 학기 안 겹침)과 과목·강의실이 있는지는 DB가 막는다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewCourse {
    subject_id: i64,
    level: Option<LevelName>,
    section: Option<SectionName>,
    capacity: Capacity,
    schedules: Vec<Schedule>,
}

impl NewCourse {
    /// 입력을 모두 확인한다.
    ///
    /// # Errors
    ///
    /// 틀린 항목이 있으면 [`CourseError`]를 화면 순서(과목, 수준, 반, 정원, 수업 시간)대로
    /// **모두** 모아 돌려준다. 줄끼리 겹치는지는 모든 줄이 맞을 때만 본다.
    pub fn parse(input: CourseInput<'_>) -> Result<Self, Vec<CourseError>> {
        let subject_id = input.subject_id.ok_or(CourseError::MissingSubject);
        let level = input.level.map_or(Ok(None), LevelName::parse);
        let section = input.section.map_or(Ok(None), SectionName::parse);
        let capacity = input
            .capacity
            .ok_or(CourseError::MissingCapacity)
            .and_then(Capacity::new);
        let schedules = parse_schedules(&input.schedules);

        match (subject_id, level, section, capacity, schedules) {
            (Ok(subject_id), Ok(level), Ok(section), Ok(capacity), Ok(schedules)) => {
                Ok(NewCourse {
                    subject_id,
                    level,
                    section,
                    capacity,
                    schedules,
                })
            }
            (subject_id, level, section, capacity, schedules) => {
                Err(
                    [subject_id.err(), level.err(), section.err(), capacity.err()]
                        .into_iter()
                        .flatten()
                        .chain(schedules.err().into_iter().flatten())
                        .collect(),
                )
            }
        }
    }

    /// 과목 DB id.
    pub fn subject_id(&self) -> i64 {
        self.subject_id
    }

    /// 수준.
    pub fn level(&self) -> Option<&LevelName> {
        self.level.as_ref()
    }

    /// 반.
    pub fn section(&self) -> Option<&SectionName> {
        self.section.as_ref()
    }

    /// 정원.
    pub fn capacity(&self) -> Capacity {
        self.capacity
    }

    /// 수업 시간. 한 줄 이상이고 서로 겹치지 않는다.
    pub fn schedules(&self) -> &[Schedule] {
        &self.schedules
    }
}

/// BR-T4: 줄마다 확인하고, 모두 맞으면 줄끼리 겹치는지 본다.
fn parse_schedules(inputs: &[ScheduleInput<'_>]) -> Result<Vec<Schedule>, Vec<CourseError>> {
    if inputs.is_empty() {
        return Err(vec![CourseError::NoSchedule]);
    }

    let mut schedules = Vec::with_capacity(inputs.len());
    let mut errors = Vec::new();
    for (index, input) in inputs.iter().enumerate() {
        let row = index + 1;
        let times = ClassTime::parse(input.start).zip(ClassTime::parse(input.end));
        let Some((start, end)) = times else {
            errors.push(CourseError::InvalidTime { row });
            continue;
        };
        match Schedule::new(input.weekday, start, end, input.room_id) {
            Some(schedule) => schedules.push(schedule),
            None => errors.push(CourseError::EndNotAfterStart { row }),
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }

    for (i, first) in schedules.iter().enumerate() {
        for (j, second) in schedules.iter().enumerate().skip(i + 1) {
            if first.overlaps(*second) {
                errors.push(CourseError::SchedulesOverlap {
                    first: i + 1,
                    second: j + 1,
                });
            }
        }
    }
    if errors.is_empty() {
        Ok(schedules)
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn time(text: &str) -> ClassTime {
        ClassTime::parse(text).unwrap()
    }

    fn row(weekday: Weekday, start: &'static str, end: &'static str) -> ScheduleInput<'static> {
        ScheduleInput {
            weekday,
            start,
            end,
            room_id: None,
        }
    }

    fn input() -> CourseInput<'static> {
        CourseInput {
            subject_id: Some(1),
            level: Some("초급"),
            section: Some("1반"),
            capacity: Some(20),
            schedules: vec![
                row(Weekday::Mon, "10:00", "11:00"),
                row(Weekday::Wed, "10:00", "11:00"),
            ],
        }
    }

    #[test]
    fn class_time_reads_hh_mm() {
        assert_eq!(time("00:00").minutes(), 0);
        assert_eq!(time("09:05").minutes(), 545);
        assert_eq!(time("23:59").minutes(), 1439);
        assert_eq!(time("13:30").to_string(), "13:30");
    }

    #[test]
    fn class_time_rejects_wrong_shape() {
        for bad in ["", "9:00", "24:00", "10:60", "10-00", "가나:다라", "10:000"] {
            assert_eq!(ClassTime::parse(bad), None, "{bad}");
        }
    }

    #[test]
    fn class_time_from_minutes_stays_in_one_day() {
        assert_eq!(ClassTime::from_minutes(600), Some(time("10:00")));
        assert_eq!(ClassTime::from_minutes(1440), None);
    }

    #[test]
    fn capacity_is_1_to_200() {
        assert_eq!(Capacity::new(1).unwrap().value(), 1);
        assert_eq!(Capacity::new(200).unwrap().value(), 200);
        for bad in [0, 201, -1, i64::MAX] {
            assert_eq!(
                Capacity::new(bad),
                Err(CourseError::CapacityOutOfRange),
                "{bad}"
            );
        }
    }

    #[test]
    fn level_and_section_are_optional() {
        assert_eq!(LevelName::parse("  "), Ok(None));
        assert_eq!(SectionName::parse(""), Ok(None));
        assert_eq!(
            LevelName::parse(" 초급 ").unwrap().unwrap().as_str(),
            "초급"
        );
        assert_eq!(
            SectionName::parse(&"가".repeat(21)),
            Err(CourseError::SectionTooLong)
        );
    }

    #[test]
    fn br_t4_end_must_be_after_start() {
        let same = Schedule::new(Weekday::Mon, time("10:00"), time("10:00"), None);
        let reversed = Schedule::new(Weekday::Mon, time("11:00"), time("10:00"), None);

        assert_eq!(same, None);
        assert_eq!(reversed, None);
    }

    #[test]
    fn overlaps_only_on_same_weekday() {
        let monday = Schedule::new(Weekday::Mon, time("10:00"), time("11:00"), None).unwrap();
        let tuesday = Schedule::new(Weekday::Tue, time("10:00"), time("11:00"), None).unwrap();

        assert!(!monday.overlaps(tuesday));
    }

    #[test]
    fn overlaps_when_times_cross() {
        let first = Schedule::new(Weekday::Mon, time("10:00"), time("11:00"), None).unwrap();
        let second = Schedule::new(Weekday::Mon, time("10:30"), time("12:00"), None).unwrap();
        let inside = Schedule::new(Weekday::Mon, time("10:15"), time("10:45"), None).unwrap();

        assert!(first.overlaps(second));
        assert!(second.overlaps(first));
        assert!(first.overlaps(inside));
    }

    #[test]
    fn back_to_back_does_not_overlap() {
        let first = Schedule::new(Weekday::Mon, time("10:00"), time("11:00"), None).unwrap();
        let next = Schedule::new(Weekday::Mon, time("11:00"), time("12:00"), None).unwrap();

        assert!(!first.overlaps(next));
    }

    #[test]
    fn parse_keeps_every_field() {
        let course = NewCourse::parse(input()).unwrap();

        assert_eq!(course.subject_id(), 1);
        assert_eq!(course.level().map(LevelName::as_str), Some("초급"));
        assert_eq!(course.section().map(SectionName::as_str), Some("1반"));
        assert_eq!(course.capacity().value(), 20);
        assert_eq!(course.schedules().len(), 2);
        assert_eq!(course.schedules()[1].weekday(), Weekday::Wed);
    }

    #[test]
    fn rooms_can_differ_by_row() {
        let course = NewCourse::parse(CourseInput {
            schedules: vec![
                ScheduleInput {
                    room_id: Some(5),
                    ..row(Weekday::Mon, "10:00", "11:00")
                },
                ScheduleInput {
                    room_id: Some(7),
                    ..row(Weekday::Wed, "10:00", "11:00")
                },
            ],
            ..input()
        })
        .unwrap();

        let rooms: Vec<Option<i64>> = course.schedules().iter().map(|s| s.room_id()).collect();
        assert_eq!(rooms, [Some(5), Some(7)]);
    }

    #[test]
    fn br_t4_needs_at_least_one_schedule() {
        let errors = NewCourse::parse(CourseInput {
            schedules: Vec::new(),
            ..input()
        })
        .unwrap_err();

        assert_eq!(errors, [CourseError::NoSchedule]);
    }

    #[test]
    fn br_t4_reports_each_bad_row() {
        let errors = NewCourse::parse(CourseInput {
            schedules: vec![
                row(Weekday::Mon, "10:00", "11:00"),
                row(Weekday::Tue, "", "11:00"),
                row(Weekday::Wed, "11:00", "10:00"),
            ],
            ..input()
        })
        .unwrap_err();

        assert_eq!(
            errors,
            [
                CourseError::InvalidTime { row: 2 },
                CourseError::EndNotAfterStart { row: 3 }
            ]
        );
    }

    #[test]
    fn br_t4_rows_in_one_course_must_not_overlap() {
        let errors = NewCourse::parse(CourseInput {
            schedules: vec![
                row(Weekday::Mon, "10:00", "11:00"),
                row(Weekday::Wed, "10:00", "11:00"),
                row(Weekday::Mon, "10:30", "11:30"),
            ],
            ..input()
        })
        .unwrap_err();

        assert_eq!(
            errors,
            [CourseError::SchedulesOverlap {
                first: 1,
                second: 3
            }]
        );
    }

    #[test]
    fn collects_every_error_in_screen_order() {
        let errors = NewCourse::parse(CourseInput {
            subject_id: None,
            level: Some(&"가".repeat(21)),
            section: None,
            capacity: None,
            schedules: Vec::new(),
        })
        .unwrap_err();

        assert_eq!(
            errors,
            [
                CourseError::MissingSubject,
                CourseError::LevelTooLong,
                CourseError::MissingCapacity,
                CourseError::NoSchedule
            ]
        );
    }

    #[test]
    fn error_message_names_the_row() {
        assert_eq!(
            CourseError::EndNotAfterStart { row: 2 }.to_string(),
            "2번째 수업 시간: 종료 시각은 시작 시각보다 늦어야 합니다"
        );
    }
}
