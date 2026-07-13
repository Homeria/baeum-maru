"""요청별 correlation ID를 검증하고 응답 header에 포함한다."""

import re
from collections.abc import Awaitable, Callable
from uuid import uuid4

from fastapi import Request, Response

REQUEST_ID_HEADER = "X-Request-ID"
_REQUEST_ID_PATTERN = re.compile(r"^[A-Za-z0-9._-]{1,64}$")


def request_id_from_header(value: str | None) -> str:
    if value and _REQUEST_ID_PATTERN.fullmatch(value):
        return value
    return str(uuid4())


async def request_id_middleware(
    request: Request,
    call_next: Callable[[Request], Awaitable[Response]],
) -> Response:
    request_id = request_id_from_header(request.headers.get(REQUEST_ID_HEADER))
    request.state.request_id = request_id
    response = await call_next(request)
    response.headers[REQUEST_ID_HEADER] = request_id
    return response
