"""강좌 신청과 상태 이력 테이블 DDL."""

_STATUSES = "'applied', 'selected', 'waitlisted', 'rejected', 'confirmed', 'cancelled'"

STATEMENTS = (
    f"""
    CREATE TABLE IF NOT EXISTS registrations (
        id INTEGER PRIMARY KEY,
        member_id INTEGER NOT NULL REFERENCES members(id) ON DELETE RESTRICT,
        offering_id INTEGER NOT NULL REFERENCES course_offerings(id) ON DELETE RESTRICT,
        status TEXT NOT NULL DEFAULT 'applied' CHECK (status IN ({_STATUSES})),
        cancelled_at TEXT,
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        version INTEGER NOT NULL DEFAULT 1 CHECK (version >= 1),
        UNIQUE (member_id, offering_id),
        CHECK (
            (status = 'cancelled' AND cancelled_at IS NOT NULL)
            OR (status <> 'cancelled' AND cancelled_at IS NULL)
        )
    )
    """,
    f"""
    CREATE TABLE IF NOT EXISTS registration_status_history (
        id INTEGER PRIMARY KEY,
        registration_id INTEGER NOT NULL REFERENCES registrations(id) ON DELETE CASCADE,
        from_status TEXT CHECK (from_status IS NULL OR from_status IN ({_STATUSES})),
        to_status TEXT NOT NULL CHECK (to_status IN ({_STATUSES})),
        reason TEXT,
        actor_kind TEXT NOT NULL CHECK (actor_kind IN ('user', 'launcher', 'system')),
        actor_user_id INTEGER REFERENCES users(id) ON DELETE RESTRICT,
        actor_access_code_id INTEGER REFERENCES access_codes(id) ON DELETE RESTRICT,
        actor_display_name TEXT,
        metadata_json TEXT,
        changed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
    )
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_registrations_member_id
    ON registrations(member_id)
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_registrations_offering_status
    ON registrations(offering_id, status)
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_registration_status_history_registration_changed
    ON registration_status_history(registration_id, changed_at)
    """,
)
