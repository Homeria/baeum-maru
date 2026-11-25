//! 아직 기능 폴더(`features/`)로 옮기지 않은 기능의 요청 처리 (ADR 0023).
//!
//! 요청 주소를 묶고 서버를 띄우는 일은 [`crate::router`]가 한다.

#[cfg(test)]
pub(crate) mod test_support;
