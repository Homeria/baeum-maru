//! 기능별 폴더 (ADR 0023). 기능마다 `handler`(요청 처리, 주소 등록), `repository`(SQL), `dto`(요청·응답 모양),
//! 필요하면 `service`(트랜잭션과 규칙 엮기)를 둔다.
//!
//! 기능끼리 부르는 방향은 ADR 0023의 의존 방향 표에 있는 것만 쓴다.

pub(crate) mod access_logs;
pub(crate) mod auth;
pub(crate) mod courses;
pub(crate) mod devices;
pub(crate) mod members;
pub(crate) mod priority_reasons;
pub(crate) mod rooms;
pub(crate) mod setup;
pub(crate) mod staff;
pub(crate) mod subjects;
pub(crate) mod terms;
