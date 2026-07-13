"""FastAPI router가 공유하는 DB Session과 pagination 의존성."""

from collections.abc import Callable, Iterator
from typing import Annotated
from urllib.parse import urlsplit

from fastapi import Query, Request, WebSocket, WebSocketException, status
from sqlalchemy.orm import Session, sessionmaker

from app.schemas.common import PaginationParams

SESSION_COOKIE_NAME = "baeum_maru_session"
type RealtimeSessionVerifier = Callable[[str], str | None]


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


def require_realtime_session(websocket: WebSocket) -> str:
    """HttpOnly session cookie를 검증한다. 실제 verifier는 인증 도메인이 제공한다."""
    origin = websocket.headers.get("origin")
    host = websocket.headers.get("host")
    try:
        origin_url = urlsplit(origin or "")
        same_origin = (
            origin_url.scheme in {"http", "https"}
            and host is not None
            and origin_url.netloc.casefold() == host.casefold()
        )
    except ValueError:
        same_origin = False

    token = websocket.cookies.get(SESSION_COOKIE_NAME)
    verifier: RealtimeSessionVerifier | None = websocket.app.state.realtime_session_verifier
    session_id = verifier(token) if token is not None and verifier is not None else None
    if not same_origin or session_id is None:
        raise WebSocketException(
            code=status.WS_1008_POLICY_VIOLATION,
            reason="Authentication required",
        )
    return session_id
