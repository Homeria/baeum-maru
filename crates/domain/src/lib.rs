//! 배움마루 업무 규칙.
//!
//! 신청, 선발, 대기 같은 규칙(`docs/domain/BUSINESS_RULES.md`)을 둔다.
//! 기능은 6단계 기능 슬라이스에서 하나씩 채운다.
//!
//! ADR 0006: DB, 서버, 화면을 모르는 순수한 코드만 둔다.

// [docs/conventions.md]: 공개 항목은 문서 주석 필수
#![warn(missing_docs, clippy::missing_errors_doc, clippy::missing_panics_doc)]

mod application;
mod course;
mod device;
mod handover;
mod lottery;
mod member;
mod personal;
mod priority;
mod recovery_code;
mod room;
mod roster;
mod staff;
mod subject;
mod term;
mod waitlist;

pub use application::{
    ApplicationRule, CourseForRules, Reason, ReasonError, RuleViolation, check_application,
};
pub use course::{
    Capacity, ClassTime, CourseError, CourseInput, LevelName, NewCourse, Schedule, ScheduleInput,
    SectionName, weekday_from_number,
};
pub use device::{APPROVAL_WINDOW, ApprovalCode, DeviceError, DeviceName, is_approval_open};
pub use handover::{DataBundleId, DataBundleIdError, HostCheck, HostIdentity, check_host};
pub use lottery::{
    Allocation, LotteryDraw, LotterySeed, LotteryVersion, OverCapacity, allocate, draw_lottery,
};
pub use member::{Gender, MemberError, MemberInput, MemberName, MemberNo, NewMember, PhoneNumber};
pub use personal::Personal;
pub use priority::{NewPriorityReason, PriorityError, PriorityMethod, PriorityReasonName};
pub use recovery_code::RecoveryCode;
pub use room::{BuildingName, Floor, NewRoom, RoomError, RoomInput, RoomName};
pub use roster::{
    MaskChar, RosterChoice, RosterColumn, RosterError, RosterKind, RosterSpec, format_phone_number,
    mask_member_no, mask_name, mask_phone_number, masked_birth_date,
};
pub use staff::{
    ACCOUNT_LOCK, Action, FailedPin, MAX_FAILED_PINS, NewStaff, Pin, Role, SCREEN_LOCK, StaffError,
    StaffInput, StaffName, StaffProfile, is_account_locked, is_screen_locked, is_valid_on,
    record_failed_pin,
};
pub use subject::{CategoryName, NewSubject, SubjectError, SubjectInput, SubjectName};
pub use term::{NewTerm, TermError, TermInput, TermName, TermPeriod};
pub use waitlist::{
    AdditionalOutcome, ContactResult, Waiting, WithdrawnFrom, additional_outcome, contact_turns,
    open_seats,
};
