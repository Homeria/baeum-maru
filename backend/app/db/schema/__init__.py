"""도메인별 sqlite3 DDL을 생성 의존 순서로 모은다."""

from app.db.schema import (
    buildings,
    courses,
    identity,
    lottery,
    members,
    operations,
    registrations,
    spaces,
)

SCHEMA_MODULES = (
    identity,
    buildings,
    spaces,
    courses,
    members,
    registrations,
    lottery,
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
        "audit_logs",
        "building_floors",
        "buildings",
        "course_categories",
        "course_levels",
        "course_offerings",
        "course_schedules",
        "courses",
        "instructors",
        "lottery_results",
        "lottery_run_targets",
        "lottery_runs",
        "members",
        "operator_sessions",
        "operators",
        "registration_status_history",
        "registrations",
        "space_types",
        "spaces",
        "terms",
        "time_slots",
    }
)

__all__ = ["SCHEMA_MODULES", "SCHEMA_STATEMENTS", "SEED_STATEMENTS", "TABLE_NAMES"]
