"""강좌 신청과 상태 이력 데이터를 다루는 repository."""

import sqlite3
from typing import Any

from app.core.exceptions import ConflictError
from app.db.database import get_db_connection

# 취소·탈락이 아닌 유효한 신청 상태
_ACTIVE = ("applied", "selected", "waitlisted", "confirmed")
_ACTIVE_SQL = "('applied', 'selected', 'waitlisted', 'confirmed')"


def _history(
    conn: sqlite3.Connection,
    registration_id: int,
    from_status: str | None,
    to_status: str,
    reason: str | None = None,
    *,
    actor_operator_id: int | None = None,
    actor_display_name: str | None = None,
) -> None:
    actor_kind = "operator" if actor_operator_id is not None else "system"
    conn.execute(
        """
        INSERT INTO registration_status_history
            (registration_id, from_status, to_status, reason,
             actor_kind, actor_operator_id, actor_display_name)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        """,
        (
            registration_id,
            from_status,
            to_status,
            reason,
            actor_kind,
            actor_operator_id,
            actor_display_name,
        ),
    )


# --- Read ---


def get_registration(registration_id: int) -> dict[str, Any] | None:
    with get_db_connection() as conn:
        row = conn.execute(
            "SELECT * FROM registrations WHERE id = ?", (registration_id,)
        ).fetchone()
    return dict(row) if row is not None else None


def list_registrations(
    *,
    member_id: int | None = None,
    offering_id: int | None = None,
    term_id: int | None = None,
    status: str | None = None,
) -> list[dict[str, Any]]:
    query = "SELECT * FROM registrations"
    conditions: list[str] = []
    params: list[Any] = []
    if member_id is not None:
        conditions.append("member_id = ?")
        params.append(member_id)
    if offering_id is not None:
        conditions.append("offering_id = ?")
        params.append(offering_id)
    if status is not None:
        conditions.append("status = ?")
        params.append(status)
    if term_id is not None:
        conditions.append("offering_id IN (SELECT id FROM course_offerings WHERE term_id = ?)")
        params.append(term_id)
    if conditions:
        query += " WHERE " + " AND ".join(conditions)
    query += " ORDER BY id"
    with get_db_connection() as conn:
        rows = conn.execute(query, params).fetchall()
    return [dict(row) for row in rows]


def list_history(registration_id: int) -> list[dict[str, Any]]:
    with get_db_connection() as conn:
        rows = conn.execute(
            "SELECT * FROM registration_status_history WHERE registration_id = ? "
            "ORDER BY changed_at, id",
            (registration_id,),
        ).fetchall()
    return [dict(row) for row in rows]


def count_active_in_term(member_id: int, term_id: int) -> int:
    with get_db_connection() as conn:
        row = conn.execute(
            f"""
            SELECT COUNT(*) FROM registrations r
            JOIN course_offerings o ON o.id = r.offering_id
            WHERE r.member_id = ? AND o.term_id = ? AND r.status IN {_ACTIVE_SQL}
            """,
            (member_id, term_id),
        ).fetchone()
    return int(row[0])


def get_member_active_slots(member_id: int) -> list[tuple[int, int]]:
    with get_db_connection() as conn:
        rows = conn.execute(
            f"""
            SELECT cs.weekday, cs.time_slot_id FROM course_schedules cs
            JOIN registrations r ON r.offering_id = cs.offering_id
            WHERE r.member_id = ? AND r.status IN {_ACTIVE_SQL}
            """,
            (member_id,),
        ).fetchall()
    return [(int(r[0]), int(r[1])) for r in rows]


def get_offering_slots(offering_id: int) -> list[tuple[int, int]]:
    with get_db_connection() as conn:
        rows = conn.execute(
            "SELECT weekday, time_slot_id FROM course_schedules WHERE offering_id = ?",
            (offering_id,),
        ).fetchall()
    return [(int(r[0]), int(r[1])) for r in rows]


# --- Write (원자적) ---


def apply_registrations(
    member_id: int,
    offering_ids: list[int],
    *,
    actor_operator_id: int | None = None,
    actor_display_name: str | None = None,
) -> list[dict[str, Any]]:
    """여러 강좌 신청을 한 transaction으로 처리한다(전부 아니면 전무).

    이미 취소/탈락한 신청은 다시 applied로 되살리고, 활성 신청이면 전체를 취소한다.
    """
    with get_db_connection() as conn:
        try:
            ids: list[int] = []
            for offering_id in offering_ids:
                existing = conn.execute(
                    "SELECT id, status FROM registrations WHERE member_id = ? AND offering_id = ?",
                    (member_id, offering_id),
                ).fetchone()
                if existing is None:
                    cursor = conn.execute(
                        "INSERT INTO registrations (member_id, offering_id, status) "
                        "VALUES (?, ?, 'applied')",
                        (member_id, offering_id),
                    )
                    rid = int(cursor.lastrowid)  # type: ignore[arg-type]
                    _history(
                        conn,
                        rid,
                        None,
                        "applied",
                        actor_operator_id=actor_operator_id,
                        actor_display_name=actor_display_name,
                    )
                elif existing["status"] in ("cancelled", "rejected"):
                    rid = int(existing["id"])
                    conn.execute(
                        "UPDATE registrations SET status = 'applied', waitlist_order = NULL, "
                        "updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                        (rid,),
                    )
                    _history(
                        conn,
                        rid,
                        str(existing["status"]),
                        "applied",
                        actor_operator_id=actor_operator_id,
                        actor_display_name=actor_display_name,
                    )
                else:
                    raise ConflictError("already_applied", "이미 신청한 강좌가 있습니다.")
                ids.append(rid)
            conn.commit()
        except sqlite3.IntegrityError as error:
            conn.rollback()
            raise ConflictError(
                "registration_conflict", "신청 처리 중 충돌이 발생했습니다."
            ) from error
        except BaseException:
            conn.rollback()
            raise
        return [
            dict(conn.execute("SELECT * FROM registrations WHERE id = ?", (i,)).fetchone())
            for i in ids
        ]


def cancel_registration(
    registration_id: int,
    reason: str | None,
    *,
    actor_operator_id: int | None = None,
    actor_display_name: str | None = None,
) -> dict[str, Any] | None:
    """신청을 취소하고, 당첨자였다면 대기 순번대로 승계한다(한 transaction)."""
    with get_db_connection() as conn:
        reg = conn.execute(
            "SELECT * FROM registrations WHERE id = ?", (registration_id,)
        ).fetchone()
        if reg is None:
            return None
        if reg["status"] == "cancelled":
            return dict(reg)
        try:
            conn.execute(
                "UPDATE registrations SET status = 'cancelled', updated_at = CURRENT_TIMESTAMP "
                "WHERE id = ?",
                (registration_id,),
            )
            _history(
                conn,
                registration_id,
                str(reg["status"]),
                "cancelled",
                reason,
                actor_operator_id=actor_operator_id,
                actor_display_name=actor_display_name,
            )
            if reg["status"] == "selected":
                nxt = conn.execute(
                    "SELECT id FROM registrations WHERE offering_id = ? AND status = 'waitlisted' "
                    "ORDER BY waitlist_order LIMIT 1",
                    (reg["offering_id"],),
                ).fetchone()
                if nxt is not None:
                    conn.execute(
                        "UPDATE registrations SET status = 'selected', "
                        "updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                        (int(nxt["id"]),),
                    )
                    _history(
                        conn,
                        int(nxt["id"]),
                        "waitlisted",
                        "selected",
                        "대기 승계",
                        actor_operator_id=actor_operator_id,
                        actor_display_name=actor_display_name,
                    )
            conn.commit()
        except BaseException:
            conn.rollback()
            raise
        return dict(
            conn.execute("SELECT * FROM registrations WHERE id = ?", (registration_id,)).fetchone()
        )
