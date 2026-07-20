"""감사 로그(audit_logs) 데이터를 다루는 repository."""

import json
import sqlite3
from typing import Any

from app.db.database import get_db_connection


def insert_audit_log(
    connection: sqlite3.Connection,
    *,
    actor_kind: str,
    actor_operator_id: int | None,
    actor_access_code_id: int | None,
    actor_display_name: str | None,
    action: str,
    resource_type: str,
    resource_id: str | None,
    summary: str,
    request_id: str | None,
    metadata_json: dict[str, Any] | None,
) -> dict[str, Any]:
    """다른 repository가 연 transaction에 감사 row를 함께 추가한다(성공한 쓰기용)."""
    cursor = connection.execute(
        """
        INSERT INTO audit_logs (
            actor_kind, actor_operator_id, actor_access_code_id, actor_display_name,
            action, resource_type, resource_id, summary, request_id, metadata_json
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            actor_kind,
            actor_operator_id,
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
    row = connection.execute(
        "SELECT * FROM audit_logs WHERE id = ?", (cursor.lastrowid,)
    ).fetchone()
    return dict(row)


def add_audit_log(
    *,
    actor_kind: str,
    actor_operator_id: int | None,
    actor_access_code_id: int | None,
    actor_display_name: str | None,
    action: str,
    resource_type: str,
    resource_id: str | None,
    summary: str,
    request_id: str | None,
    metadata_json: dict[str, Any] | None,
) -> dict[str, Any]:
    """독립적인 감사 기록을 이 repository가 소유한 transaction으로 저장한다(읽기·실패용)."""
    with get_db_connection() as conn:
        record = insert_audit_log(
            conn,
            actor_kind=actor_kind,
            actor_operator_id=actor_operator_id,
            actor_access_code_id=actor_access_code_id,
            actor_display_name=actor_display_name,
            action=action,
            resource_type=resource_type,
            resource_id=resource_id,
            summary=summary,
            request_id=request_id,
            metadata_json=metadata_json,
        )
        conn.commit()
    return record
