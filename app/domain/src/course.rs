//! 강좌와 수업 시간 항목, 형식 확인.

use time::Weekday;

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

/// 요일 번호(1=월 … 7=일, ISO 8601)에서 요일을 만든다. 범위 밖이면 `None`.
///
/// DB와 JSON이 이 번호를 쓴다. 거꾸로는 [`Weekday::number_from_monday`].
///
/// # Examples
///
/// ```
/// use baeum_domain::weekday_from_number;
/// use time::Weekday;
///
/// assert_eq!(weekday_from_number(1), Some(Weekday::Monday));
/// assert_eq!(weekday_from_number(7), Some(Weekday::Sunday));
/// assert_eq!(weekday_from_number(0), None);
/// ```
pub fn weekday_from_number(number: u8) -> Option<Weekday> {
    match number {
        1..=7 => Some(Weekday::Monday.nth_next(number - 1)),
        _ => None,
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
mod tests;
