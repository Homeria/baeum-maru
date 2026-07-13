"""실제 SQLite에서 aggregate CASCADE와 업무 이력 RESTRICT 정책을 검증한다."""

import pytest
from sqlalchemy import Engine, text
from sqlalchemy.exc import IntegrityError

from tests.db.schema_seed import seed_operational_graph


def row_count(engine: Engine, table_name: str) -> int:
    with engine.connect() as connection:
        value = connection.scalar(text(f"SELECT COUNT(*) FROM {table_name}"))
    assert isinstance(value, int)
    return value


def test_referenced_business_rows_are_restricted(migrated_engine: Engine) -> None:
    seed_operational_graph(migrated_engine)

    restricted_deletes = (
        "DELETE FROM users WHERE id = 1",
        "DELETE FROM building_floors WHERE id = 1",
        "DELETE FROM members WHERE id = 1",
        "DELETE FROM course_offerings WHERE id = 1",
    )
    for statement in restricted_deletes:
        with pytest.raises(IntegrityError), migrated_engine.begin() as connection:
            connection.execute(text(statement))


def test_owned_rows_are_removed_by_cascade(migrated_engine: Engine) -> None:
    seed_operational_graph(migrated_engine)

    with migrated_engine.begin() as connection:
        connection.execute(text("DELETE FROM operation_jobs WHERE id = 1"))
        connection.execute(text("DELETE FROM attendance_sessions WHERE id = 1"))
        connection.execute(text("DELETE FROM lottery_runs WHERE id = 1"))
        connection.execute(text("DELETE FROM registrations WHERE id = 1"))
        connection.execute(text("DELETE FROM course_offerings WHERE id = 1"))
        connection.execute(text("DELETE FROM locations WHERE id = 1"))
        connection.execute(text("DELETE FROM buildings WHERE id = 1"))

    assert row_count(migrated_engine, "operation_job_errors") == 0
    assert row_count(migrated_engine, "attendance_records") == 0
    assert row_count(migrated_engine, "lottery_run_targets") == 0
    assert row_count(migrated_engine, "lottery_results") == 0
    assert row_count(migrated_engine, "registration_status_history") == 0
    assert row_count(migrated_engine, "course_schedules") == 0
    assert row_count(migrated_engine, "location_role_assignments") == 0
    assert row_count(migrated_engine, "building_floors") == 0


def test_deleting_location_role_only_removes_assignments(migrated_engine: Engine) -> None:
    seed_operational_graph(migrated_engine)

    with migrated_engine.begin() as connection:
        connection.execute(text("DELETE FROM location_roles WHERE id = 1"))

    assert row_count(migrated_engine, "location_role_assignments") == 0
    assert row_count(migrated_engine, "locations") == 1
