//! 처음 설정 (BR-K11). 확인과 저장은 `service`가 한 트랜잭션에서 한다.
//! 입력 확인 오류의 칸 이름은 직원 기능에서 가져온다 (ADR 0023).

mod dto;
pub(crate) mod handler;
pub(crate) mod repository;
pub(crate) mod service;
#[cfg(test)]
mod tests;
