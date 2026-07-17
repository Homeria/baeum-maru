"""Excel, backup, 감사 로그와 장시간 작업 요청을 조정하는 service."""

import math
import re
from collections.abc import Mapping, Sequence
from typing import Any, Literal

from app.repositories.operation_repository import AuditLogRecord, add_audit_log

ActorKind = Literal["user", "launcher", "system"]
_ACTOR_KINDS = frozenset({"user", "launcher", "system"})
_SENSITIVE_METADATA_KEY = re.compile(
    r"password|token|secret|authorization|cookie|access[_-]?code|phone|birth",
    re.IGNORECASE,
)


def _required_text(value: str, field: str, max_length: int | None = None) -> str:
    normalized = value.strip()
    if not normalized:
        raise ValueError(f"{field} must not be empty")
    if max_length is not None and len(normalized) > max_length:
        raise ValueError(f"{field} must be at most {max_length} characters")
    return normalized


def _optional_text(value: str | None, field: str, max_length: int) -> str | None:
    if value is None:
        return None
    normalized = value.strip()
    if not normalized:
        return None
    if len(normalized) > max_length:
        raise ValueError(f"{field} must be at most {max_length} characters")
    return normalized


def _audit_json(value: Any, path: str) -> Any:
    if value is None or isinstance(value, (str, bool, int)):
        return value
    if isinstance(value, float):
        if not math.isfinite(value):
            raise ValueError(f"{path} must contain a finite number")
        return value
    if isinstance(value, Mapping):
        result: dict[str, Any] = {}
        for raw_key, item in value.items():
            if not isinstance(raw_key, str):
                raise ValueError(f"{path} keys must be strings")
            if _SENSITIVE_METADATA_KEY.search(raw_key):
                raise ValueError(f"{path}.{raw_key} is not allowed in audit metadata")
            result[raw_key] = _audit_json(item, f"{path}.{raw_key}")
        return result
    if isinstance(value, Sequence) and not isinstance(value, (str, bytes, bytearray)):
        return [_audit_json(item, f"{path}[]") for item in value]
    raise ValueError(f"{path} must contain JSON-compatible values")


def record_audit(
    *,
    actor_kind: ActorKind,
    action: str,
    resource_type: str,
    summary: str,
    actor_user_id: int | None = None,
    actor_access_code_id: int | None = None,
    actor_display_name: str | None = None,
    resource_id: str | None = None,
    request_id: str | None = None,
    metadata: Mapping[str, Any] | None = None,
) -> AuditLogRecord:
    """감사 값을 검증한 뒤 Repository가 소유한 transaction으로 저장한다."""
    if actor_kind not in _ACTOR_KINDS:
        raise ValueError("actor_kind is invalid")
    if actor_kind == "user" and actor_user_id is None:
        raise ValueError("user actor requires actor_user_id")
    if actor_kind != "user" and (actor_user_id is not None or actor_access_code_id is not None):
        raise ValueError("launcher and system actors cannot reference user credentials")

    metadata_json = _audit_json(metadata, "metadata") if metadata is not None else None
    if metadata_json is not None and not isinstance(metadata_json, dict):
        raise ValueError("metadata must be an object")

    return add_audit_log(
        actor_kind=actor_kind,
        actor_user_id=actor_user_id,
        actor_access_code_id=actor_access_code_id,
        actor_display_name=_optional_text(actor_display_name, "actor_display_name", 80),
        action=_required_text(action, "action", 80),
        resource_type=_required_text(resource_type, "resource_type", 80),
        resource_id=_optional_text(resource_id, "resource_id", 80),
        summary=_required_text(summary, "summary"),
        request_id=_optional_text(request_id, "request_id", 64),
        metadata_json=metadata_json,
    )
