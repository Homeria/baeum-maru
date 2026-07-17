"""관계자(operator), 접속 코드와 로그인 세션 테이블 DDL."""

STATEMENTS = (
    """
    CREATE TABLE IF NOT EXISTS operators (
        id INTEGER PRIMARY KEY,
        display_name TEXT NOT NULL,
        role TEXT NOT NULL CHECK (role IN ('staff', 'temporary_staff', 'viewer')),
        is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
    )
    """,
    """
    CREATE TABLE IF NOT EXISTS access_codes (
        id INTEGER PRIMARY KEY,
        operator_id INTEGER NOT NULL REFERENCES operators(id) ON DELETE RESTRICT,
        code_hash TEXT NOT NULL UNIQUE,
        issued_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        expires_at TEXT NOT NULL CHECK (expires_at > issued_at),
        revoked_at TEXT CHECK (revoked_at IS NULL OR revoked_at >= issued_at),
        last_used_at TEXT
    )
    """,
    """
    CREATE TABLE IF NOT EXISTS operator_sessions (
        id INTEGER PRIMARY KEY,
        operator_id INTEGER NOT NULL REFERENCES operators(id) ON DELETE RESTRICT,
        access_code_id INTEGER NOT NULL REFERENCES access_codes(id) ON DELETE RESTRICT,
        token_hash TEXT NOT NULL UNIQUE,
        issued_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        expires_at TEXT NOT NULL CHECK (expires_at > issued_at),
        last_seen_at TEXT NOT NULL CHECK (last_seen_at >= issued_at),
        revoked_at TEXT CHECK (revoked_at IS NULL OR revoked_at >= issued_at)
    )
    """,
    "CREATE INDEX IF NOT EXISTS ix_operators_role_is_active ON operators(role, is_active)",
    "CREATE INDEX IF NOT EXISTS ix_access_codes_operator_id ON access_codes(operator_id)",
    """
    CREATE INDEX IF NOT EXISTS ix_access_codes_lifecycle
    ON access_codes(expires_at, revoked_at)
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_operator_sessions_operator_id
    ON operator_sessions(operator_id)
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_operator_sessions_access_code_id
    ON operator_sessions(access_code_id)
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_operator_sessions_lifecycle
    ON operator_sessions(expires_at, revoked_at)
    """,
)
