//! 호스트 PC의 화면만 쓰는 앱 명령: 접속 주소, 기다리는 승인 요청 수 (ADR 0004, BR-K15).
//!
//! 서버 API가 아니라 앱 명령이라 그 PC의 창에서만 부를 수 있다. 승인 요청 수를 자주 물어도
//! 로그인 세션을 건드리지 않아, 10분 화면 잠금(BR-K8)이 그대로 동작한다.

use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

use sqlx::SqlitePool;
use tauri::{AppHandle, Manager};
use time::OffsetDateTime;

use crate::db::devices;

/// 다른 PC가 입력할 이 PC의 주소(예: `192.168.0.12`). 찾지 못하면 `None`.
///
/// 같은 네트워크로 나가는 길의 이 PC 쪽 주소를 쓴다. UDP 소켓을 바깥 주소에 "연결"하면
/// 운영체제가 쓸 주소를 고르는데, 이때 실제로 보내는 것은 없다.
#[tauri::command]
pub fn host_address() -> Option<String> {
    let socket = UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0))).ok()?;
    // 주소를 고르게 하려는 것뿐이라 닿지 않아도 된다 (문서용 예시 주소, RFC 5737)
    socket
        .connect(SocketAddr::from((Ipv4Addr::new(192, 0, 2, 1), 9)))
        .ok()?;
    match socket.local_addr().ok()?.ip() {
        IpAddr::V4(ip) if !ip.is_loopback() && !ip.is_unspecified() => Some(ip.to_string()),
        _ => None,
    }
}

/// 5분이 지나지 않은 기다리는 승인 요청 수. 호스트가 아니거나 읽지 못하면 0.
#[tauri::command]
pub async fn open_device_request_count(app: AppHandle) -> usize {
    let Some(db) = app.try_state::<SqlitePool>() else {
        return 0;
    };
    match devices::open_requests(&db, OffsetDateTime::now_utc()).await {
        Ok(requests) => requests.len(),
        Err(error) => {
            tracing::error!(%error, "승인 요청 수를 읽지 못했습니다");
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_address_is_a_usable_ipv4_when_found() {
        // 네트워크가 없는 PC에서는 못 찾을 수 있다. 찾았으면 다른 PC가 쓸 수 있는 모양이어야 한다
        if let Some(address) = host_address() {
            let ip: Ipv4Addr = address.parse().unwrap();
            assert!(!ip.is_loopback() && !ip.is_unspecified(), "{address}");
        }
    }
}
