//! 요청 처리의 오류와 그 JSON 응답.
//!
//! docs/conventions.md: 오류 처리, 오류 응답 모양.

use std::fmt::Display;

use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use ts_rs::TS;

/// 요청을 처리하지 못했다.
#[derive(Debug)]
pub enum ApiError {
    /// 본문이 JSON이 아니거나 모양이 틀렸다. 화면 코드의 잘못이다.
    MalformedRequest,
    /// 입력한 칸이 형식에 맞지 않는다.
    InvalidInput(Vec<FieldError>),
    /// BR-M2: 회원번호는 겹치지 않는다.
    DuplicateMemberNo,
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
}

/// 오류 종류. 한 번 정하면 바꾸지 않는다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[ts(export)]
pub enum ErrorCode {
    MalformedRequest,
    InvalidInput,
    DuplicateMemberNo,
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
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
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
