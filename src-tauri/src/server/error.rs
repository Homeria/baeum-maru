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

use super::applications::RuleViolationResponse;
use crate::db::DeleteError;

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
    /// BR-A5: 신청 규칙을 어겼다. 어긴 규칙을 모두 담는다.
    RulesBroken(Vec<RuleViolationResponse>),
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
    /// ADR 0012: 앱과 호스트의 버전 앞 두 자리가 다르다.
    VersionMismatch,
    /// DB 같은 내부 문제. 자세한 내용은 로그에만 있다.
    Internal,
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
    DuplicateRoomName,
    RoomNotFound,
    DuplicateSubjectName,
    SubjectNotFound,
    SubjectInUse,
    RoomInUse,
    DuplicateTermName,
    TermNotFound,
    TermInUse,
    DuplicateCourse,
    CourseNotFound,
    RulesBroken,
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
    VersionMismatch,
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
            ApiError::RulesBroken(_) => (
                StatusCode::CONFLICT,
                ErrorCode::RulesBroken,
                "신청 규칙을 어긴 강좌가 있습니다",
                Some("BR-A5"),
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
                "PIN이 맞지 않습니다. 5번 연속 틀리면 5분 동안 잠깁니다",
                None,
                Vec::new(),
            ),
            ApiError::AccountLocked => (
                StatusCode::FORBIDDEN,
                ErrorCode::AccountLocked,
                "PIN을 5번 연속 틀려 잠겼습니다. 5분이 지나면 다시 할 수 있습니다",
                Some("BR-K7"),
                Vec::new(),
            ),
            ApiError::AccountUnavailable => (
                StatusCode::FORBIDDEN,
                ErrorCode::AccountUnavailable,
                "사용 중지했거나 유효기간이 지난 계정입니다. 관리자에게 물어 주세요",
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
            ApiError::VersionMismatch => (
                StatusCode::CONFLICT,
                ErrorCode::VersionMismatch,
                "이 PC의 배움마루 버전이 호스트와 다릅니다. 같은 버전으로 맞춰 주세요",
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
