//! Tauri 명령(ADR 0023): 이 PC의 화면이 부르는 앱 기능. launcher의 controller다.
//!
//! 창, 대화상자, 다시 켜기처럼 Tauri가 필요한 일만 여기서 하고, 나머지는 Tauri를 모르는
//! 모듈을 부른다. Tauri를 모르는 모듈은 이 모듈을 부르지 않는다.

pub mod handover;
pub mod run_mode;
pub mod take_over;
