"""추첨 실행, 대상 snapshot과 결과 테이블 DDL."""

from app.db.schema.courses import _CAPACITY_CHECK

STATEMENTS = (
    """
    CREATE TABLE IF NOT EXISTS lottery_runs (
        id INTEGER PRIMARY KEY,
        term_id INTEGER NOT NULL REFERENCES terms(id) ON DELETE RESTRICT,
        seed INTEGER NOT NULL,
        status TEXT NOT NULL DEFAULT 'prepared'
            CHECK (status IN ('prepared', 'running', 'completed', 'failed', 'cancelled')),
        executed_by_user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        started_at TEXT,
        completed_at TEXT,
        note TEXT
    )
    """,
    f"""
    CREATE TABLE IF NOT EXISTS lottery_run_targets (
        id INTEGER PRIMARY KEY,
        lottery_run_id INTEGER NOT NULL REFERENCES lottery_runs(id) ON DELETE CASCADE,
        offering_id INTEGER NOT NULL REFERENCES course_offerings(id) ON DELETE RESTRICT,
        capacity_type TEXT NOT NULL
            CHECK (capacity_type IN ('fixed', 'open', 'gender_split')),
        capacity_total INTEGER,
        male_capacity INTEGER,
        female_capacity INTEGER,
        eligible_count INTEGER NOT NULL CHECK (eligible_count >= 0),
        UNIQUE (lottery_run_id, offering_id),
        CHECK ({_CAPACITY_CHECK})
    )
    """,
    """
    CREATE TABLE IF NOT EXISTS lottery_results (
        id INTEGER PRIMARY KEY,
        lottery_run_target_id INTEGER NOT NULL
            REFERENCES lottery_run_targets(id) ON DELETE CASCADE,
        registration_id INTEGER NOT NULL REFERENCES registrations(id) ON DELETE RESTRICT,
        result TEXT NOT NULL CHECK (result IN ('selected', 'waitlisted', 'rejected')),
        result_order INTEGER NOT NULL CHECK (result_order >= 1),
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        UNIQUE (lottery_run_target_id, registration_id),
        UNIQUE (lottery_run_target_id, result, result_order)
    )
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_lottery_runs_term_status
    ON lottery_runs(term_id, status)
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_lottery_run_targets_run_id
    ON lottery_run_targets(lottery_run_id)
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_lottery_run_targets_offering_id
    ON lottery_run_targets(offering_id)
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_lottery_results_target_id
    ON lottery_results(lottery_run_target_id)
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_lottery_results_registration_id
    ON lottery_results(registration_id)
    """,
)
