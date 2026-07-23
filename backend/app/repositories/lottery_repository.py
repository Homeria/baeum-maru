"""추첨 실행 snapshot과 결과 데이터를 다루는 repository."""

from typing import Any

from app.db.database import get_db_connection

# --- Read ---


def get_draw_candidates() -> list[dict[str, Any]]:
    """전체 'applied' 신청자를 개설강좌·성별과 함께 조회한다(알고리즘 입력)."""
    with get_db_connection() as conn:
        rows = conn.execute(
            """
            SELECT o.id AS offering_id, o.capacity_type, o.capacity_total,
                   o.male_capacity, o.female_capacity,
                   r.id AS registration_id, m.gender AS gender
            FROM course_offerings o
            JOIN registrations r ON r.offering_id = o.id AND r.status = 'applied'
            JOIN members m ON m.id = r.member_id
            ORDER BY o.id, r.id
            """
        ).fetchall()
    return [dict(row) for row in rows]


def list_runs() -> list[dict[str, Any]]:
    with get_db_connection() as conn:
        rows = conn.execute("SELECT * FROM lottery_runs ORDER BY id DESC").fetchall()
    return [dict(row) for row in rows]


def get_run(run_id: int) -> dict[str, Any] | None:
    with get_db_connection() as conn:
        row = conn.execute("SELECT * FROM lottery_runs WHERE id = ?", (run_id,)).fetchone()
    return dict(row) if row is not None else None


def get_run_targets(run_id: int) -> list[dict[str, Any]]:
    with get_db_connection() as conn:
        rows = conn.execute(
            "SELECT * FROM lottery_run_targets WHERE lottery_run_id = ? ORDER BY id",
            (run_id,),
        ).fetchall()
    return [dict(row) for row in rows]


def get_run_results(run_id: int) -> list[dict[str, Any]]:
    with get_db_connection() as conn:
        rows = conn.execute(
            """
            SELECT lr.* FROM lottery_results lr
            JOIN lottery_run_targets t ON t.id = lr.lottery_run_target_id
            WHERE t.lottery_run_id = ?
            ORDER BY lr.lottery_run_target_id, lr.result, lr.result_order
            """,
            (run_id,),
        ).fetchall()
    return [dict(row) for row in rows]


# --- Write (원자적 commit) ---


def commit_lottery(
    *,
    seed: int,
    executed_by_operator_id: int | None,
    actor_display_name: str | None,
    targets: list[dict[str, Any]],
) -> int:
    """추첨 결과를 한 transaction으로 저장하고 registrations 상태를 반영한다."""
    actor_kind = "operator" if executed_by_operator_id is not None else "system"
    with get_db_connection() as conn:
        try:
            run_cursor = conn.execute(
                "INSERT INTO lottery_runs (seed, executed_by_operator_id) VALUES (?, ?)",
                (seed, executed_by_operator_id),
            )
            run_id = int(run_cursor.lastrowid)  # type: ignore[arg-type]
            for target in targets:
                target_cursor = conn.execute(
                    """
                    INSERT INTO lottery_run_targets (
                        lottery_run_id, offering_id, capacity_type, capacity_total,
                        male_capacity, female_capacity, eligible_count,
                        eligible_male, eligible_female
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                    """,
                    (
                        run_id,
                        target["offering_id"],
                        target["capacity_type"],
                        target["capacity_total"],
                        target["male_capacity"],
                        target["female_capacity"],
                        target["eligible_count"],
                        target["eligible_male"],
                        target["eligible_female"],
                    ),
                )
                target_id = int(target_cursor.lastrowid)  # type: ignore[arg-type]
                for res in target["results"]:
                    conn.execute(
                        "INSERT INTO lottery_results "
                        "(lottery_run_target_id, registration_id, result, result_order) "
                        "VALUES (?, ?, ?, ?)",
                        (target_id, res["registration_id"], res["result"], res["result_order"]),
                    )
                    waitlist_order = res["result_order"] if res["result"] == "waitlisted" else None
                    conn.execute(
                        "UPDATE registrations SET status = ?, waitlist_order = ?, "
                        "updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                        (res["result"], waitlist_order, res["registration_id"]),
                    )
                    conn.execute(
                        """
                        INSERT INTO registration_status_history
                            (registration_id, from_status, to_status, reason,
                             actor_kind, actor_operator_id, actor_display_name)
                        VALUES (?, 'applied', ?, '추첨', ?, ?, ?)
                        """,
                        (
                            res["registration_id"],
                            res["result"],
                            actor_kind,
                            executed_by_operator_id,
                            actor_display_name,
                        ),
                    )
            conn.commit()
        except BaseException:
            conn.rollback()
            raise
        return run_id
