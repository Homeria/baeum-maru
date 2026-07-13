"""HTTP에 의존하지 않는 애플리케이션 공통 예외."""

from collections.abc import Mapping
from typing import Any


class ApplicationError(Exception):
    """사용자에게 안정된 코드와 메시지를 전달할 수 있는 업무 예외."""

    def __init__(
        self,
        code: str,
        message: str,
        *,
        details: list[Mapping[str, Any]] | None = None,
    ) -> None:
        super().__init__(message)
        self.code = code
        self.message = message
        self.details = details or []


class ResourceNotFoundError(ApplicationError):
    """요청한 업무 리소스를 찾을 수 없는 경우."""


class ConflictError(ApplicationError):
    """중복, version 불일치 등 현재 상태와 요청이 충돌한 경우."""


class PermissionDeniedError(ApplicationError):
    """현재 인증 주체에게 작업 권한이 없는 경우."""
