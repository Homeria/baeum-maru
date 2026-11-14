//! 버린 코드: 실험 03(Tauri 한글 입력) + 04(다른 기기에서 접속)
//!
//! 앱을 켜면
//! 1. 이 PC의 모든 네트워크 주소(0.0.0.0)에서 작은 웹 서버가 뜬다.
//! 2. 앱 창은 그 서버의 화면(http://127.0.0.1:포트)을 연다.
//! 3. 같은 Wi-Fi의 휴대폰이나 다른 PC는 화면에 나온 주소로 같은 화면에 들어올 수 있다.
//!
//! 앱 창과 휴대폰 브라우저가 같은 화면을 쓰므로, 한글 입력과 접속을 한 번에 시험할 수 있다.

// Windows에서 실행할 때 뒤에 검은 콘솔 창이 같이 뜨지 않게 한다 (release 빌드만).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use std::net::{IpAddr, TcpListener as StdTcpListener};

use axum::response::Html;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tauri::{WebviewUrl, WebviewWindowBuilder};

/// 쓰고 싶은 포트. 이미 쓰이고 있으면 운영체제가 빈 포트를 골라준다.
const PREFERRED_PORT: u16 = 47800;

/// 화면 파일을 실행 파일 안에 그대로 넣는다 (컴파일할 때 읽힘).
const PAGE: &str = include_str!("../dist/index.html");

fn main() -> Result<(), Box<dyn Error>> {
    // 창을 열기 전에 포트부터 잡는다. 그래야 창이 열렸을 때 서버가 이미 준비돼 있다.
    let listener = bind_listener()?;
    let port = listener.local_addr()?.port();
    listener.set_nonblocking(true)?; // tokio에 넘기려면 필요

    tauri::Builder::default()
        .setup(move |app| {
            tauri::async_runtime::spawn(serve(listener, port));

            let url: tauri::Url = format!("http://127.0.0.1:{port}/?from=app").parse()?;
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url))
                .title("배움마루 실험 03·04")
                .inner_size(960.0, 800.0)
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())?;
    Ok(())
}

/// 0.0.0.0은 "이 PC의 모든 네트워크 주소"라는 뜻이다. 127.0.0.1로 열면 이 PC 안에서만 들어올 수 있다.
fn bind_listener() -> std::io::Result<StdTcpListener> {
    StdTcpListener::bind(("0.0.0.0", PREFERRED_PORT)).or_else(|_| StdTcpListener::bind(("0.0.0.0", 0)))
}

async fn serve(listener: StdTcpListener, port: u16) {
    let listener = match tokio::net::TcpListener::from_std(listener) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("서버를 시작하지 못함: {e}");
            return;
        }
    };

    let app = Router::new()
        .route("/", get(|| async { Html(PAGE) }))
        .route("/api/info", get(move || info(port)))
        .route("/api/echo", post(echo));

    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("서버 오류: {e}");
    }
}

#[derive(Serialize)]
struct Info {
    computer: String,
    port: u16,
    addresses: Vec<Address>,
}

#[derive(Serialize)]
struct Address {
    interface: String,
    url: String,
}

/// 화면에 보여줄 접속 주소 목록.
async fn info(port: u16) -> Json<Info> {
    let computer = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "알 수 없음".into());

    let addresses = local_ip_address::list_afinet_netifas()
        .unwrap_or_default()
        .into_iter()
        .filter(|(_, ip)| matches!(ip, IpAddr::V4(v4) if !v4.is_loopback() && !v4.is_link_local()))
        .map(|(interface, ip)| Address { interface, url: format!("http://{ip}:{port}") })
        .collect();

    Json(Info { computer, port, addresses })
}

#[derive(Deserialize)]
struct EchoRequest {
    text: String,
}

#[derive(Serialize)]
struct EchoResponse {
    text: String,
    chars: usize,
    bytes: usize,
}

/// 받은 글자를 그대로 돌려준다. 한글이 서버를 거쳐도 깨지지 않는지 확인용.
async fn echo(Json(req): Json<EchoRequest>) -> Json<EchoResponse> {
    Json(EchoResponse { chars: req.text.chars().count(), bytes: req.text.len(), text: req.text })
}
