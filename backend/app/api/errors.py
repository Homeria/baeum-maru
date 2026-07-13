"""FastAPI 예외를 안정된 공통 오류 응답으로 변환한다."""

import logging
from collections.abc import Mapping, Sequence
from typing import Any

from fastapi import FastAPI, Request
from fastapi.exceptions import RequestValidationError
from fastapi.responses import JSONResponse
from starlette.exceptions import HTTPException

from app.api.middleware import REQUEST_ID_HEADER
from app.core.exceptions import (
    ApplicationError,
    ConflictError,
    PermissionDeniedError,
    ResourceNotFoundError,
)
from app.schemas.common import ErrorBody, ErrorDetail, ErrorResponse

logger = logging.getLogger(__name__)


def request_id(request: Request) -> str:
    return str(getattr(request.state, "request_id", "unknown"))


def error_response(
    request: Request,
    *,
    status_code: int,
    code: str,
    message: str,
    details: list[ErrorDetail] | None = None,
    headers: Mapping[str, str] | None = None,
) -> JSONResponse:
    correlation_id = request_id(request)
    body = ErrorResponse(
        error=ErrorBody(
            code=code,
            message=message,
            request_id=correlation_id,
            details=details or [],
        )
    )
    response_headers = dict(headers or {})
    response_headers[REQUEST_ID_HEADER] = correlation_id
    return JSONResponse(
        status_code=status_code,
        content=body.model_dump(mode="json"),
        headers=response_headers,
    )


def validation_details(errors: Sequence[Any]) -> list[ErrorDetail]:
    details: list[ErrorDetail] = []
    for raw_error in errors:
        error = raw_error if isinstance(raw_error, Mapping) else {}
        location = error.get("loc", ())
        field = ".".join(str(part) for part in location) if location else None
        details.append(
            ErrorDetail(
                field=field,
                message=str(error.get("msg", "Invalid value")),
                type=str(error.get("type", "validation_error")),
            )
        )
    return details


def application_error_details(error: ApplicationError) -> list[ErrorDetail]:
    return [
        ErrorDetail.model_validate(dict(detail))
        for detail in error.details
        if isinstance(detail, Mapping)
    ]


def application_error_status(error: ApplicationError) -> int:
    if isinstance(error, ResourceNotFoundError):
        return 404
    if isinstance(error, ConflictError):
        return 409
    if isinstance(error, PermissionDeniedError):
        return 403
    return 400


def register_exception_handlers(app: FastAPI) -> None:
    @app.exception_handler(RequestValidationError)
    async def handle_validation_error(
        request: Request, error: RequestValidationError
    ) -> JSONResponse:
        return error_response(
            request,
            status_code=422,
            code="validation_error",
            message="요청 값이 올바르지 않습니다.",
            details=validation_details(error.errors()),
        )

    @app.exception_handler(ApplicationError)
    async def handle_application_error(request: Request, error: ApplicationError) -> JSONResponse:
        return error_response(
            request,
            status_code=application_error_status(error),
            code=error.code,
            message=error.message,
            details=application_error_details(error),
        )

    @app.exception_handler(HTTPException)
    async def handle_http_error(request: Request, error: HTTPException) -> JSONResponse:
        message = (
            str(error.detail) if isinstance(error.detail, str) else "요청을 처리할 수 없습니다."
        )
        return error_response(
            request,
            status_code=error.status_code,
            code="http_error",
            message=message,
            headers=error.headers,
        )

    @app.exception_handler(Exception)
    async def handle_unexpected_error(request: Request, error: Exception) -> JSONResponse:
        logger.exception("Unhandled API error", extra={"request_id": request_id(request)})
        return error_response(
            request,
            status_code=500,
            code="internal_error",
            message="서버에서 요청을 처리하지 못했습니다.",
        )
