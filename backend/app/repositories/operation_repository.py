"""감사 로그, 작업 상태, 멱등성 key와 operation lock 데이터를 다루는 repository."""

from typing import Any

from sqlalchemy.orm import Session

from app.models.operations import AuditLog


def add_audit_log(
    session: Session,
    *,
    actor_kind: str,
    actor_user_id: int | None,
    actor_access_code_id: int | None,
    actor_display_name: str | None,
    action: str,
    resource_type: str,
    resource_id: str | None,
    summary: str,
    request_id: str | None,
    metadata_json: dict[str, Any] | None,
) -> AuditLog:
    """현재 transaction에 감사 row를 추가한다. commit은 호출 service가 수행한다."""
    audit_log = AuditLog(
        actor_kind=actor_kind,
        actor_user_id=actor_user_id,
        actor_access_code_id=actor_access_code_id,
        actor_display_name=actor_display_name,
        action=action,
        resource_type=resource_type,
        resource_id=resource_id,
        summary=summary,
        request_id=request_id,
        metadata_json=metadata_json,
    )
    session.add(audit_log)
    session.flush()
    return audit_log
