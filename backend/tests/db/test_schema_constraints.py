"""대표 입력으로 SQLite CHECK, UNIQUE와 partial index 계약을 검증한다."""

import sqlite3
from collections.abc import Mapping
from typing import Any

import pytest

from app.db.database import Database
from tests.db.schema_seed import seed_operational_graph


def assert_integrity_error(
    database: Database,
    statement: str,
    parameters: Mapping[str, Any] | None = None,
) -> None:
    with pytest.raises(sqlite3.IntegrityError), database.transaction() as connection:
        connection.execute(statement, parameters or {})


def test_identity_and_reference_checks_reject_invalid_values(
    initialized_database: Database,
) -> None:
    seed_operational_graph(initialized_database)
    invalid_statements = (
        "INSERT INTO organization_settings (id, organization_name) VALUES (2, '기관')",
        "INSERT INTO users (id, display_name, role) VALUES (2, '사용자', 'administrator')",
        "INSERT INTO buildings (id, name, is_active) VALUES (2, '별관', 2)",
        "INSERT INTO access_codes "
        "(id, user_id, code_hash, display_code, issued_at, expires_at) "
        "VALUES (2, 1, 'hash-2', 'CODE2', '2026-01-02 18:00:00', '2026-01-02 09:00:00')",
        "INSERT INTO building_floors (id, building_id, label) VALUES (2, 1, '3층')",
        "INSERT INTO time_slots (id, name, start_time, end_time) "
        "VALUES (2, '잘못된 교시', '15:00:00', '14:00:00')",
    )
    for statement in invalid_statements:
        assert_integrity_error(initialized_database, statement)


def test_course_capacity_schedule_and_section_constraints(
    initialized_database: Database,
) -> None:
    seed_operational_graph(initialized_database)
    invalid_capacity_statements = (
        "INSERT INTO course_offerings "
        "(id, term_id, course_id, section_label, capacity_type) "
        "VALUES (2, 1, 1, '고정정원 누락', 'fixed')",
        "INSERT INTO course_offerings "
        "(id, term_id, course_id, section_label, capacity_type, capacity_total) "
        "VALUES (2, 1, 1, '무제한 오류', 'open', 10)",
        "INSERT INTO course_offerings "
        "(id, term_id, course_id, section_label, capacity_type, male_capacity, female_capacity) "
        "VALUES (2, 1, 1, '성별정원 오류', 'gender_split', 0, 0)",
    )
    for statement in invalid_capacity_statements:
        assert_integrity_error(initialized_database, statement)

    assert_integrity_error(
        initialized_database,
        "INSERT INTO course_offerings "
        "(id, term_id, course_id, capacity_type, capacity_total) "
        "VALUES (2, 1, 1, 'fixed', 10)",
    )
    assert_integrity_error(
        initialized_database,
        "INSERT INTO course_offerings "
        "(id, term_id, course_id, section_label, capacity_type, capacity_total) "
        "VALUES (2, 1, 1, '   ', 'fixed', 10)",
    )

    with initialized_database.transaction() as connection:
        connection.execute(
            "INSERT INTO course_offerings "
            "(id, term_id, course_id, section_label, capacity_type, capacity_total) "
            "VALUES (2, 1, 1, '1반', 'fixed', 10)"
        )

    assert_integrity_error(
        initialized_database,
        "INSERT INTO course_offerings "
        "(id, term_id, course_id, section_label, capacity_type, capacity_total) "
        "VALUES (3, 1, 1, '1반', 'fixed', 10)",
    )
    assert_integrity_error(
        initialized_database,
        "INSERT INTO course_schedules "
        "(id, offering_id, weekday, time_slot_id, location_id) VALUES (2, 1, 8, 1, 1)",
    )


def test_registration_lottery_and_attendance_constraints(
    initialized_database: Database,
) -> None:
    seed_operational_graph(initialized_database)
    with initialized_database.transaction() as connection:
        connection.execute(
            "INSERT INTO members (id, member_no, name, gender_code, phone) "
            "VALUES (2, '10-00002', '회원2', 'unknown', '01000000002'), "
            "(3, '10-00003', '회원3', 'unknown', '01000000003')"
        )

    invalid_statements = (
        "INSERT INTO registrations (id, member_id, offering_id) VALUES (2, 1, 1)",
        "INSERT INTO registrations (id, member_id, offering_id, status) "
        "VALUES (2, 2, 1, 'cancelled')",
        "INSERT INTO registrations (id, member_id, offering_id, status, cancelled_at) "
        "VALUES (2, 2, 1, 'applied', '2026-01-02 10:00:00')",
        "INSERT INTO registration_status_history "
        "(id, registration_id, to_status, actor_kind) "
        "VALUES (2, 1, 'unknown', 'system')",
    )
    for statement in invalid_statements:
        assert_integrity_error(initialized_database, statement)

    with initialized_database.transaction() as connection:
        connection.execute(
            "INSERT INTO registrations (id, member_id, offering_id) VALUES (2, 2, 1)"
        )

    assert_integrity_error(
        initialized_database,
        "INSERT INTO lottery_results "
        "(id, lottery_run_target_id, registration_id, result, result_order) "
        "VALUES (2, 1, 2, 'selected', 0)",
    )
    assert_integrity_error(
        initialized_database,
        "INSERT INTO attendance_records "
        "(id, attendance_session_id, registration_id, status) "
        "VALUES (2, 1, 2, 'unknown')",
    )


def test_job_idempotency_and_lock_checks_reject_invalid_state(
    initialized_database: Database,
) -> None:
    invalid_statements = (
        "INSERT INTO operation_jobs "
        "(id, job_type, total_count, success_count, failure_count) "
        "VALUES (2, 'import', 1, 1, 1)",
        "INSERT INTO idempotency_records "
        "(id, namespace, key_hash, request_hash, status, created_at, updated_at, expires_at) "
        "VALUES (1, 'registration:create', 'key', 'request', 'processing', "
        "'2026-01-02 10:00:00', '2026-01-02 10:00:00', '2026-01-02 09:00:00')",
        "INSERT INTO operation_locks "
        "(resource_type, resource_id, operation, owner_token, acquired_at, expires_at) "
        "VALUES ('lottery', '1', 'run', 'owner', "
        "'2026-01-02 10:00:00', '2026-01-02 09:00:00')",
    )
    for statement in invalid_statements:
        assert_integrity_error(initialized_database, statement)
