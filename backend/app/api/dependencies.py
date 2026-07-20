"""FastAPI router가 공유하는 pagination과 인증 의존성."""

from collections.abc import Callable
from typing import Annotated, Any
from urllib.parse import urlsplit

from fastapi import Query, Request, WebSocket, WebSocketException, status

import app.services.auth_service as auth_service
from app.core.exceptions import AuthenticationError
from app.core.settings import AppSettings
from app.schemas.common import PaginationParams

SESSION_COOKIE_NAME = "baeum_maru_session"
type RealtimeSessionVerifier = Callable[[str], str | None]


def get_pagination(
    page: Annotated[int, Query(ge=1)] = 1,
    page_size: Annotated[int, Query(ge=1, le=100)] = 20,
) -> PaginationParams:
    return PaginationParams(page=page, page_size=page_size)


def get_settings(request: Request) -> AppSettings:
    """lifespan에서 검증해 app.state에 둔 설정을 라우터에 전달한다."""
    return request.app.state.settings  # type: ignore[no-any-return]


def get_current_operator(request: Request) -> dict[str, Any]:
    """세션 쿠키를 검증해 현재 관계자를 돌려주고, 없으면 401을 낸다."""
    token = request.cookies.get(SESSION_COOKIE_NAME)
    operator = auth_service.verify_session(token) if token else None
    if operator is None:
        raise AuthenticationError("authentication_required", "로그인이 필요합니다.")
    return operator


def default_realtime_session_verifier(token: str) -> str | None:
    """실시간 WebSocket이 세션 쿠키를 검증할 때 쓰는 기본 verifier."""
    operator = auth_service.verify_session(token)
    return str(operator["session_id"]) if operator is not None else None


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
