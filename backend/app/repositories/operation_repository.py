"""감사 로그, 작업 상태, 멱등성 key와 operation lock 데이터를 다루는 repository."""

import json
import sqlite3
from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True, slots=True)
class AuditLogRecord:
    """audit_logs 한 행을 표현하는 읽기 전용 값 객체."""

    id: int
    actor_kind: str
    actor_user_id: int | None
    actor_access_code_id: int | None
    actor_display_name: str | None
    action: str
    resource_type: str
    resource_id: str | None
    summary: str
    request_id: str | None
    metadata_json: dict[str, Any] | None
    created_at: str


def add_audit_log(
    connection: sqlite3.Connection,
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
) -> AuditLogRecord:
    """현재 transaction에 감사 row를 추가한다. commit은 호출 service가 수행한다."""
    cursor = connection.execute(
        """
        INSERT INTO audit_logs (
            actor_kind,
            actor_user_id,
            actor_access_code_id,
            actor_display_name,
            action,
            resource_type,
            resource_id,
            summary,
            request_id,
            metadata_json
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            actor_kind,
            actor_user_id,
            actor_access_code_id,
            actor_display_name,
            action,
            resource_type,
            resource_id,
            summary,
            request_id,
            json.dumps(metadata_json, ensure_ascii=False) if metadata_json is not None else None,
        ),
    )
    audit_log_id = cursor.lastrowid
    if audit_log_id is None:
        raise RuntimeError("SQLite did not return an audit log id")

    row = connection.execute(
        """
        SELECT id, actor_kind, actor_user_id, actor_access_code_id, actor_display_name,
               action, resource_type, resource_id, summary, request_id,
               metadata_json, created_at
        FROM audit_logs
        WHERE id = ?
        """,
        (audit_log_id,),
    ).fetchone()
    if row is None:
        raise RuntimeError("Inserted audit log could not be read")

    return AuditLogRecord(
        id=int(row["id"]),
        actor_kind=str(row["actor_kind"]),
        actor_user_id=row["actor_user_id"],
        actor_access_code_id=row["actor_access_code_id"],
        actor_display_name=row["actor_display_name"],
        action=str(row["action"]),
        resource_type=str(row["resource_type"]),
        resource_id=row["resource_id"],
        summary=str(row["summary"]),
        request_id=row["request_id"],
        metadata_json=json.loads(row["metadata_json"]) if row["metadata_json"] else None,
        created_at=str(row["created_at"]),
    )
