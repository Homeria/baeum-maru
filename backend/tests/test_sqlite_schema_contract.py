from collections.abc import Iterator
from pathlib import Path

import pytest
from alembic.config import Config
from sqlalchemy import Connection, inspect, text
from sqlalchemy.exc import IntegrityError

from alembic import command
from app.db.session import Database, create_database, database_url
from tests.schema_contract import build_sqlite_schema_contract, load_sqlite_schema_contract

BACKEND_ROOT = Path(__file__).resolve().parents[1]


@pytest.fixture
def migrated_database(tmp_path: Path) -> Iterator[Database]:
    database_file = tmp_path / "한글 경로" / "schema contract.db"
    database_file.parent.mkdir(parents=True)
    configuration = Config(str(BACKEND_ROOT / "alembic.ini"))
    configuration.set_main_option(
        "sqlalchemy.url",
        database_url(database_file).render_as_string(hide_password=False),
    )
    command.upgrade(configuration, "head")

    database = create_database(database_file)
    try:
        yield database
    finally:
        database.dispose()


def test_migrated_sqlite_schema_matches_approved_contract(
    migrated_database: Database,
) -> None:
    actual_contract = build_sqlite_schema_contract(inspect(migrated_database.engine))

    assert actual_contract == load_sqlite_schema_contract()


def test_sqlite_rejects_representative_check_and_foreign_key_violations(
    migrated_database: Database,
) -> None:
    with migrated_database.engine.begin() as connection:
        connection.execute(
            text("INSERT INTO users (id, display_name, role) VALUES (1, '담당자', 'staff')")
        )

    invalid_statements = [
        "INSERT INTO organization_settings (id, organization_name) VALUES (2, '기관')",
        "INSERT INTO buildings (id, name, is_active) VALUES (1, '본관', 2)",
        "INSERT INTO access_codes "
        "(id, user_id, code_hash, display_code, issued_at, expires_at) "
        "VALUES (1, 1, 'hash', 'CODE', '2026-01-01 10:00:00', '2026-01-01 10:00:00')",
        "INSERT INTO terms (id, name, starts_on, ends_on) "
        "VALUES (1, '잘못된 학기', '2026-12-31', '2026-01-01')",
        "INSERT INTO time_slots (id, name, start_time, end_time) "
        "VALUES (1, '역전 교시', '11:00:00', '10:00:00')",
        "INSERT INTO operation_jobs (id, job_type, status, total_count, success_count, "
        "failure_count) VALUES (1, 'export', 'queued', 1, 1, 1)",
        "INSERT INTO idempotency_records "
        "(id, namespace, key_hash, request_hash, status, created_at, expires_at) "
        "VALUES (1, 'registration', 'key', 'request', 'processing', "
        "'2026-01-02 00:00:00', '2026-01-01 00:00:00')",
        "INSERT INTO operation_locks "
        "(resource_type, resource_id, operation, owner_token, acquired_at, expires_at) "
        "VALUES ('term', '1', 'lottery', 'owner', "
        "'2026-01-02 00:00:00', '2026-01-01 00:00:00')",
        "INSERT INTO members (id, member_no, name, gender_code, phone) "
        "VALUES (1, '10-00001', '회원', 'not-a-gender', '01000000000')",
    ]

    for statement in invalid_statements:
        with pytest.raises(IntegrityError), migrated_database.engine.begin() as connection:
            connection.execute(text(statement))


def test_unique_and_partial_unique_constraints_are_enforced(
    migrated_database: Database,
) -> None:
    with migrated_database.engine.begin() as connection:
        connection.execute(text("INSERT INTO buildings (id, name) VALUES (1, '본관')"))

    with pytest.raises(IntegrityError), migrated_database.engine.begin() as connection:
        connection.execute(text("INSERT INTO buildings (id, name) VALUES (2, '본관')"))

    with migrated_database.engine.begin() as connection:
        _insert_course_offering_references(connection)
        connection.execute(
            text(
                "INSERT INTO course_offerings "
                "(id, term_id, course_id, capacity_type, capacity_total) "
                "VALUES (1, 1, 1, 'fixed', 20)"
            )
        )

    with pytest.raises(IntegrityError), migrated_database.engine.begin() as connection:
        connection.execute(
            text(
                "INSERT INTO course_offerings "
                "(id, term_id, course_id, capacity_type, capacity_total) "
                "VALUES (2, 1, 1, 'fixed', 20)"
            )
        )

    with migrated_database.engine.begin() as connection:
        connection.execute(
            text(
                "INSERT INTO course_offerings "
                "(id, term_id, course_id, section_label, capacity_type, capacity_total) "
                "VALUES (3, 1, 1, '1반', 'fixed', 20), "
                "(4, 1, 1, '2반', 'fixed', 20)"
            )
        )

    with pytest.raises(IntegrityError), migrated_database.engine.begin() as connection:
        connection.execute(
            text(
                "INSERT INTO course_offerings "
                "(id, term_id, course_id, section_label, capacity_type, capacity_total) "
                "VALUES (5, 1, 1, '1반', 'fixed', 20)"
            )
        )


def test_cascade_and_restrict_delete_policies_are_enforced(
    migrated_database: Database,
) -> None:
    with migrated_database.engine.begin() as connection:
        connection.execute(
            text("INSERT INTO buildings (id, name) VALUES (1, '철거 가능 건물'), (2, '본관')")
        )
        connection.execute(
            text(
                "INSERT INTO building_floors (id, building_id, label) "
                "VALUES (1, 1, '1층'), (2, 2, '2층')"
            )
        )
        connection.execute(
            text("INSERT INTO locations (id, building_floor_id, name) VALUES (1, 2, '문화실')")
        )
        connection.execute(text("INSERT INTO location_roles (id, name) VALUES (1, '강의')"))
        connection.execute(
            text("INSERT INTO location_role_assignments (location_id, role_id) VALUES (1, 1)")
        )

        connection.execute(text("DELETE FROM buildings WHERE id = 1"))
        assert (
            connection.scalar(text("SELECT count(*) FROM building_floors WHERE building_id = 1"))
            == 0
        )

        connection.execute(text("DELETE FROM location_roles WHERE id = 1"))
        assert (
            connection.scalar(
                text("SELECT count(*) FROM location_role_assignments WHERE role_id = 1")
            )
            == 0
        )

    with pytest.raises(IntegrityError), migrated_database.engine.begin() as connection:
        connection.execute(text("DELETE FROM buildings WHERE id = 2"))

    with migrated_database.engine.connect() as connection:
        assert connection.scalar(text("SELECT count(*) FROM buildings WHERE id = 2")) == 1
        assert connection.scalar(text("PRAGMA foreign_key_check")) is None


def _insert_course_offering_references(connection: Connection) -> None:
    connection.execute(text("INSERT INTO course_categories (id, name) VALUES (1, '평생교육')"))
    connection.execute(
        text("INSERT INTO courses (id, category_id, name) VALUES (1, 1, '한글교실 초급')")
    )
    connection.execute(text("INSERT INTO terms (id, name) VALUES (1, '2026년 2학기')"))
