"""페이지네이션과 공통 오류에 사용하는 API schema."""

from __future__ import annotations

from pydantic import BaseModel, Field


class ErrorDetail(BaseModel):
    field: str | None = None
    message: str
    type: str | None = None


class ErrorBody(BaseModel):
    code: str
    message: str
    request_id: str
    details: list[ErrorDetail] = Field(default_factory=list)


class ErrorResponse(BaseModel):
    error: ErrorBody


class PaginationParams(BaseModel):
    page: int = Field(default=1, ge=1)
    page_size: int = Field(default=20, ge=1, le=100)

    @property
    def offset(self) -> int:
        return (self.page - 1) * self.page_size


class PageMetadata(BaseModel):
    page: int
    page_size: int
    total_items: int = Field(ge=0)
    total_pages: int = Field(ge=0)

    @classmethod
    def from_total(cls, pagination: PaginationParams, total_items: int) -> PageMetadata:
        total_pages = (
            (total_items + pagination.page_size - 1) // pagination.page_size if total_items else 0
        )
        return cls(
            page=pagination.page,
            page_size=pagination.page_size,
            total_items=total_items,
            total_pages=total_pages,
        )


class PageResponse[ItemT](BaseModel):
    items: list[ItemT]
    meta: PageMetadata
