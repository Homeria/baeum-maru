"""강좌 기준 정보(분류·난도·강사·학기·교시) 데이터를 다루는 repository."""

import sqlite3
from typing import Any

from app.core.exceptions import ConflictError
from app.db.database import get_db_connection


def _all(table: str, *, include_inactive: bool, order: str) -> list[dict[str, Any]]:
    query = f"SELECT * FROM {table}"
    if not include_inactive:
        query += " WHERE is_active = 1"
    query += f" ORDER BY {order}"
    with get_db_connection() as conn:
        rows = conn.execute(query).fetchall()
    return [dict(row) for row in rows]


def _one(table: str, row_id: int) -> dict[str, Any] | None:
    with get_db_connection() as conn:
        row = conn.execute(f"SELECT * FROM {table} WHERE id = ?", (row_id,)).fetchone()
    return dict(row) if row is not None else None


# --- course_categories (분류) ---


def list_categories(*, include_inactive: bool = False) -> list[dict[str, Any]]:
    return _all("course_categories", include_inactive=include_inactive, order="sort_order, name")


def get_category(category_id: int) -> dict[str, Any] | None:
    return _one("course_categories", category_id)


def create_category(*, name: str, sort_order: int) -> dict[str, Any]:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute(
                "INSERT INTO course_categories (name, sort_order) VALUES (?, ?)",
                (name, sort_order),
            )
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError(
                "category_name_exists", "이미 같은 이름의 분류가 있습니다."
            ) from error
        row = conn.execute(
            "SELECT * FROM course_categories WHERE id = ?", (cursor.lastrowid,)
        ).fetchone()
    return dict(row)


def update_category(
    category_id: int, *, name: str, sort_order: int, is_active: bool
) -> dict[str, Any] | None:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute(
                """
                UPDATE course_categories
                SET name = ?, sort_order = ?, is_active = ?, updated_at = CURRENT_TIMESTAMP
                WHERE id = ?
                """,
                (name, sort_order, int(is_active), category_id),
            )
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError(
                "category_name_exists", "이미 같은 이름의 분류가 있습니다."
            ) from error
        if cursor.rowcount == 0:
            return None
        row = conn.execute(
            "SELECT * FROM course_categories WHERE id = ?", (category_id,)
        ).fetchone()
    return dict(row)


def delete_category(category_id: int) -> bool:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute("DELETE FROM course_categories WHERE id = ?", (category_id,))
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError(
                "category_in_use", "이 분류를 쓰는 강좌가 있어 삭제할 수 없습니다. 비활성화하세요."
            ) from error
    return cursor.rowcount > 0


# --- course_levels (난도) ---


def list_levels(*, include_inactive: bool = False) -> list[dict[str, Any]]:
    return _all("course_levels", include_inactive=include_inactive, order="sort_order, name")


def get_level(level_id: int) -> dict[str, Any] | None:
    return _one("course_levels", level_id)


def create_level(*, name: str, sort_order: int) -> dict[str, Any]:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute(
                "INSERT INTO course_levels (name, sort_order) VALUES (?, ?)",
                (name, sort_order),
            )
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError("level_name_exists", "이미 같은 이름의 난도가 있습니다.") from error
        row = conn.execute(
            "SELECT * FROM course_levels WHERE id = ?", (cursor.lastrowid,)
        ).fetchone()
    return dict(row)


def update_level(
    level_id: int, *, name: str, sort_order: int, is_active: bool
) -> dict[str, Any] | None:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute(
                """
                UPDATE course_levels
                SET name = ?, sort_order = ?, is_active = ?, updated_at = CURRENT_TIMESTAMP
                WHERE id = ?
                """,
                (name, sort_order, int(is_active), level_id),
            )
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError("level_name_exists", "이미 같은 이름의 난도가 있습니다.") from error
        if cursor.rowcount == 0:
            return None
        row = conn.execute("SELECT * FROM course_levels WHERE id = ?", (level_id,)).fetchone()
    return dict(row)


def delete_level(level_id: int) -> bool:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute("DELETE FROM course_levels WHERE id = ?", (level_id,))
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError(
                "level_in_use", "이 난도를 쓰는 강좌가 있어 삭제할 수 없습니다. 비활성화하세요."
            ) from error
    return cursor.rowcount > 0


# --- instructors (강사) ---


def list_instructors(*, include_inactive: bool = False) -> list[dict[str, Any]]:
    return _all("instructors", include_inactive=include_inactive, order="name")


def get_instructor(instructor_id: int) -> dict[str, Any] | None:
    return _one("instructors", instructor_id)


def create_instructor(*, name: str, phone: str | None) -> dict[str, Any]:
    with get_db_connection() as conn:
        cursor = conn.execute("INSERT INTO instructors (name, phone) VALUES (?, ?)", (name, phone))
        conn.commit()
        row = conn.execute("SELECT * FROM instructors WHERE id = ?", (cursor.lastrowid,)).fetchone()
    return dict(row)


def update_instructor(
    instructor_id: int, *, name: str, phone: str | None, is_active: bool
) -> dict[str, Any] | None:
    with get_db_connection() as conn:
        cursor = conn.execute(
            """
            UPDATE instructors
            SET name = ?, phone = ?, is_active = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            """,
            (name, phone, int(is_active), instructor_id),
        )
        conn.commit()
        if cursor.rowcount == 0:
            return None
        row = conn.execute("SELECT * FROM instructors WHERE id = ?", (instructor_id,)).fetchone()
    return dict(row)


def delete_instructor(instructor_id: int) -> bool:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute("DELETE FROM instructors WHERE id = ?", (instructor_id,))
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError(
                "instructor_in_use", "개설 강좌에 배정돼 있어 삭제할 수 없습니다. 비활성화하세요."
            ) from error
    return cursor.rowcount > 0


# --- terms (학기) ---


def list_terms() -> list[dict[str, Any]]:
    with get_db_connection() as conn:
        rows = conn.execute("SELECT * FROM terms ORDER BY starts_on DESC, name").fetchall()
    return [dict(row) for row in rows]


def get_term(term_id: int) -> dict[str, Any] | None:
    return _one("terms", term_id)


def create_term(
    *,
    name: str,
    starts_on: str | None,
    ends_on: str | None,
    registration_opens_at: str | None,
    registration_closes_at: str | None,
    max_registrations_per_member: int,
    status: str,
) -> dict[str, Any]:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute(
                """
                INSERT INTO terms (
                    name, starts_on, ends_on, registration_opens_at,
                    registration_closes_at, max_registrations_per_member, status
                ) VALUES (?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    name,
                    starts_on,
                    ends_on,
                    registration_opens_at,
                    registration_closes_at,
                    max_registrations_per_member,
                    status,
                ),
            )
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError("term_name_exists", "이미 같은 이름의 학기가 있습니다.") from error
        row = conn.execute("SELECT * FROM terms WHERE id = ?", (cursor.lastrowid,)).fetchone()
    return dict(row)


def update_term(
    term_id: int,
    *,
    name: str,
    starts_on: str | None,
    ends_on: str | None,
    registration_opens_at: str | None,
    registration_closes_at: str | None,
    max_registrations_per_member: int,
    status: str,
) -> dict[str, Any] | None:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute(
                """
                UPDATE terms
                SET name = ?, starts_on = ?, ends_on = ?, registration_opens_at = ?,
                    registration_closes_at = ?, max_registrations_per_member = ?, status = ?,
                    updated_at = CURRENT_TIMESTAMP
                WHERE id = ?
                """,
                (
                    name,
                    starts_on,
                    ends_on,
                    registration_opens_at,
                    registration_closes_at,
                    max_registrations_per_member,
                    status,
                    term_id,
                ),
            )
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError("term_name_exists", "이미 같은 이름의 학기가 있습니다.") from error
        if cursor.rowcount == 0:
            return None
        row = conn.execute("SELECT * FROM terms WHERE id = ?", (term_id,)).fetchone()
    return dict(row)


def delete_term(term_id: int) -> bool:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute("DELETE FROM terms WHERE id = ?", (term_id,))
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError(
                "term_in_use", "개설 강좌나 추첨이 있어 삭제할 수 없습니다."
            ) from error
    return cursor.rowcount > 0


# --- time_slots (교시) ---


def list_time_slots(*, include_inactive: bool = False) -> list[dict[str, Any]]:
    return _all("time_slots", include_inactive=include_inactive, order="sort_order, start_time")


def get_time_slot(time_slot_id: int) -> dict[str, Any] | None:
    return _one("time_slots", time_slot_id)


def create_time_slot(
    *, name: str, start_time: str, end_time: str, sort_order: int
) -> dict[str, Any]:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute(
                "INSERT INTO time_slots (name, start_time, end_time, sort_order) "
                "VALUES (?, ?, ?, ?)",
                (name, start_time, end_time, sort_order),
            )
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError(
                "time_slot_exists", "이미 등록된 교시(이름 또는 시간)입니다."
            ) from error
        row = conn.execute("SELECT * FROM time_slots WHERE id = ?", (cursor.lastrowid,)).fetchone()
    return dict(row)


def update_time_slot(
    time_slot_id: int,
    *,
    name: str,
    start_time: str,
    end_time: str,
    sort_order: int,
    is_active: bool,
) -> dict[str, Any] | None:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute(
                """
                UPDATE time_slots
                SET name = ?, start_time = ?, end_time = ?, sort_order = ?, is_active = ?,
                    updated_at = CURRENT_TIMESTAMP
                WHERE id = ?
                """,
                (name, start_time, end_time, sort_order, int(is_active), time_slot_id),
            )
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError(
                "time_slot_exists", "이미 등록된 교시(이름 또는 시간)입니다."
            ) from error
        if cursor.rowcount == 0:
            return None
        row = conn.execute("SELECT * FROM time_slots WHERE id = ?", (time_slot_id,)).fetchone()
    return dict(row)


def delete_time_slot(time_slot_id: int) -> bool:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute("DELETE FROM time_slots WHERE id = ?", (time_slot_id,))
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError(
                "time_slot_in_use", "시간표에 배정돼 있어 삭제할 수 없습니다. 비활성화하세요."
            ) from error
    return cursor.rowcount > 0
