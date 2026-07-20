"""관계자(operator) 데이터를 다루는 repository."""

import sqlite3
from typing import Any

from app.core.exceptions import ConflictError
from app.db.database import get_db_connection

# CRUD - Read


def list_operators(*, include_inactive: bool = False) -> list[dict[str, Any]]:
    query = "SELECT * FROM operators"
    if not include_inactive:
        query += " WHERE is_active = 1"
    query += " ORDER BY display_name"
    with get_db_connection() as conn:
        rows = conn.execute(query).fetchall()
    return [dict(row) for row in rows]


def get_operator(operator_id: int) -> dict[str, Any] | None:
    with get_db_connection() as conn:
        row = conn.execute("SELECT * FROM operators WHERE id = ?", (operator_id,)).fetchone()
    return dict(row) if row is not None else None


# CRUD - Create/Update/Delete


def create_operator(*, display_name: str, role: str) -> dict[str, Any]:
    with get_db_connection() as conn:
        cursor = conn.execute(
            "INSERT INTO operators (display_name, role) VALUES (?, ?)",
            (display_name, role),
        )
        conn.commit()
        row = conn.execute("SELECT * FROM operators WHERE id = ?", (cursor.lastrowid,)).fetchone()
    return dict(row)


def update_operator(
    operator_id: int, *, display_name: str, role: str, is_active: bool
) -> dict[str, Any] | None:
    with get_db_connection() as conn:
        cursor = conn.execute(
            """
            UPDATE operators
            SET display_name = ?, role = ?, is_active = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            """,
            (display_name, role, int(is_active), operator_id),
        )
        conn.commit()
        if cursor.rowcount == 0:
            return None
        row = conn.execute("SELECT * FROM operators WHERE id = ?", (operator_id,)).fetchone()
    return dict(row)


def delete_operator(operator_id: int) -> bool:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute("DELETE FROM operators WHERE id = ?", (operator_id,))
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError(
                "operator_in_use", "발급 이력이 있어 삭제할 수 없습니다. 비활성화하세요."
            ) from error
    return cursor.rowcount > 0
