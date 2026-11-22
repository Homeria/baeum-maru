//! 직원 계정과 역할, PIN, 잠금 규칙 (BR-K).

use time::{Date, Duration, OffsetDateTime};

/// 직원 항목이 형식에 맞지 않는다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum StaffError {
    /// 이름이 비었다.
    #[error("이름을 입력해 주세요")]
    EmptyName,
    /// 이름이 너무 길다.
    #[error("이름은 20자까지 씁니다")]
    NameTooLong,
    /// BR-K4: PIN이 숫자 4~8자리가 아니다.
    #[error("PIN은 숫자 4~8자리입니다")]
    InvalidPin,
    /// BR-K4: 관리자 PIN이 6자리보다 짧다.
    #[error("관리자 PIN은 6자리 이상입니다")]
    AdminPinTooShort,
    /// BR-K5: 임시 직원인데 유효기간이 없다.
    #[error("임시 직원은 유효기간을 정해 주세요")]
    MissingValidUntil,
}

/// 직원 이름. 1~20자이고 앞뒤 공백은 지운다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffName(String);

impl StaffName {
    /// 가장 긴 글자 수.
    pub const MAX_CHARS: usize = 20;

    /// 입력한 글자에서 이름을 만든다.
    ///
    /// # Errors
    ///
    /// 비었으면 [`StaffError::EmptyName`], 20자를 넘으면 [`StaffError::NameTooLong`].
    pub fn parse(input: &str) -> Result<Self, StaffError> {
        let trimmed = input.trim();
        match trimmed.chars().count() {
            0 => Err(StaffError::EmptyName),
            n if n > Self::MAX_CHARS => Err(StaffError::NameTooLong),
            _ => Ok(StaffName(trimmed.to_owned())),
        }
    }

    /// 이름 글자. 쓴 띄어쓰기 그대로다.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// BR-K2: 겹침을 볼 때 쓰는 값. 띄어쓰기를 모두 뺀다.
    pub fn key(&self) -> String {
        self.0.chars().filter(|c| !c.is_whitespace()).collect()
    }
}

/// 직원의 역할 (BR-K3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// 관리자. 모든 일을 한다.
    Admin,
    /// 직원. 직원 관리와 접속 기록 보기 말고 모두.
    Staff,
    /// 임시 직원(사회복무요원, 자원봉사자). 보기, 회원 등록·고치기, 신청 받기·지우기만.
    TempStaff,
    /// 조회 전용. 보기만.
    ReadOnly,
}

/// 역할에 따라 할 수 있는지가 갈리는 일. [`Role::can`]에 쓴다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// 모든 목록과 항목 보기.
    View,
    /// 회원 등록·고치기.
    EditMembers,
    /// 회원 지우기.
    DeleteMembers,
    /// 학기·강좌 등록·고치기·지우기.
    EditCourses,
    /// 기초 정보(과목·강의실) 등록·고치기·지우기.
    EditReferenceData,
    /// 신청 받기·지우기.
    TakeApplications,
    /// 신청 규칙을 어긴 신청을 사유를 적고 받기(BR-A6).
    OverrideRules,
    /// 신청에 우선 배정 사유 붙이기·떼기(BR-S12).
    AssignPriority,
    /// 추첨·재추첨·선발 마감(BR-L4, BR-L5). 호스트 PC에서만 한다(BR-L6)는 앱이 따로 확인한다.
    RunLottery,
    /// 우선 배정 사유 목록 바꾸기(BR-S10).
    ManagePriorityReasons,
    /// 직원 계정 관리.
    ManageStaff,
    /// 접속 기록 보기.
    ViewAccessLog,
}

impl Role {
    /// 모든 역할. 화면 순서.
    pub const ALL: [Role; 4] = [Role::Admin, Role::Staff, Role::TempStaff, Role::ReadOnly];

    /// DB와 JSON에 쓰는 이름. 예: `temp_staff`
    pub fn code(self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::Staff => "staff",
            Role::TempStaff => "temp_staff",
            Role::ReadOnly => "read_only",
        }
    }

    /// [`Role::code`]의 이름에서 역할을 찾는다. 모르는 이름이면 `None`.
    pub fn from_code(code: &str) -> Option<Self> {
        Role::ALL.into_iter().find(|role| role.code() == code)
    }

    /// 이 역할이 `action`을 할 수 있는지. 표는 BUSINESS_RULES의 "역할별로 할 수 있는 일".
    pub fn can(self, action: Action) -> bool {
        match (self, action) {
            (_, Action::View) | (Role::Admin, _) => true,
            (
                Role::Staff,
                Action::ManagePriorityReasons | Action::ManageStaff | Action::ViewAccessLog,
            ) => false,
            (Role::Staff, _) => true,
            (Role::TempStaff, Action::EditMembers | Action::TakeApplications) => true,
            (Role::TempStaff | Role::ReadOnly, _) => false,
        }
    }
}

/// 확인을 통과한 PIN. 숫자 4~8자리 (BR-K4).
///
/// 비밀이라 `Debug`로 찍어도 숫자가 보이지 않는다. DB에는 해시만 둔다 (ADR 0014).
#[derive(Clone, PartialEq, Eq)]
pub struct Pin(String);

impl Pin {
    /// 가장 짧은 자리 수.
    pub const MIN_DIGITS: usize = 4;
    /// 관리자의 가장 짧은 자리 수.
    pub const ADMIN_MIN_DIGITS: usize = 6;
    /// 가장 긴 자리 수.
    pub const MAX_DIGITS: usize = 8;

    /// `role`인 직원이 쓸 PIN을 만든다. 앞뒤 공백은 지운다.
    ///
    /// # Errors
    ///
    /// 숫자 4~8자리가 아니면 [`StaffError::InvalidPin`], 관리자인데 6자리보다 짧으면
    /// [`StaffError::AdminPinTooShort`].
    pub fn parse(input: &str, role: Role) -> Result<Self, StaffError> {
        let pin = input.trim();
        let digits = pin.len();
        if !pin.chars().all(|c| c.is_ascii_digit())
            || !(Self::MIN_DIGITS..=Self::MAX_DIGITS).contains(&digits)
        {
            return Err(StaffError::InvalidPin);
        }
        if role == Role::Admin && digits < Self::ADMIN_MIN_DIGITS {
            return Err(StaffError::AdminPinTooShort);
        }
        Ok(Pin(pin.to_owned()))
    }

    /// PIN 글자. 해시를 만들 때만 쓴다.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for Pin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Pin(***)")
    }
}

/// 화면에서 받은 직원 정보. PIN은 따로 받는다. 확인 전이다.
#[derive(Debug, Clone, Copy)]
pub struct StaffInput<'a> {
    /// 이름.
    pub name: &'a str,
    /// 역할.
    pub role: Role,
    /// 유효기간(마지막 날). 임시 직원은 필수.
    pub valid_until: Option<Date>,
}

/// 확인을 통과한 직원 정보: 이름, 역할, 유효기간. 고치기에 쓰고, [`NewStaff`]가 PIN과 함께 담는다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffProfile {
    name: StaffName,
    role: Role,
    valid_until: Option<Date>,
}

impl StaffProfile {
    /// 입력을 모두 확인한다.
    ///
    /// # Errors
    ///
    /// 틀린 항목을 화면 순서(이름, 유효기간)대로 **모두** 모아 돌려준다.
    pub fn parse(input: StaffInput<'_>) -> Result<Self, Vec<StaffError>> {
        let (name, valid_until) = parse_name_and_valid_until(input);
        match (name, valid_until) {
            (Ok(name), Ok(valid_until)) => Ok(StaffProfile {
                name,
                role: input.role,
                valid_until,
            }),
            (name, valid_until) => Err([name.err(), valid_until.err()]
                .into_iter()
                .flatten()
                .collect()),
        }
    }

    /// 이름.
    pub fn name(&self) -> &StaffName {
        &self.name
    }

    /// 역할.
    pub fn role(&self) -> Role {
        self.role
    }

    /// 유효기간(마지막 날). 없으면 기한 없음.
    pub fn valid_until(&self) -> Option<Date> {
        self.valid_until
    }
}

/// 확인을 통과한 새 직원. DB에 저장하기 전의 모양이다.
///
/// BR-K2(이름 겹침)는 DB가 막는다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewStaff {
    profile: StaffProfile,
    pin: Pin,
}

impl NewStaff {
    /// 직원 정보와 PIN을 모두 확인한다.
    ///
    /// # Errors
    ///
    /// 틀린 항목을 화면 순서(이름, PIN, 유효기간)대로 **모두** 모아 돌려준다.
    pub fn parse(input: StaffInput<'_>, pin: &str) -> Result<Self, Vec<StaffError>> {
        let (name, valid_until) = parse_name_and_valid_until(input);
        let pin = Pin::parse(pin, input.role);
        match (name, pin, valid_until) {
            (Ok(name), Ok(pin), Ok(valid_until)) => Ok(NewStaff {
                profile: StaffProfile {
                    name,
                    role: input.role,
                    valid_until,
                },
                pin,
            }),
            (name, pin, valid_until) => Err([name.err(), pin.err(), valid_until.err()]
                .into_iter()
                .flatten()
                .collect()),
        }
    }

    /// 이름, 역할, 유효기간.
    pub fn profile(&self) -> &StaffProfile {
        &self.profile
    }

    /// PIN.
    pub fn pin(&self) -> &Pin {
        &self.pin
    }
}

/// 이름과 유효기간을 따로 확인한다. BR-K5: 임시 직원은 유효기간이 반드시 있다.
fn parse_name_and_valid_until(
    input: StaffInput<'_>,
) -> (
    Result<StaffName, StaffError>,
    Result<Option<Date>, StaffError>,
) {
    let name = StaffName::parse(input.name);
    let valid_until = match (input.role, input.valid_until) {
        (Role::TempStaff, None) => Err(StaffError::MissingValidUntil),
        (_, valid_until) => Ok(valid_until),
    };
    (name, valid_until)
}

/// BR-K5: 유효기간이 `today`에도 남았는지. 마지막 날까지 쓸 수 있다. 기한이 없으면 늘 남았다.
pub fn is_valid_on(valid_until: Option<Date>, today: Date) -> bool {
    valid_until.is_none_or(|last_day| today <= last_day)
}

/// BR-K7: 연속으로 틀리면 계정을 잠그는 횟수.
pub const MAX_FAILED_PINS: u32 = 5;

/// BR-K7: 계정을 잠그는 시간.
pub const ACCOUNT_LOCK: Duration = Duration::minutes(5);

/// BR-K8: 요청이 없으면 세션을 잠그는 시간.
pub const SCREEN_LOCK: Duration = Duration::minutes(10);

/// BR-K7: `locked_until`까지 잠긴 계정이 `now`에도 잠겨 있는지. 그 시각이 되면 풀린다.
pub fn is_account_locked(locked_until: Option<OffsetDateTime>, now: OffsetDateTime) -> bool {
    locked_until.is_some_and(|until| now < until)
}

/// BR-K8: 마지막 요청이 `last_seen_at`인 세션이 `now`에 잠겼는지. 10분이 되면 잠긴다.
pub fn is_screen_locked(last_seen_at: OffsetDateTime, now: OffsetDateTime) -> bool {
    now - last_seen_at >= SCREEN_LOCK
}

/// PIN을 틀린 뒤의 계정 상태.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FailedPin {
    /// 새로 센 연속 틀린 횟수. 잠그면 0으로 돌아간다.
    pub failed_count: u32,
    /// 잠갔으면 풀리는 시각.
    pub locked_until: Option<OffsetDateTime>,
}

/// BR-K7: 지금까지 `failed_count`번 틀린 계정이 `now`에 또 틀렸다.
/// 5번째면 5분 잠그고 횟수를 0으로 돌린다.
pub fn record_failed_pin(failed_count: u32, now: OffsetDateTime) -> FailedPin {
    let failed_count = failed_count + 1;
    if failed_count >= MAX_FAILED_PINS {
        FailedPin {
            failed_count: 0,
            locked_until: Some(now + ACCOUNT_LOCK),
        }
    } else {
        FailedPin {
            failed_count,
            locked_until: None,
        }
    }
}

#[cfg(test)]
mod tests {

    use time::macros::{date, datetime};

    use super::*;

    fn input(name: &str, role: Role, valid_until: Option<Date>) -> StaffInput<'_> {
        StaffInput {
            name,
            role,
            valid_until,
        }
    }

    #[test]
    fn br_k2_name_key_ignores_spaces() {
        assert_eq!(StaffName::parse(" 직원 1 ").unwrap().key(), "직원1");
    }

    #[test]
    fn br_k3_role_table() {
        use Action::*;
        let all = [
            View,
            EditMembers,
            DeleteMembers,
            EditCourses,
            EditReferenceData,
            TakeApplications,
            OverrideRules,
            AssignPriority,
            RunLottery,
            ManagePriorityReasons,
            ManageStaff,
            ViewAccessLog,
        ];
        let allowed = |role: Role| -> Vec<Action> {
            all.into_iter().filter(|action| role.can(*action)).collect()
        };

        assert_eq!(allowed(Role::Admin), all);
        assert_eq!(
            allowed(Role::Staff),
            [
                View,
                EditMembers,
                DeleteMembers,
                EditCourses,
                EditReferenceData,
                TakeApplications,
                OverrideRules,
                AssignPriority,
                RunLottery
            ]
        );
        assert_eq!(
            allowed(Role::TempStaff),
            [View, EditMembers, TakeApplications]
        );
        assert_eq!(allowed(Role::ReadOnly), [View]);
    }

    #[test]
    fn role_code_round_trips() {
        for role in Role::ALL {
            assert_eq!(Role::from_code(role.code()), Some(role));
        }
        assert_eq!(Role::from_code("boss"), None);
    }

    #[test]
    fn br_k4_pin_is_four_to_eight_digits() {
        for ok in ["1234", "12345678", " 0000 "] {
            assert!(Pin::parse(ok, Role::Staff).is_ok(), "{ok}");
        }
        for bad in ["123", "123456789", "12a4", "", "１２３４"] {
            assert_eq!(
                Pin::parse(bad, Role::Staff),
                Err(StaffError::InvalidPin),
                "{bad}"
            );
        }
    }

    #[test]
    fn br_k4_admin_pin_is_six_digits_or_more() {
        assert_eq!(
            Pin::parse("12345", Role::Admin),
            Err(StaffError::AdminPinTooShort)
        );
        assert!(Pin::parse("123456", Role::Admin).is_ok());
    }

    #[test]
    fn pin_debug_hides_digits() {
        let pin = Pin::parse("1234", Role::Staff).unwrap();
        assert_eq!(format!("{pin:?}"), "Pin(***)");
    }

    #[test]
    fn br_k5_temp_staff_needs_valid_until() {
        assert_eq!(
            StaffProfile::parse(input("직원1", Role::TempStaff, None)),
            Err(vec![StaffError::MissingValidUntil])
        );
        assert!(
            StaffProfile::parse(input("직원1", Role::TempStaff, Some(date!(2027 - 03 - 31))))
                .is_ok()
        );
        assert!(StaffProfile::parse(input("직원1", Role::Staff, None)).is_ok());
    }

    #[test]
    fn new_staff_collects_every_error_in_screen_order() {
        let errors = NewStaff::parse(input("", Role::TempStaff, None), "12").unwrap_err();

        assert_eq!(
            errors,
            [
                StaffError::EmptyName,
                StaffError::InvalidPin,
                StaffError::MissingValidUntil
            ]
        );
    }

    #[test]
    fn new_staff_pin_error_comes_first_without_name_error() {
        let errors = NewStaff::parse(input("직원1", Role::TempStaff, None), "12").unwrap_err();

        assert_eq!(
            errors,
            [StaffError::InvalidPin, StaffError::MissingValidUntil]
        );
    }

    #[test]
    fn br_k5_valid_through_last_day() {
        let last_day = Some(date!(2027 - 03 - 31));

        assert!(is_valid_on(last_day, date!(2027 - 03 - 31)));
        assert!(!is_valid_on(last_day, date!(2027 - 04 - 01)));
        assert!(is_valid_on(None, date!(2099 - 01 - 01)));
    }

    #[test]
    fn br_k7_fifth_failure_locks_for_five_minutes() {
        let now = datetime!(2027-03-02 09:00:00 UTC);

        assert_eq!(
            record_failed_pin(3, now),
            FailedPin {
                failed_count: 4,
                locked_until: None
            }
        );
        assert_eq!(
            record_failed_pin(4, now),
            FailedPin {
                failed_count: 0,
                locked_until: Some(now + Duration::minutes(5))
            }
        );
    }

    fn at(hour: u8, minute: u8, second: u8) -> OffsetDateTime {
        date!(2026 - 11 - 15)
            .with_hms(hour, minute, second)
            .unwrap()
            .assume_utc()
    }

    #[test]
    fn account_unlocks_at_locked_until() {
        let until = Some(at(1, 5, 0));

        assert!(is_account_locked(until, at(1, 4, 59)));
        assert!(!is_account_locked(until, at(1, 5, 0)));
        assert!(!is_account_locked(None, at(1, 0, 0)));
    }

    #[test]
    fn screen_locks_ten_minutes_after_last_request() {
        let last_seen = at(1, 0, 0);

        assert!(!is_screen_locked(last_seen, at(1, 9, 59)));
        assert!(is_screen_locked(last_seen, at(1, 10, 0)));
    }
}
