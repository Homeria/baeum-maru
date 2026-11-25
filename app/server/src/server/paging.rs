//! 목록을 쪽으로 나눌 때 쪽 번호와 한 쪽 개수 확인. 회원 찾기, 접속 기록이 함께 쓴다.

use super::error::{ApiError, FieldError};

/// 한 쪽에 보여 주는 최대 개수.
pub const MAX_PAGE_SIZE: u32 = 100;

/// 확인을 통과한 쪽 번호(1부터)와 한 쪽 개수(1~[`MAX_PAGE_SIZE`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Paging {
    pub page: u32,
    pub page_size: u32,
}

impl Paging {
    /// `?page=&pageSize=` 값을 확인한다. 없으면 1쪽, `default_size`개.
    ///
    /// # Errors
    ///
    /// 틀린 칸을 모두 모아 [`ApiError::InvalidInput`]. 칸 이름은 `page`, `pageSize`.
    pub fn parse(
        page: Option<u32>,
        page_size: Option<u32>,
        default_size: u32,
    ) -> Result<Self, ApiError> {
        let page = page.unwrap_or(1);
        let page_size = page_size.unwrap_or(default_size);

        let mut fields = Vec::new();
        if page < 1 {
            fields.push(FieldError {
                field: "page".to_owned(),
                message: "쪽 번호는 1부터입니다".to_owned(),
            });
        }
        if !(1..=MAX_PAGE_SIZE).contains(&page_size) {
            fields.push(FieldError {
                field: "pageSize".to_owned(),
                message: format!("한 쪽 개수는 1~{MAX_PAGE_SIZE}개입니다"),
            });
        }
        if fields.is_empty() {
            Ok(Paging { page, page_size })
        } else {
            Err(ApiError::InvalidInput(fields))
        }
    }
}
