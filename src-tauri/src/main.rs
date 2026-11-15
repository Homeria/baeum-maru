//! 배움마루 앱.
//!
//! 지금은 서버만 띄운다. 창(Tauri)은 화면 뼈대가 생기면 붙인다.

mod server;
mod version;

use std::error::Error;
use std::net::{Ipv4Addr, SocketAddr};

use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let host: version::Version = version::CURRENT.parse()?;

    // [ADR 0001]: 다른 PC도 접속하므로 모든 주소에서 받음
    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, server::PORT));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, version = version::CURRENT, "서버 시작");

    axum::serve(listener, server::router(host)).await?;
    Ok(())
}
