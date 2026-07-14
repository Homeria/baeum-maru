"""sqlite3 schema catalog가 문서의 초기 테이블 전체를 등록하는지 검증한다."""

from app.db.schema import SCHEMA_MODULES, TABLE_NAMES

EXPECTED_TABLES = {
    "access_codes",
    "attendance_records",
    "attendance_sessions",
    "audit_logs",
    "building_floors",
    "buildings",
    "course_categories",
    "course_offerings",
    "course_schedules",
    "courses",
    "gender_codes",
    "idempotency_records",
    "instructors",
    "location_role_assignments",
    "location_roles",
    "locations",
    "lottery_results",
    "lottery_run_targets",
    "lottery_runs",
    "members",
    "operation_job_errors",
    "operation_jobs",
    "operation_locks",
    "organization_settings",
    "registration_status_history",
    "registrations",
    "terms",
    "time_slots",
    "user_sessions",
    "users",
}


def test_schema_catalog_registers_all_baseline_tables() -> None:
    assert len(SCHEMA_MODULES) == 9
    assert TABLE_NAMES == EXPECTED_TABLES


def test_schema_uses_plain_sql_statements() -> None:
    statements = [statement for module in SCHEMA_MODULES for statement in module.STATEMENTS]
    assert all(isinstance(statement, str) and statement.strip() for statement in statements)
