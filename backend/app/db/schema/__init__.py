"""도메인별 sqlite3 DDL을 생성 의존 순서로 모은다."""

from app.db.schema import (
    attendance,
    courses,
    identity,
    locations,
    lottery,
    members,
    operations,
    organization,
    registrations,
)

SCHEMA_MODULES = (
    organization,
    identity,
    locations,
    courses,
    members,
    registrations,
    lottery,
    attendance,
    operations,
)

SCHEMA_STATEMENTS = tuple(
    statement.strip() for module in SCHEMA_MODULES for statement in module.STATEMENTS
)

SEED_STATEMENTS = tuple(
    statement.strip()
    for module in SCHEMA_MODULES
    for statement in getattr(module, "SEED_STATEMENTS", ())
)

TABLE_NAMES = frozenset(
    {
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
)

__all__ = ["SCHEMA_MODULES", "SCHEMA_STATEMENTS", "SEED_STATEMENTS", "TABLE_NAMES"]
