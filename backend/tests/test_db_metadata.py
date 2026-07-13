from app.db import models
from app.db.base import Base

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


def test_metadata_registers_schema_baseline_tables() -> None:
    assert len(models.MODEL_MODULES) == 9
    assert set(Base.metadata.tables) == EXPECTED_TABLES


def test_all_version_columns_enable_sqlalchemy_optimistic_concurrency() -> None:
    versioned_mappers = [
        mapper for mapper in Base.registry.mappers if "version" in mapper.local_table.c
    ]

    assert versioned_mappers
    assert all(
        mapper.version_id_col is mapper.local_table.c.version for mapper in versioned_mappers
    )
