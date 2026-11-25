//! 버전 맞춰 보기.
//!
//! ADR 0012: 접속할 때 `/api/version`으로 호환부터 확인하고,
//! 다른 요청도 버전을 실어 보내며 앞 두 자리가 다르면 거절한다.
//! 같은 응답에 데이터 묶음 ID와 세대 번호도 실어, 접속 PC가 옛 호스트를 알아보게 한다(BR-H5).

use axum::Json;
use axum::extract::{Request, State};
use axum::http::HeaderMap;
use axum::middleware::Next;
use axum::response::Response;
use serde::Serialize;
use sqlx::SqlitePool;
use ts_rs::TS;

use super::error::ApiError;
use crate::db::data_bundle;
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
    /// 이 호스트 데이터의 묶음 ID(BR-H4). 무작위 값이라 개인정보가 아니다.
    pub bundle_id: String,
    /// 이 호스트의 세대 번호(BR-H4).
    pub generation: u32,
}

/// `/api/version`: 기기 승인 없이 버전과 데이터 묶음 ID, 세대 번호를 알려 준다.
///
/// # Errors
///
/// 묶음 정보를 읽지 못하면 [`ApiError::Internal`].
pub async fn get_version(
    State(host): State<Version>,
    State(db): State<SqlitePool>,
    headers: HeaderMap,
) -> Result<Json<VersionResponse>, ApiError> {
    let identity = data_bundle::read(&db).await.map_err(ApiError::internal)?;
    Ok(Json(VersionResponse {
        host_version: version::CURRENT.to_owned(),
        compatible: is_compatible(&headers, host),
        bundle_id: identity.bundle_id.to_string(),
        generation: identity.generation,
    }))
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
    use crate::db::open_temp;

    async fn call(host: &str, client: Option<&str>) -> VersionResponse {
        let (_dir, db) = open_temp().await;
        let mut headers = HeaderMap::new();
        if let Some(client) = client {
            headers.insert(VERSION_HEADER, client.parse().unwrap());
        }
        get_version(State(host.parse().unwrap()), State(db), headers)
            .await
            .unwrap()
            .0
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

    #[tokio::test]
    async fn br_h5_reports_bundle_id_and_generation() {
        let (_dir, db) = open_temp().await;
        let identity = data_bundle::read(&db).await.unwrap();

        let response = get_version(State("0.3.0".parse().unwrap()), State(db), HeaderMap::new())
            .await
            .unwrap()
            .0;

        assert_eq!(response.bundle_id, identity.bundle_id.to_string());
        assert_eq!(response.generation, 1);
    }
}
