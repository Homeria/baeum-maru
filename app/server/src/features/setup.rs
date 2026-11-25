//! 처음 설정 (BR-K11). 단순한 기능이라 service 없이 handler와 repository만 둔다.
//! 입력 확인 오류의 칸 이름은 직원 기능에서 가져온다 (ADR 0023).

mod dto;
pub(crate) mod handler;
pub(crate) mod repository;
#[cfg(test)]
mod tests;
