//! 백업(ADR 0011): DB를 통째로 복사한 파일을 쌓고 오래된 것을 정리한다.
//!
//! 백업마다 파일 하나다. 언제, 왜 만들었는지는 파일 이름에 적어 두고 다시 읽는다.
//! 그래서 목록이나 마지막 복사 시각을 따로 저장하지 않는다.
#![cfg_attr(not(test), expect(dead_code, reason = "백업 만들기를 붙이기 전"))]

pub mod name;
pub mod retention;
