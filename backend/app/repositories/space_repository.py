"""장소 유형과 장소 데이터를 다루는 repository."""

import sqlite3
from typing import Any

from app.core.exceptions import ConflictError
from app.db.database import get_db_connection

# CRUD - Read (space_types)


def list_space_types(*, include_inactive: bool = False) -> list[dict[str, Any]]:
    query = "SELECT * FROM space_types"
    if not include_inactive:
        query += " WHERE is_active = 1"
    query += " ORDER BY sort_order, name"
    with get_db_connection() as conn:
        rows = conn.execute(query).fetchall()
    return [dict(row) for row in rows]


def get_space_type(space_type_id: int) -> dict[str, Any] | None:
    with get_db_connection() as conn:
        row = conn.execute("SELECT * FROM space_types WHERE id = ?", (space_type_id,)).fetchone()
    return dict(row) if row is not None else None


# CRUD - Create/Update/Delete (space_types)


def create_space_type(*, name: str, is_course_eligible: bool, sort_order: int) -> dict[str, Any]:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute(
                "INSERT INTO space_types (name, is_course_eligible, sort_order) VALUES (?, ?, ?)",
                (name, int(is_course_eligible), sort_order),
            )
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError(
                "space_type_name_exists", "이미 같은 이름의 장소 유형이 있습니다."
            ) from error
        row = conn.execute("SELECT * FROM space_types WHERE id = ?", (cursor.lastrowid,)).fetchone()
    return dict(row)


def update_space_type(
    space_type_id: int,
    *,
    name: str,
    is_course_eligible: bool,
    sort_order: int,
    is_active: bool,
) -> dict[str, Any] | None:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute(
                """
                UPDATE space_types
                SET name = ?, is_course_eligible = ?, sort_order = ?, is_active = ?,
                    updated_at = CURRENT_TIMESTAMP
                WHERE id = ?
                """,
                (name, int(is_course_eligible), sort_order, int(is_active), space_type_id),
            )
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError(
                "space_type_name_exists", "이미 같은 이름의 장소 유형이 있습니다."
            ) from error
        if cursor.rowcount == 0:
            return None
        row = conn.execute("SELECT * FROM space_types WHERE id = ?", (space_type_id,)).fetchone()
    return dict(row)


def delete_space_type(space_type_id: int) -> bool:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute("DELETE FROM space_types WHERE id = ?", (space_type_id,))
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError(
                "space_type_in_use",
                "이 유형을 쓰는 장소가 있어 삭제할 수 없습니다. 비활성화하세요.",
            ) from error
    return cursor.rowcount > 0


# CRUD - Read (spaces)


def list_spaces(
    *, building_floor_id: int | None = None, include_inactive: bool = False
) -> list[dict[str, Any]]:
    query = "SELECT * FROM spaces"
    conditions: list[str] = []
    params: list[Any] = []
    if building_floor_id is not None:
        conditions.append("building_floor_id = ?")
        params.append(building_floor_id)
    if not include_inactive:
        conditions.append("is_active = 1")
    if conditions:
        query += " WHERE " + " AND ".join(conditions)
    query += " ORDER BY sort_order, name"
    with get_db_connection() as conn:
        rows = conn.execute(query, params).fetchall()
    return [dict(row) for row in rows]


def get_space(space_id: int) -> dict[str, Any] | None:
    with get_db_connection() as conn:
        row = conn.execute("SELECT * FROM spaces WHERE id = ?", (space_id,)).fetchone()
    return dict(row) if row is not None else None


# CRUD - Create/Update/Delete (spaces)


def create_space(
    *, building_floor_id: int, space_type_id: int, name: str, sort_order: int
) -> dict[str, Any]:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute(
                """
                INSERT INTO spaces (building_floor_id, space_type_id, name, sort_order)
                VALUES (?, ?, ?, ?)
                """,
                (building_floor_id, space_type_id, name, sort_order),
            )
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError(
                "space_name_exists", "이 층에 같은 이름의 장소가 있습니다."
            ) from error
        row = conn.execute("SELECT * FROM spaces WHERE id = ?", (cursor.lastrowid,)).fetchone()
    return dict(row)


def update_space(
    space_id: int,
    *,
    building_floor_id: int,
    space_type_id: int,
    name: str,
    sort_order: int,
    is_active: bool,
) -> dict[str, Any] | None:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute(
                """
                UPDATE spaces
                SET building_floor_id = ?, space_type_id = ?, name = ?, sort_order = ?,
                    is_active = ?, updated_at = CURRENT_TIMESTAMP
                WHERE id = ?
                """,
                (building_floor_id, space_type_id, name, sort_order, int(is_active), space_id),
            )
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError(
                "space_name_exists", "이 층에 같은 이름의 장소가 있습니다."
            ) from error
        if cursor.rowcount == 0:
            return None
        row = conn.execute("SELECT * FROM spaces WHERE id = ?", (space_id,)).fetchone()
    return dict(row)


def delete_space(space_id: int) -> bool:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute("DELETE FROM spaces WHERE id = ?", (space_id,))
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError(
                "space_in_use", "시간표에 배정돼 있어 삭제할 수 없습니다. 비활성화하세요."
            ) from error
    return cursor.rowcount > 0
