//! 여러 기능이 함께 쓰는 바탕(ADR 0023): DB 열기, 시계, 암호화, 데이터 묶음, 백업 파일, 버전.
//!
//! HTTP와 기능(`features`)을 모른다. 업무 규칙(`baeum-domain`) 말고는 아무것도 부르지 않는다.

pub mod backup_file;
pub mod clock;
pub mod data_bundle;
pub mod db;
pub mod secret;
pub mod startup;
pub mod version;
