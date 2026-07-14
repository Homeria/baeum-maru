"""실제 SQLite에서 aggregate CASCADE와 업무 이력 RESTRICT 정책을 검증한다."""

import sqlite3

import pytest

from app.db.database import Database
from tests.db.schema_seed import seed_operational_graph


def row_count(database: Database, table_name: str) -> int:
    with database.connection() as connection:
        value = connection.execute(f'SELECT COUNT(*) FROM "{table_name}"').fetchone()[0]
    assert isinstance(value, int)
    return value


def test_referenced_business_rows_are_restricted(initialized_database: Database) -> None:
    seed_operational_graph(initialized_database)
    restricted_deletes = (
        "DELETE FROM users WHERE id = 1",
        "DELETE FROM building_floors WHERE id = 1",
        "DELETE FROM members WHERE id = 1",
        "DELETE FROM course_offerings WHERE id = 1",
    )
    for statement in restricted_deletes:
        with (
            pytest.raises(sqlite3.IntegrityError),
            initialized_database.transaction() as connection,
        ):
            connection.execute(statement)


def test_owned_rows_are_removed_by_cascade(initialized_database: Database) -> None:
    seed_operational_graph(initialized_database)
    with initialized_database.transaction() as connection:
        connection.execute("DELETE FROM operation_jobs WHERE id = 1")
        connection.execute("DELETE FROM attendance_sessions WHERE id = 1")
        connection.execute("DELETE FROM lottery_runs WHERE id = 1")
        connection.execute("DELETE FROM registrations WHERE id = 1")
        connection.execute("DELETE FROM course_offerings WHERE id = 1")
        connection.execute("DELETE FROM locations WHERE id = 1")
        connection.execute("DELETE FROM buildings WHERE id = 1")

    assert row_count(initialized_database, "operation_job_errors") == 0
    assert row_count(initialized_database, "attendance_records") == 0
    assert row_count(initialized_database, "lottery_run_targets") == 0
    assert row_count(initialized_database, "lottery_results") == 0
    assert row_count(initialized_database, "registration_status_history") == 0
    assert row_count(initialized_database, "course_schedules") == 0
    assert row_count(initialized_database, "location_role_assignments") == 0
    assert row_count(initialized_database, "building_floors") == 0


def test_deleting_location_role_only_removes_assignments(
    initialized_database: Database,
) -> None:
    seed_operational_graph(initialized_database)
    with initialized_database.transaction() as connection:
        connection.execute("DELETE FROM location_roles WHERE id = 1")

    assert row_count(initialized_database, "location_role_assignments") == 0
    assert row_count(initialized_database, "locations") == 1
