//! 요청 처리의 오류와 그 JSON 응답.
//!
//! docs/conventions.md: 오류 처리, 오류 응답 모양.

use std::fmt::Display;

use axum::Json;
use axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use ts_rs::TS;

// TODO(#88): 신청 기능을 옮길 때 이 의존(http → 신청)을 없앤다. 지금은 공통층이 신청 응답 타입을 안다
use crate::infra::db::DeleteError;
use crate::pin_check::PinRefusal;
use crate::server::applications::RuleViolationResponse;

/// 요청을 처리하지 못했다.
#[derive(Debug)]
pub enum ApiError {
    /// 본문이 JSON이 아니거나 모양이 틀렸다. 화면 코드의 잘못이다.
    MalformedRequest,
    /// 입력한 칸이 형식에 맞지 않는다.
    InvalidInput(Vec<FieldError>),
    /// BR-M2: 회원번호는 겹치지 않는다.
    DuplicateMemberNo,
    /// 주소의 id인 회원이 없다.
    MemberNotFound,
    /// BR-M5: 신청이나 삭제 기록이 있는 회원은 지울 수 없다.
    MemberInUse,
    /// BR-R1: 같은 건물 안에서 강의실 이름은 겹치지 않는다.
    DuplicateRoomName,
    /// 주소의 id인 강의실이 없다.
    RoomNotFound,
    /// BR-J1: 과목 이름은 기관 전체에서 겹치지 않는다.
    DuplicateSubjectName,
    /// 주소의 id인 과목이 없다.
    SubjectNotFound,
    /// BR-J2: 강좌가 쓰는 과목은 지울 수 없다.
    SubjectInUse,
    /// BR-S10: 우선 배정 사유 이름은 겹치지 않는다.
    DuplicatePriorityReasonName,
    /// 주소의 id인 우선 배정 사유가 없다.
    PriorityReasonNotFound,
    /// BR-S10: 신청에 붙었던 사유는 지울 수 없다.
    PriorityReasonInUse,
    /// BR-R2: 수업 시간이 쓰는 강의실은 지울 수 없다.
    RoomInUse,
    /// BR-T1: 학기 이름은 겹치지 않는다.
    DuplicateTermName,
    /// 주소의 id인 학기가 없다.
    TermNotFound,
    /// BR-T5: 강좌가 있는 학기는 지울 수 없다.
    TermInUse,
    /// BR-T3: 같은 학기 안에서 과목·수준·반이 모두 같은 강좌는 하나뿐이다.
    DuplicateCourse,
    /// 주소의 id인 강좌가 없다.
    CourseNotFound,
    /// BR-T6: 신청이나 삭제 기록이 있는 강좌는 지울 수 없다.
    CourseInUse,
    /// BR-L3: 추첨한 강좌는 선발 마감 전까지 새 신청을 받지 않는다.
    CourseDrawn,
    /// BR-X1: 학기 종료일이 지나 추가 신청을 받지 않는다.
    TermEnded,
    /// BR-L5: 선발을 마감한 강좌는 다시 추첨하거나 마감할 수 없다.
    SelectionClosed,
    /// 추첨 전에는 마감할 수 없다.
    NotDrawn,
    /// BR-C3: 포기와 승격은 선발 마감 뒤에만 한다.
    SelectionNotClosed,
    /// BR-P1: 학기에 선발을 마감한 강좌가 없어 명단을 만들 수 없다.
    NoClosedCourses,
    /// 신청이 지금 상태라 할 수 없다. 예: 대기자가 아닌 신청에 연락 결과
    SelectionChangeNotAllowed,
    /// BR-W3, BR-W8: 빈자리가 없어 당첨으로 바꿀 수 없다.
    NoOpenSeat,
    /// 접수됨 신청이 없어 추첨할 수 없다.
    NoApplicants,
    /// BR-S5: 자리 보장 신청이 정원보다 많아 추첨할 수 없다.
    PriorityOverCapacity,
    /// BR-L6, ADR 0011: 호스트 PC에서만 하는 일이다. 예: 추첨, 두 번째 위치 바꾸기
    HostOnly,
    /// BR-A5: 신청 규칙을 어겼다. 어긴 규칙을 모두 담는다.
    RulesBroken(Vec<RuleViolationResponse>),
    /// 주소의 id인 신청이 없다.
    ApplicationNotFound,
    /// BR-D1: 접수됨(선발 전) 신청만 지울 수 있다.
    ApplicationNotDeletable,
    /// 유효한 신청(접수됨, 당첨, 대기)이 아니라 우선 배정을 바꿀 수 없다.
    ApplicationNotChangeable,
    /// BR-K11: 처음 설정은 직원이 하나도 없을 때 한 번만 한다.
    SetupAlreadyDone,
    /// 주소나 본문의 id인 직원이 없다.
    StaffNotFound,
    /// BR-K2: 직원 이름은 겹치지 않는다.
    DuplicateStaffName,
    /// BR-K9: 사용 중인 관리자가 하나도 남지 않게 된다.
    LastAdmin,
    /// 로그인하거나 화면 잠금을 풀 때 PIN이 틀렸다.
    WrongPin,
    /// BR-K7: PIN을 5번 연속 틀려 계정이 잠겼다.
    AccountLocked,
    /// BR-K5, BR-K6: 사용 중지했거나 유효기간이 지난 계정이다.
    AccountUnavailable,
    /// BR-K12: 복구 코드가 맞지 않거나 이미 썼다.
    WrongRecoveryCode,
    /// BR-K12: 복구 코드로는 관리자 PIN만 다시 정한다.
    RecoveryAdminOnly,
    /// BR-K1: 로그인하지 않았거나 세션이 끝났다.
    LoginRequired,
    /// BR-K8: 10분 동안 요청이 없어 세션이 잠겼다.
    SessionLocked,
    /// BR-K3: 이 직원의 역할로는 할 수 없는 일이다.
    NotAllowed,
    /// BR-K14: 승인받지 않았거나 승인을 취소한 기기다.
    DeviceNotApproved,
    /// BR-K15: 그런 승인 요청이 없거나, 이미 허락·거절했거나, 5분이 지났다.
    ApprovalRequestNotOpen,
    /// 기다리는 승인 요청이 너무 많다.
    TooManyApprovalRequests,
    /// 주소의 id인 기기가 없거나 이미 승인을 취소했다.
    DeviceNotFound,
    /// ADR 0012: 앱과 호스트의 버전 앞 두 자리가 다르다.
    VersionMismatch,
    /// ADR 0011: 지금 백업을 만들지 못했다. 자세한 내용은 로그에만 있다.
    BackupFailed,
    /// DB 같은 내부 문제. 자세한 내용은 로그에만 있다.
    Internal,
}

/// PIN 확인 거절을 같은 문장의 응답으로 바꾼다.
impl From<PinRefusal> for ApiError {
    fn from(refusal: PinRefusal) -> Self {
        match refusal {
            PinRefusal::WrongPin => ApiError::WrongPin,
            PinRefusal::AccountLocked => ApiError::AccountLocked,
            PinRefusal::AccountUnavailable => ApiError::AccountUnavailable,
            PinRefusal::Internal => ApiError::Internal,
        }
    }
}

impl ApiError {
    /// 내부 오류를 로그에 남기고 [`ApiError::Internal`]을 만든다.
    ///
    /// `error`에 회원 정보가 들어가지 않게 한다 (docs/conventions.md: 로그와 개인정보).
    pub fn internal(error: impl Display) -> Self {
        tracing::error!(%error, "요청을 처리하지 못했습니다");
        ApiError::Internal
    }

    /// 지우기 실패를 바꾼다. 다른 데이터가 쓰고 있으면 `in_use`, DB 문제면 [`ApiError::Internal`].
    pub fn from_delete(error: DeleteError, in_use: ApiError) -> Self {
        match error {
            DeleteError::InUse => in_use,
            DeleteError::Db(error) => ApiError::internal(error),
        }
    }

    /// 업무 규칙 확인에서 나온 오류를 칸마다 [`ApiError::InvalidInput`]으로 모은다.
    ///
    /// `field_name`은 오류가 어느 칸(요청 JSON 이름)의 것인지 알려 준다.
    pub fn invalid_input<E: Display>(errors: Vec<E>, field_name: fn(&E) -> &'static str) -> Self {
        let fields = errors
            .iter()
            .map(|error| FieldError {
                field: field_name(error).to_owned(),
                message: error.to_string(),
            })
            .collect();
        ApiError::InvalidInput(fields)
    }
}

/// 오류 종류. 한 번 정하면 바꾸지 않는다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[ts(export)]
pub enum ErrorCode {
    MalformedRequest,
    InvalidInput,
    DuplicateMemberNo,
    MemberNotFound,
    MemberInUse,
    DuplicateRoomName,
    RoomNotFound,
    DuplicateSubjectName,
    SubjectNotFound,
    SubjectInUse,
    DuplicatePriorityReasonName,
    PriorityReasonNotFound,
    PriorityReasonInUse,
    RoomInUse,
    DuplicateTermName,
    TermNotFound,
    TermInUse,
    DuplicateCourse,
    CourseNotFound,
    CourseInUse,
    CourseDrawn,
    TermEnded,
    SelectionClosed,
    NotDrawn,
    SelectionNotClosed,
    NoClosedCourses,
    SelectionChangeNotAllowed,
    NoOpenSeat,
    NoApplicants,
    PriorityOverCapacity,
    HostOnly,
    RulesBroken,
    ApplicationNotFound,
    ApplicationNotDeletable,
    ApplicationNotChangeable,
    SetupAlreadyDone,
    StaffNotFound,
    DuplicateStaffName,
    LastAdmin,
    WrongPin,
    AccountLocked,
    AccountUnavailable,
    WrongRecoveryCode,
    RecoveryAdminOnly,
    LoginRequired,
    SessionLocked,
    NotAllowed,
    DeviceNotApproved,
    ApprovalRequestNotOpen,
    TooManyApprovalRequests,
    DeviceNotFound,
    VersionMismatch,
    BackupFailed,
    Internal,
}

/// 형식에 맞지 않는 칸 하나.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct FieldError {
    /// 요청 JSON의 칸 이름. 예: `memberNo`
    pub field: String,
    /// 직원에게 보여 줄 문장.
    pub message: String,
}

/// 오류 응답 본문.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct ErrorResponse {
    pub code: ErrorCode,
    pub message: String,
    /// 업무 규칙 위반일 때만. 예: `BR-M2`
    pub rule: Option<String>,
    /// [`ErrorCode::InvalidInput`]일 때 틀린 칸들. 그 밖에는 비어 있다.
    pub fields: Vec<FieldError>,
    /// [`ErrorCode::RulesBroken`]일 때 어긴 신청 규칙들. 그 밖에는 비어 있다.
    pub violations: Vec<RuleViolationResponse>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let violations = match &self {
            ApiError::RulesBroken(violations) => violations.clone(),
            _ => Vec::new(),
        };
        let (status, code, message, rule, fields) = match self {
            ApiError::MalformedRequest => (
                StatusCode::BAD_REQUEST,
                ErrorCode::MalformedRequest,
                "요청 모양이 맞지 않습니다",
                None,
                Vec::new(),
            ),
            ApiError::InvalidInput(fields) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                ErrorCode::InvalidInput,
                "입력을 확인해 주세요",
                None,
                fields,
            ),
            ApiError::DuplicateMemberNo => (
                StatusCode::CONFLICT,
                ErrorCode::DuplicateMemberNo,
                "이미 있는 회원번호입니다",
                Some("BR-M2"),
                Vec::new(),
            ),
            ApiError::MemberNotFound => (
                StatusCode::NOT_FOUND,
                ErrorCode::MemberNotFound,
                "회원을 찾지 못했습니다. 지워졌을 수 있습니다",
                None,
                Vec::new(),
            ),
            ApiError::MemberInUse => (
                StatusCode::CONFLICT,
                ErrorCode::MemberInUse,
                "신청 기록이 있는 회원은 지울 수 없습니다. 지운 신청의 기록도 포함합니다",
                Some("BR-M5"),
                Vec::new(),
            ),
            ApiError::DuplicateRoomName => (
                StatusCode::CONFLICT,
                ErrorCode::DuplicateRoomName,
                "같은 건물에 같은 이름의 강의실이 있습니다",
                Some("BR-R1"),
                Vec::new(),
            ),
            ApiError::RoomNotFound => (
                StatusCode::NOT_FOUND,
                ErrorCode::RoomNotFound,
                "강의실을 찾지 못했습니다. 지워졌을 수 있습니다",
                None,
                Vec::new(),
            ),
            ApiError::DuplicateSubjectName => (
                StatusCode::CONFLICT,
                ErrorCode::DuplicateSubjectName,
                "이미 있는 과목입니다. 띄어쓰기만 달라도 같은 과목으로 봅니다",
                Some("BR-J1"),
                Vec::new(),
            ),
            ApiError::SubjectNotFound => (
                StatusCode::NOT_FOUND,
                ErrorCode::SubjectNotFound,
                "과목을 찾지 못했습니다. 지워졌을 수 있습니다",
                None,
                Vec::new(),
            ),
            ApiError::SubjectInUse => (
                StatusCode::CONFLICT,
                ErrorCode::SubjectInUse,
                "강좌가 쓰는 과목은 지울 수 없습니다. 지난 학기 강좌도 이 과목을 씁니다",
                Some("BR-J2"),
                Vec::new(),
            ),
            ApiError::DuplicatePriorityReasonName => (
                StatusCode::CONFLICT,
                ErrorCode::DuplicatePriorityReasonName,
                "이미 있는 사유입니다. 띄어쓰기만 달라도 같은 사유로 봅니다",
                Some("BR-S10"),
                Vec::new(),
            ),
            ApiError::PriorityReasonNotFound => (
                StatusCode::NOT_FOUND,
                ErrorCode::PriorityReasonNotFound,
                "우선 배정 사유를 찾지 못했습니다. 지워졌을 수 있습니다",
                None,
                Vec::new(),
            ),
            ApiError::PriorityReasonInUse => (
                StatusCode::CONFLICT,
                ErrorCode::PriorityReasonInUse,
                "신청에 붙인 적이 있는 사유는 지울 수 없습니다. 이름은 고칠 수 있습니다",
                Some("BR-S10"),
                Vec::new(),
            ),
            ApiError::RoomInUse => (
                StatusCode::CONFLICT,
                ErrorCode::RoomInUse,
                "강좌의 수업 시간이 쓰는 강의실은 지울 수 없습니다. 지난 학기 강좌도 이 강의실을 씁니다",
                Some("BR-R2"),
                Vec::new(),
            ),
            ApiError::DuplicateTermName => (
                StatusCode::CONFLICT,
                ErrorCode::DuplicateTermName,
                "같은 이름의 학기가 있습니다",
                Some("BR-T1"),
                Vec::new(),
            ),
            ApiError::TermNotFound => (
                StatusCode::NOT_FOUND,
                ErrorCode::TermNotFound,
                "학기를 찾지 못했습니다. 지워졌을 수 있습니다",
                None,
                Vec::new(),
            ),
            ApiError::TermInUse => (
                StatusCode::CONFLICT,
                ErrorCode::TermInUse,
                "강좌가 있는 학기는 지울 수 없습니다. 강좌를 먼저 지워 주세요",
                Some("BR-T5"),
                Vec::new(),
            ),
            ApiError::DuplicateCourse => (
                StatusCode::CONFLICT,
                ErrorCode::DuplicateCourse,
                "이 학기에 과목·수준·반이 모두 같은 강좌가 있습니다. 수준이나 반으로 나눠 주세요",
                Some("BR-T3"),
                Vec::new(),
            ),
            ApiError::CourseNotFound => (
                StatusCode::NOT_FOUND,
                ErrorCode::CourseNotFound,
                "강좌를 찾지 못했습니다. 지워졌을 수 있습니다",
                None,
                Vec::new(),
            ),
            ApiError::CourseInUse => (
                StatusCode::CONFLICT,
                ErrorCode::CourseInUse,
                "신청 기록이 있는 강좌는 지울 수 없습니다. 지운 신청의 기록도 포함합니다",
                Some("BR-T6"),
                Vec::new(),
            ),
            ApiError::CourseDrawn => (
                StatusCode::CONFLICT,
                ErrorCode::CourseDrawn,
                "추첨한 강좌는 선발 마감 전까지 새 신청을 받지 않습니다. 고른 강좌에서 빼 주세요",
                Some("BR-L3"),
                Vec::new(),
            ),
            ApiError::TermEnded => (
                StatusCode::CONFLICT,
                ErrorCode::TermEnded,
                "학기가 끝나 추가 신청을 받지 않습니다. 고른 강좌에서 빼 주세요",
                Some("BR-X1"),
                Vec::new(),
            ),
            ApiError::SelectionClosed => (
                StatusCode::CONFLICT,
                ErrorCode::SelectionClosed,
                "선발을 마감한 강좌입니다. 다시 추첨할 수 없습니다",
                Some("BR-L5"),
                Vec::new(),
            ),
            ApiError::NotDrawn => (
                StatusCode::CONFLICT,
                ErrorCode::NotDrawn,
                "추첨하지 않은 강좌는 마감할 수 없습니다",
                None,
                Vec::new(),
            ),
            ApiError::SelectionNotClosed => (
                StatusCode::CONFLICT,
                ErrorCode::SelectionNotClosed,
                "선발을 마감한 강좌만 포기와 승격을 처리합니다. 추첨 탭에서 먼저 마감해 주세요",
                Some("BR-C3"),
                Vec::new(),
            ),
            ApiError::NoClosedCourses => (
                StatusCode::CONFLICT,
                ErrorCode::NoClosedCourses,
                "선발을 마감한 강좌가 없어 명단을 만들 수 없습니다. 추첨 탭에서 먼저 마감해 주세요",
                Some("BR-P1"),
                Vec::new(),
            ),
            ApiError::SelectionChangeNotAllowed => (
                StatusCode::CONFLICT,
                ErrorCode::SelectionChangeNotAllowed,
                "신청 상태가 바뀌어 할 수 없습니다. 화면을 새로 고쳐 주세요",
                None,
                Vec::new(),
            ),
            ApiError::NoOpenSeat => (
                StatusCode::CONFLICT,
                ErrorCode::NoOpenSeat,
                "빈자리가 없어 당첨으로 바꿀 수 없습니다",
                Some("BR-W3"),
                Vec::new(),
            ),
            ApiError::NoApplicants => (
                StatusCode::CONFLICT,
                ErrorCode::NoApplicants,
                "추첨할 신청이 없습니다",
                None,
                Vec::new(),
            ),
            ApiError::PriorityOverCapacity => (
                StatusCode::CONFLICT,
                ErrorCode::PriorityOverCapacity,
                "우선 배정 신청이 정원보다 많아 추첨할 수 없습니다. 우선 배정을 줄이거나 정원을 늘려 주세요",
                Some("BR-S5"),
                Vec::new(),
            ),
            ApiError::HostOnly => (
                StatusCode::FORBIDDEN,
                ErrorCode::HostOnly,
                "호스트 PC에서만 할 수 있습니다",
                Some("BR-L6"),
                Vec::new(),
            ),
            ApiError::RulesBroken(_) => (
                StatusCode::CONFLICT,
                ErrorCode::RulesBroken,
                "신청 규칙을 어긴 강좌가 있습니다",
                Some("BR-A5"),
                Vec::new(),
            ),
            ApiError::ApplicationNotFound => (
                StatusCode::NOT_FOUND,
                ErrorCode::ApplicationNotFound,
                "신청을 찾지 못했습니다. 지워졌을 수 있습니다",
                None,
                Vec::new(),
            ),
            ApiError::ApplicationNotDeletable => (
                StatusCode::CONFLICT,
                ErrorCode::ApplicationNotDeletable,
                "선발 뒤의 신청은 지울 수 없습니다. 포기로 처리해 주세요",
                Some("BR-D1"),
                Vec::new(),
            ),
            ApiError::ApplicationNotChangeable => (
                StatusCode::CONFLICT,
                ErrorCode::ApplicationNotChangeable,
                "유효한 신청이 아니라 우선 배정을 바꿀 수 없습니다",
                Some("BR-S13"),
                Vec::new(),
            ),
            ApiError::SetupAlreadyDone => (
                StatusCode::CONFLICT,
                ErrorCode::SetupAlreadyDone,
                "이미 처음 설정을 마쳤습니다. 관리자 계정으로 로그인해 주세요",
                Some("BR-K11"),
                Vec::new(),
            ),
            ApiError::StaffNotFound => (
                StatusCode::NOT_FOUND,
                ErrorCode::StaffNotFound,
                "직원을 찾지 못했습니다",
                None,
                Vec::new(),
            ),
            ApiError::DuplicateStaffName => (
                StatusCode::CONFLICT,
                ErrorCode::DuplicateStaffName,
                "같은 이름의 직원이 있습니다. 띄어쓰기만 달라도 같은 이름으로 봅니다",
                Some("BR-K2"),
                Vec::new(),
            ),
            ApiError::LastAdmin => (
                StatusCode::CONFLICT,
                ErrorCode::LastAdmin,
                "사용 중인 관리자가 한 명뿐이라 사용 중지하거나 역할을 바꿀 수 없습니다. 다른 관리자를 먼저 만들어 주세요",
                Some("BR-K9"),
                Vec::new(),
            ),
            ApiError::WrongPin => (
                StatusCode::UNAUTHORIZED,
                ErrorCode::WrongPin,
                PinRefusal::WrongPin.message(),
                None,
                Vec::new(),
            ),
            ApiError::AccountLocked => (
                StatusCode::FORBIDDEN,
                ErrorCode::AccountLocked,
                PinRefusal::AccountLocked.message(),
                Some("BR-K7"),
                Vec::new(),
            ),
            ApiError::AccountUnavailable => (
                StatusCode::FORBIDDEN,
                ErrorCode::AccountUnavailable,
                PinRefusal::AccountUnavailable.message(),
                None,
                Vec::new(),
            ),
            ApiError::WrongRecoveryCode => (
                StatusCode::UNAUTHORIZED,
                ErrorCode::WrongRecoveryCode,
                "복구 코드가 맞지 않습니다. 가장 최근에 인쇄해 둔 코드를 넣어 주세요",
                Some("BR-K12"),
                Vec::new(),
            ),
            ApiError::RecoveryAdminOnly => (
                StatusCode::FORBIDDEN,
                ErrorCode::RecoveryAdminOnly,
                "복구 코드로는 관리자 PIN만 다시 정할 수 있습니다. 다른 직원의 PIN은 관리자에게 부탁해 주세요",
                Some("BR-K12"),
                Vec::new(),
            ),
            ApiError::LoginRequired => (
                StatusCode::UNAUTHORIZED,
                ErrorCode::LoginRequired,
                "로그인해 주세요",
                Some("BR-K1"),
                Vec::new(),
            ),
            ApiError::SessionLocked => (
                StatusCode::UNAUTHORIZED,
                ErrorCode::SessionLocked,
                "10분 동안 쓰지 않아 화면이 잠겼습니다. PIN을 넣어 다시 열어 주세요",
                Some("BR-K8"),
                Vec::new(),
            ),
            ApiError::NotAllowed => (
                StatusCode::FORBIDDEN,
                ErrorCode::NotAllowed,
                "이 역할로는 할 수 없는 일입니다",
                Some("BR-K3"),
                Vec::new(),
            ),
            ApiError::DeviceNotApproved => (
                StatusCode::UNAUTHORIZED,
                ErrorCode::DeviceNotApproved,
                "승인받지 않은 기기입니다. 호스트 PC에서 관리자의 승인을 받아 주세요",
                Some("BR-K14"),
                Vec::new(),
            ),
            ApiError::ApprovalRequestNotOpen => (
                StatusCode::CONFLICT,
                ErrorCode::ApprovalRequestNotOpen,
                "승인 요청이 없거나 5분이 지났습니다. 기기에서 다시 요청해 주세요",
                Some("BR-K15"),
                Vec::new(),
            ),
            ApiError::TooManyApprovalRequests => (
                StatusCode::TOO_MANY_REQUESTS,
                ErrorCode::TooManyApprovalRequests,
                "기다리는 승인 요청이 많습니다. 잠시 뒤에 다시 요청해 주세요",
                None,
                Vec::new(),
            ),
            ApiError::DeviceNotFound => (
                StatusCode::NOT_FOUND,
                ErrorCode::DeviceNotFound,
                "기기를 찾지 못했습니다. 이미 승인을 취소했을 수 있습니다",
                None,
                Vec::new(),
            ),
            ApiError::VersionMismatch => (
                StatusCode::CONFLICT,
                ErrorCode::VersionMismatch,
                "이 PC의 배움마루 버전이 호스트와 다릅니다. 같은 버전으로 맞춰 주세요",
                None,
                Vec::new(),
            ),
            ApiError::BackupFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::BackupFailed,
                "백업하지 못했습니다. 디스크 공간이 남아 있는지 확인해 주세요",
                None,
                Vec::new(),
            ),
            ApiError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::Internal,
                "처리하지 못했습니다. 잠시 뒤에 다시 해 주세요",
                None,
                Vec::new(),
            ),
        };
        let body = ErrorResponse {
            code,
            message: message.to_owned(),
            rule: rule.map(str::to_owned),
            fields,
            violations,
        };
        (status, Json(body)).into_response()
    }
}

/// axum이 본문을 읽지 못했을 때. 거절 사유에는 보낸 값이 섞일 수 있어 응답과 로그에 싣지 않는다.
impl From<JsonRejection> for ApiError {
    fn from(_: JsonRejection) -> Self {
        ApiError::MalformedRequest
    }
}

/// `?` 뒤 값을 읽지 못했을 때. 검색어가 섞일 수 있어 응답과 로그에 싣지 않는다.
impl From<QueryRejection> for ApiError {
    fn from(_: QueryRejection) -> Self {
        ApiError::MalformedRequest
    }
}

/// 주소의 id가 숫자가 아닐 때.
impl From<PathRejection> for ApiError {
    fn from(_: PathRejection) -> Self {
        ApiError::MalformedRequest
    }
}
