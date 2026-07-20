"""접속 코드와 로그인 세션의 수명주기를 다루는 repository."""

import sqlite3
from typing import Any

from app.core.exceptions import ConflictError
from app.db.database import get_db_connection

# --- 접속 코드 ---


def create_access_code(*, operator_id: int, code_hash: str, ttl_minutes: int) -> dict[str, Any]:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute(
                """
                INSERT INTO access_codes (operator_id, code_hash, expires_at)
                VALUES (?, ?, datetime('now', ?))
                """,
                (operator_id, code_hash, f"+{ttl_minutes} minutes"),
            )
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError("access_code_collision", "접속 코드가 충돌했습니다.") from error
        row = conn.execute(
            "SELECT * FROM access_codes WHERE id = ?", (cursor.lastrowid,)
        ).fetchone()
    return dict(row)


def list_access_codes(operator_id: int) -> list[dict[str, Any]]:
    with get_db_connection() as conn:
        rows = conn.execute(
            "SELECT * FROM access_codes WHERE operator_id = ? ORDER BY issued_at DESC",
            (operator_id,),
        ).fetchall()
    return [dict(row) for row in rows]


def get_active_access_code_by_hash(code_hash: str) -> dict[str, Any] | None:
    """폐기되지 않고 만료 전인 접속 코드를 관계자 정보와 함께 조회한다."""
    with get_db_connection() as conn:
        row = conn.execute(
            """
            SELECT a.id AS access_code_id, a.operator_id,
                   o.display_name, o.role, o.is_active
            FROM access_codes a
            JOIN operators o ON o.id = a.operator_id
            WHERE a.code_hash = ?
              AND a.revoked_at IS NULL
              AND a.expires_at > datetime('now')
            """,
            (code_hash,),
        ).fetchone()
    return dict(row) if row is not None else None


def revoke_access_code(code_id: int) -> bool:
    with get_db_connection() as conn:
        cursor = conn.execute(
            "UPDATE access_codes SET revoked_at = datetime('now') "
            "WHERE id = ? AND revoked_at IS NULL",
            (code_id,),
        )
        conn.commit()
    return cursor.rowcount > 0


# --- 세션 ---


def open_session(
    *, operator_id: int, access_code_id: int, token_hash: str, ttl_minutes: int
) -> dict[str, Any]:
    """접속 코드 사용 시각을 남기고 세션을 여는 작업을 한 transaction으로 처리한다."""
    with get_db_connection() as conn:
        try:
            conn.execute(
                "UPDATE access_codes SET last_used_at = datetime('now') WHERE id = ?",
                (access_code_id,),
            )
            cursor = conn.execute(
                """
                INSERT INTO operator_sessions (
                    operator_id, access_code_id, token_hash, expires_at, last_seen_at
                ) VALUES (?, ?, ?, datetime('now', ?), datetime('now'))
                """,
                (operator_id, access_code_id, token_hash, f"+{ttl_minutes} minutes"),
            )
            conn.commit()
        except BaseException:
            conn.rollback()
            raise
        row = conn.execute(
            "SELECT * FROM operator_sessions WHERE id = ?", (cursor.lastrowid,)
        ).fetchone()
    return dict(row)


def get_active_session_by_hash(token_hash: str) -> dict[str, Any] | None:
    """폐기·만료되지 않고 관계자가 활성인 세션을 관계자 정보와 함께 조회한다."""
    with get_db_connection() as conn:
        row = conn.execute(
            """
            SELECT s.id AS session_id, s.operator_id, s.expires_at,
                   o.display_name, o.role, o.is_active
            FROM operator_sessions s
            JOIN operators o ON o.id = s.operator_id
            WHERE s.token_hash = ?
              AND s.revoked_at IS NULL
              AND s.expires_at > datetime('now')
              AND o.is_active = 1
            """,
            (token_hash,),
        ).fetchone()
    return dict(row) if row is not None else None


def touch_session(session_id: int) -> None:
    with get_db_connection() as conn:
        conn.execute(
            "UPDATE operator_sessions SET last_seen_at = datetime('now') WHERE id = ?",
            (session_id,),
        )
        conn.commit()


def revoke_session_by_hash(token_hash: str) -> bool:
    with get_db_connection() as conn:
        cursor = conn.execute(
            "UPDATE operator_sessions SET revoked_at = datetime('now') "
            "WHERE token_hash = ? AND revoked_at IS NULL",
            (token_hash,),
        )
        conn.commit()
    return cursor.rowcount > 0
