"""SQLite schema 계약 테스트가 공유하는 최소 운영 데이터."""

from sqlalchemy import Engine, text


def seed_operational_graph(engine: Engine) -> None:
    with engine.begin() as connection:
        statements = (
            "INSERT INTO users (id, display_name, role) VALUES (1, '담당자', 'staff')",
            "INSERT INTO access_codes "
            "(id, user_id, code_hash, display_code, issued_at, expires_at) "
            "VALUES (1, 1, 'hash-1', 'CODE1', '2026-01-01 09:00:00', '2026-01-01 18:00:00')",
            "INSERT INTO buildings (id, name) VALUES (1, '본관')",
            "INSERT INTO building_floors (id, building_id, label) VALUES (1, 1, '3층')",
            "INSERT INTO location_roles (id, name, is_course_eligible) VALUES (1, '강의', 1)",
            "INSERT INTO locations (id, building_floor_id, name) VALUES (1, 1, '문화교육실')",
            "INSERT INTO location_role_assignments (location_id, role_id) VALUES (1, 1)",
            "INSERT INTO course_categories (id, name) VALUES (1, '평생교육')",
            "INSERT INTO courses (id, category_id, name) VALUES (1, 1, '한글교실 초급')",
            "INSERT INTO instructors (id, name) VALUES (1, '강사')",
            "INSERT INTO terms (id, name) VALUES (1, '2026년 2학기')",
            "INSERT INTO time_slots (id, name, start_time, end_time) "
            "VALUES (1, '오후 1교시', '14:00:00', '14:50:00')",
            "INSERT INTO course_offerings "
            "(id, term_id, course_id, instructor_id, capacity_type, capacity_total) "
            "VALUES (1, 1, 1, 1, 'fixed', 20)",
            "INSERT INTO course_schedules "
            "(id, offering_id, weekday, time_slot_id, location_id) VALUES (1, 1, 1, 1, 1)",
            "INSERT INTO members (id, member_no, name, gender_code, phone) "
            "VALUES (1, '10-00001', '회원', 'unknown', '01012345678')",
            "INSERT INTO registrations (id, member_id, offering_id) VALUES (1, 1, 1)",
            "INSERT INTO registration_status_history "
            "(id, registration_id, to_status, actor_kind) VALUES (1, 1, 'applied', 'system')",
            "INSERT INTO lottery_runs (id, term_id, seed, executed_by_user_id) "
            "VALUES (1, 1, 12345, 1)",
            "INSERT INTO lottery_run_targets "
            "(id, lottery_run_id, offering_id, capacity_type, capacity_total, eligible_count) "
            "VALUES (1, 1, 1, 'fixed', 20, 1)",
            "INSERT INTO lottery_results "
            "(id, lottery_run_target_id, registration_id, result, result_order) "
            "VALUES (1, 1, 1, 'selected', 1)",
            "INSERT INTO attendance_sessions (id, offering_id, session_date) "
            "VALUES (1, 1, '2026-08-03')",
            "INSERT INTO attendance_records "
            "(id, attendance_session_id, registration_id, status) "
            "VALUES (1, 1, 1, 'present')",
            "INSERT INTO operation_jobs (id, job_type) VALUES (1, 'export')",
            "INSERT INTO operation_job_errors (id, job_id, message) VALUES (1, 1, '예시 오류')",
        )
        for statement in statements:
            connection.execute(text(statement))
