//! 버전 맞춰 보기.
//!
//! ADR 0012: 접속할 때 `/api/version`으로 호환부터 확인하고,
//! 다른 요청도 버전을 실어 보내며 앞 두 자리가 다르면 거절한다.

use axum::Json;
use axum::extract::{Request, State};
use axum::http::HeaderMap;
use axum::middleware::Next;
use axum::response::Response;
use serde::Serialize;
use ts_rs::TS;

use super::error::ApiError;
use crate::version::{self, Version};

/// 요청한 앱이 자기 버전을 싣는 헤더.
pub const VERSION_HEADER: &str = "x-baeum-version";

/// `/api/version` 응답.
#[derive(Debug, Serialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct VersionResponse {
    /// 호스트 버전.
    pub host_version: String,
    /// 요청한 앱과 함께 쓸 수 있는지. 헤더가 없거나 형식이 틀리면 `false`.
    pub compatible: bool,
}

/// `/api/version`: 기기 승인 없이 버전만 알려 준다.
pub async fn get_version(State(host): State<Version>, headers: HeaderMap) -> Json<VersionResponse> {
    Json(VersionResponse {
        host_version: version::CURRENT.to_owned(),
        compatible: is_compatible(&headers, host),
    })
}

/// 요청의 버전이 호스트와 맞을 때만 다음으로 넘긴다. `/api/version` 말고 모든 요청에 붙인다.
///
/// # Errors
///
/// 헤더가 없거나, 형식이 틀리거나, 앞 두 자리가 다르면 [`ApiError::VersionMismatch`].
pub async fn require_compatible(
    State(host): State<Version>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    if is_compatible(request.headers(), host) {
        Ok(next.run(request).await)
    } else {
        Err(ApiError::VersionMismatch)
    }
}

/// 헤더가 없거나 형식이 틀리면 `false`.
fn is_compatible(headers: &HeaderMap, host: Version) -> bool {
    headers
        .get(VERSION_HEADER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<Version>().ok())
        .is_some_and(|client| client.is_compatible_with(host))
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn call(host: &str, client: Option<&str>) -> VersionResponse {
        let mut headers = HeaderMap::new();
        if let Some(client) = client {
            headers.insert(VERSION_HEADER, client.parse().unwrap());
        }
        get_version(State(host.parse().unwrap()), headers).await.0
    }

    #[tokio::test]
    async fn compatible_when_only_patch_differs() {
        assert!(call("0.3.0", Some("0.3.9")).await.compatible);
    }

    #[tokio::test]
    async fn incompatible_when_minor_differs() {
        assert!(!call("0.3.0", Some("0.4.0")).await.compatible);
    }

    #[tokio::test]
    async fn incompatible_without_header() {
        assert!(!call("0.3.0", None).await.compatible);
    }

    #[tokio::test]
    async fn incompatible_with_broken_header() {
        assert!(!call("0.3.0", Some("abc")).await.compatible);
    }

    #[tokio::test]
    async fn reports_host_version() {
        assert_eq!(call("0.3.0", None).await.host_version, version::CURRENT);
    }
}
