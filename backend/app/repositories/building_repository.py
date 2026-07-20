"""건물과 층 데이터를 다루는 repository."""

import sqlite3
from typing import Any

from app.core.exceptions import ConflictError
from app.db.database import get_db_connection

# CRUD - Read (buildings)


def list_buildings(*, include_inactive: bool = False) -> list[dict[str, Any]]:
    query = "SELECT * FROM buildings"
    if not include_inactive:
        query += " WHERE is_active = 1"
    query += " ORDER BY sort_order, name"
    with get_db_connection() as conn:
        rows = conn.execute(query).fetchall()
    return [dict(row) for row in rows]


def get_building(building_id: int) -> dict[str, Any] | None:
    with get_db_connection() as conn:
        row = conn.execute("SELECT * FROM buildings WHERE id = ?", (building_id,)).fetchone()
    return dict(row) if row is not None else None


# CRUD - Create/Update/Delete (buildings)


def create_building(*, name: str, description: str | None, sort_order: int) -> dict[str, Any]:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute(
                "INSERT INTO buildings (name, description, sort_order) VALUES (?, ?, ?)",
                (name, description, sort_order),
            )
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError(
                "building_name_exists", "이미 같은 이름의 건물이 있습니다."
            ) from error
        row = conn.execute("SELECT * FROM buildings WHERE id = ?", (cursor.lastrowid,)).fetchone()
    return dict(row)


def update_building(
    building_id: int,
    *,
    name: str,
    description: str | None,
    sort_order: int,
    is_active: bool,
) -> dict[str, Any] | None:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute(
                """
                UPDATE buildings
                SET name = ?, description = ?, sort_order = ?, is_active = ?,
                    updated_at = CURRENT_TIMESTAMP
                WHERE id = ?
                """,
                (name, description, sort_order, int(is_active), building_id),
            )
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError(
                "building_name_exists", "이미 같은 이름의 건물이 있습니다."
            ) from error
        if cursor.rowcount == 0:
            return None
        row = conn.execute("SELECT * FROM buildings WHERE id = ?", (building_id,)).fetchone()
    return dict(row)


def delete_building(building_id: int) -> bool:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute("DELETE FROM buildings WHERE id = ?", (building_id,))
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError(
                "building_in_use", "층이나 장소가 남아 있어 삭제할 수 없습니다. 비활성화하세요."
            ) from error
    return cursor.rowcount > 0


# CRUD - Read (building_floors)


def list_floors(building_id: int, *, include_inactive: bool = False) -> list[dict[str, Any]]:
    query = "SELECT * FROM building_floors WHERE building_id = ?"
    if not include_inactive:
        query += " AND is_active = 1"
    query += " ORDER BY sort_order, label"
    with get_db_connection() as conn:
        rows = conn.execute(query, (building_id,)).fetchall()
    return [dict(row) for row in rows]


def get_floor(floor_id: int) -> dict[str, Any] | None:
    with get_db_connection() as conn:
        row = conn.execute("SELECT * FROM building_floors WHERE id = ?", (floor_id,)).fetchone()
    return dict(row) if row is not None else None


# CRUD - Create/Update/Delete (building_floors)


def create_floor(*, building_id: int, label: str, sort_order: int) -> dict[str, Any]:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute(
                "INSERT INTO building_floors (building_id, label, sort_order) VALUES (?, ?, ?)",
                (building_id, label, sort_order),
            )
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError(
                "floor_label_exists", "이 건물에 같은 이름의 층이 있습니다."
            ) from error
        row = conn.execute(
            "SELECT * FROM building_floors WHERE id = ?", (cursor.lastrowid,)
        ).fetchone()
    return dict(row)


def update_floor(
    floor_id: int, *, label: str, sort_order: int, is_active: bool
) -> dict[str, Any] | None:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute(
                """
                UPDATE building_floors
                SET label = ?, sort_order = ?, is_active = ?, updated_at = CURRENT_TIMESTAMP
                WHERE id = ?
                """,
                (label, sort_order, int(is_active), floor_id),
            )
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError(
                "floor_label_exists", "이 건물에 같은 이름의 층이 있습니다."
            ) from error
        if cursor.rowcount == 0:
            return None
        row = conn.execute("SELECT * FROM building_floors WHERE id = ?", (floor_id,)).fetchone()
    return dict(row)


def delete_floor(floor_id: int) -> bool:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute("DELETE FROM building_floors WHERE id = ?", (floor_id,))
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError(
                "floor_in_use", "장소가 남아 있어 삭제할 수 없습니다. 비활성화하세요."
            ) from error
    return cursor.rowcount > 0
