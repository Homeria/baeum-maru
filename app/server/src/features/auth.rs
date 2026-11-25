//! 로그인, 세션, PIN 확인, 복구 코드 (BR-K1, BR-K5~K8, BR-K11~K13, ADR 0014).
//!
//! 다른 기능은 PIN 확인을 [`service::check_pin`]으로 부른다 (ADR 0023).

pub(crate) mod dto;
pub(crate) mod handler;
pub(crate) mod middleware;
pub(crate) mod repository;
pub(crate) mod service;
#[cfg(test)]
mod tests;
