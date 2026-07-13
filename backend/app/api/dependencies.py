"""FastAPI router가 공유하는 DB Session과 pagination 의존성."""

from collections.abc import Iterator
from typing import Annotated

from fastapi import Query, Request
from sqlalchemy.orm import Session, sessionmaker

from app.schemas.common import PaginationParams


def get_db(request: Request) -> Iterator[Session]:
    """요청마다 Session을 열고 응답 후 닫는다. commit 여부는 service가 결정한다."""
    factory: sessionmaker[Session] = request.app.state.session_factory
    with factory() as session:
        yield session


def get_pagination(
    page: Annotated[int, Query(ge=1)] = 1,
    page_size: Annotated[int, Query(ge=1, le=100)] = 20,
) -> PaginationParams:
    return PaginationParams(page=page, page_size=page_size)
