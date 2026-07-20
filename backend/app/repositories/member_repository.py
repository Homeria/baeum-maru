"""회원 데이터를 다루는 repository."""

import sqlite3
from typing import Any

from app.core.exceptions import ConflictError
from app.db.database import get_db_connection

# CRUD - Read


def list_members(*, q: str | None = None, include_inactive: bool = False) -> list[dict[str, Any]]:
    query = "SELECT * FROM members"
    conditions: list[str] = []
    params: list[Any] = []
    if q:
        conditions.append("(name LIKE ? OR phone LIKE ? OR member_no LIKE ?)")
        like = f"%{q}%"
        params.extend([like, like, like])
    if not include_inactive:
        conditions.append("is_active = 1")
    if conditions:
        query += " WHERE " + " AND ".join(conditions)
    query += " ORDER BY name"
    with get_db_connection() as conn:
        rows = conn.execute(query, params).fetchall()
    return [dict(row) for row in rows]


def get_member(member_id: int) -> dict[str, Any] | None:
    with get_db_connection() as conn:
        row = conn.execute("SELECT * FROM members WHERE id = ?", (member_id,)).fetchone()
    return dict(row) if row is not None else None


# CRUD - Create/Update/Delete


def create_member(*, member_no: str, name: str, gender: str, phone: str) -> dict[str, Any]:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute(
                "INSERT INTO members (member_no, name, gender, phone) VALUES (?, ?, ?, ?)",
                (member_no, name, gender, phone),
            )
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError("member_no_exists", "이미 등록된 회원번호입니다.") from error
        row = conn.execute("SELECT * FROM members WHERE id = ?", (cursor.lastrowid,)).fetchone()
    return dict(row)


def update_member(
    member_id: int, *, name: str, gender: str, phone: str, is_active: bool
) -> dict[str, Any] | None:
    with get_db_connection() as conn:
        cursor = conn.execute(
            """
            UPDATE members
            SET name = ?, gender = ?, phone = ?, is_active = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            """,
            (name, gender, phone, int(is_active), member_id),
        )
        conn.commit()
        if cursor.rowcount == 0:
            return None
        row = conn.execute("SELECT * FROM members WHERE id = ?", (member_id,)).fetchone()
    return dict(row)


def delete_member(member_id: int) -> bool:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute("DELETE FROM members WHERE id = ?", (member_id,))
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError(
                "member_in_use", "신청 이력이 있어 삭제할 수 없습니다. 비활성화하세요."
            ) from error
    return cursor.rowcount > 0
