"""실제 SQLite의 FK 정책과 필수 query index 구조를 검증한다."""

from app.db.database import Database

EXPECTED_INDEXES = {
    "ix_access_codes_lifecycle",
    "ix_access_codes_user_id",
    "ix_attendance_records_registration_id",
    "ix_attendance_sessions_offering_date",
    "ix_audit_logs_actor_user_id",
    "ix_audit_logs_created_at",
    "ix_audit_logs_resource",
    "ix_building_floors_building_id",
    "ix_course_offerings_course_id",
    "ix_course_offerings_instructor_id",
    "ix_course_offerings_term_status",
    "ix_course_schedules_location_id",
    "ix_course_schedules_offering_id",
    "ix_course_schedules_weekday_slot",
    "ix_courses_category_active",
    "ix_idempotency_records_expires_at",
    "ix_location_role_assignments_role_id",
    "ix_locations_floor_active",
    "ix_lottery_results_registration_id",
    "ix_lottery_results_target_id",
    "ix_lottery_run_targets_offering_id",
    "ix_lottery_run_targets_run_id",
    "ix_lottery_runs_term_status",
    "ix_members_is_active",
    "ix_members_name",
    "ix_members_phone",
    "ix_operation_job_errors_job_id",
    "ix_operation_jobs_type_status",
    "ix_operation_locks_expires_at",
    "ix_registrations_member_id",
    "ix_registrations_offering_status",
    "ix_registration_status_history_registration_changed",
    "ix_user_sessions_access_code_id",
    "ix_user_sessions_lifecycle",
    "ix_user_sessions_user_id",
    "ix_users_role_is_active",
    "uq_course_offerings_term_course_no_section",
    "uq_course_offerings_term_course_section",
}

EXPECTED_CASCADE_FOREIGN_KEYS = {
    ("attendance_records", "attendance_session_id"),
    ("building_floors", "building_id"),
    ("course_schedules", "offering_id"),
    ("location_role_assignments", "location_id"),
    ("location_role_assignments", "role_id"),
    ("lottery_results", "lottery_run_target_id"),
    ("lottery_run_targets", "lottery_run_id"),
    ("operation_job_errors", "job_id"),
    ("registration_status_history", "registration_id"),
}


def test_required_query_indexes_exist_without_duplicate_names(
    initialized_database: Database,
) -> None:
    with initialized_database.connection() as connection:
        table_names = [
            str(row[0])
            for row in connection.execute(
                "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'"
            )
        ]
        index_names = [
            str(row[1])
            for table_name in table_names
            for row in connection.execute(f'PRAGMA index_list("{table_name}")')
            if not str(row[1]).startswith("sqlite_autoindex_")
        ]

    assert set(index_names) == EXPECTED_INDEXES
    assert len(index_names) == len(set(index_names))


def test_every_foreign_key_has_documented_delete_policy(
    initialized_database: Database,
) -> None:
    with initialized_database.connection() as connection:
        table_names = [
            str(row[0])
            for row in connection.execute(
                "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'"
            )
        ]
        actual_policies = {
            (table_name, str(row[3])): str(row[6]).upper()
            for table_name in table_names
            for row in connection.execute(f'PRAGMA foreign_key_list("{table_name}")')
        }

    assert len(actual_policies) == 35
    assert {
        key for key, policy in actual_policies.items() if policy == "CASCADE"
    } == EXPECTED_CASCADE_FOREIGN_KEYS
    assert all(
        policy == "CASCADE" if key in EXPECTED_CASCADE_FOREIGN_KEYS else policy == "RESTRICT"
        for key, policy in actual_policies.items()
    )
