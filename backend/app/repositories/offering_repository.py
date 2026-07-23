"""개설 강좌와 시간표 데이터를 다루는 repository."""

import sqlite3
from typing import Any

from app.core.exceptions import ConflictError
from app.db.database import get_db_connection

# --- course_offerings (개설 강좌) ---


def list_offerings() -> list[dict[str, Any]]:
    with get_db_connection() as conn:
        rows = conn.execute("SELECT * FROM course_offerings ORDER BY sort_order, id").fetchall()
    return [dict(row) for row in rows]


def get_offering(offering_id: int) -> dict[str, Any] | None:
    with get_db_connection() as conn:
        row = conn.execute("SELECT * FROM course_offerings WHERE id = ?", (offering_id,)).fetchone()
    return dict(row) if row is not None else None


def create_offering(
    *,
    course_id: int,
    section_label: str | None,
    instructor_id: int | None,
    capacity_type: str,
    capacity_total: int | None,
    male_capacity: int | None,
    female_capacity: int | None,
    status: str,
    sort_order: int,
) -> dict[str, Any]:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute(
                """
                INSERT INTO course_offerings (
                    course_id, section_label, instructor_id, capacity_type,
                    capacity_total, male_capacity, female_capacity, status, sort_order
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    course_id,
                    section_label,
                    instructor_id,
                    capacity_type,
                    capacity_total,
                    male_capacity,
                    female_capacity,
                    status,
                    sort_order,
                ),
            )
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError(
                "offering_section_exists", "같은 과목에 같은 분반이 이미 있습니다."
            ) from error
        row = conn.execute(
            "SELECT * FROM course_offerings WHERE id = ?", (cursor.lastrowid,)
        ).fetchone()
    return dict(row)


def update_offering(
    offering_id: int,
    *,
    course_id: int,
    section_label: str | None,
    instructor_id: int | None,
    capacity_type: str,
    capacity_total: int | None,
    male_capacity: int | None,
    female_capacity: int | None,
    status: str,
    sort_order: int,
) -> dict[str, Any] | None:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute(
                """
                UPDATE course_offerings
                SET course_id = ?, section_label = ?, instructor_id = ?,
                    capacity_type = ?, capacity_total = ?, male_capacity = ?,
                    female_capacity = ?, status = ?, sort_order = ?,
                    updated_at = CURRENT_TIMESTAMP
                WHERE id = ?
                """,
                (
                    course_id,
                    section_label,
                    instructor_id,
                    capacity_type,
                    capacity_total,
                    male_capacity,
                    female_capacity,
                    status,
                    sort_order,
                    offering_id,
                ),
            )
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError(
                "offering_section_exists", "같은 과목에 같은 분반이 이미 있습니다."
            ) from error
        if cursor.rowcount == 0:
            return None
        row = conn.execute("SELECT * FROM course_offerings WHERE id = ?", (offering_id,)).fetchone()
    return dict(row)


def delete_offering(offering_id: int) -> bool:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute("DELETE FROM course_offerings WHERE id = ?", (offering_id,))
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError(
                "offering_in_use", "신청이나 추첨 이력이 있어 삭제할 수 없습니다."
            ) from error
    return cursor.rowcount > 0


# --- course_schedules (시간표) ---


def list_schedules(offering_id: int) -> list[dict[str, Any]]:
    with get_db_connection() as conn:
        rows = conn.execute(
            "SELECT * FROM course_schedules WHERE offering_id = ? ORDER BY weekday, time_slot_id",
            (offering_id,),
        ).fetchall()
    return [dict(row) for row in rows]


def get_schedule(schedule_id: int) -> dict[str, Any] | None:
    with get_db_connection() as conn:
        row = conn.execute("SELECT * FROM course_schedules WHERE id = ?", (schedule_id,)).fetchone()
    return dict(row) if row is not None else None


def create_schedule(
    *, offering_id: int, weekday: int, time_slot_id: int, space_id: int
) -> dict[str, Any]:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute(
                """
                INSERT INTO course_schedules (offering_id, weekday, time_slot_id, space_id)
                VALUES (?, ?, ?, ?)
                """,
                (offering_id, weekday, time_slot_id, space_id),
            )
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError(
                "schedule_exists", "이 개설 강좌에 같은 요일·교시가 이미 있습니다."
            ) from error
        row = conn.execute(
            "SELECT * FROM course_schedules WHERE id = ?", (cursor.lastrowid,)
        ).fetchone()
    return dict(row)


def update_schedule(
    schedule_id: int, *, weekday: int, time_slot_id: int, space_id: int
) -> dict[str, Any] | None:
    with get_db_connection() as conn:
        try:
            cursor = conn.execute(
                """
                UPDATE course_schedules
                SET weekday = ?, time_slot_id = ?, space_id = ?
                WHERE id = ?
                """,
                (weekday, time_slot_id, space_id, schedule_id),
            )
            conn.commit()
        except sqlite3.IntegrityError as error:
            raise ConflictError(
                "schedule_exists", "이 개설 강좌에 같은 요일·교시가 이미 있습니다."
            ) from error
        if cursor.rowcount == 0:
            return None
        row = conn.execute("SELECT * FROM course_schedules WHERE id = ?", (schedule_id,)).fetchone()
    return dict(row)


def delete_schedule(schedule_id: int) -> bool:
    with get_db_connection() as conn:
        cursor = conn.execute("DELETE FROM course_schedules WHERE id = ?", (schedule_id,))
        conn.commit()
    return cursor.rowcount > 0
