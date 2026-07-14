"""출석 회차와 회원 출석 기록 테이블 DDL."""

STATEMENTS = (
    """
    CREATE TABLE IF NOT EXISTS attendance_sessions (
        id INTEGER PRIMARY KEY,
        offering_id INTEGER NOT NULL REFERENCES course_offerings(id) ON DELETE RESTRICT,
        schedule_id INTEGER REFERENCES course_schedules(id) ON DELETE RESTRICT,
        session_date TEXT NOT NULL,
        sequence_no INTEGER NOT NULL DEFAULT 1 CHECK (sequence_no >= 1),
        note TEXT,
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        version INTEGER NOT NULL DEFAULT 1 CHECK (version >= 1),
        UNIQUE (offering_id, session_date, sequence_no)
    )
    """,
    """
    CREATE TABLE IF NOT EXISTS attendance_records (
        id INTEGER PRIMARY KEY,
        attendance_session_id INTEGER NOT NULL
            REFERENCES attendance_sessions(id) ON DELETE CASCADE,
        registration_id INTEGER NOT NULL REFERENCES registrations(id) ON DELETE RESTRICT,
        status TEXT NOT NULL CHECK (status IN ('present', 'absent', 'late', 'excused')),
        note TEXT,
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        version INTEGER NOT NULL DEFAULT 1 CHECK (version >= 1),
        UNIQUE (attendance_session_id, registration_id)
    )
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_attendance_sessions_offering_date
    ON attendance_sessions(offering_id, session_date)
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_attendance_records_registration_id
    ON attendance_records(registration_id)
    """,
)
