"""사용자, 접속 코드와 로그인 세션 테이블 DDL."""

STATEMENTS = (
    """
    CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY,
        display_name TEXT NOT NULL,
        affiliation TEXT,
        contact_note TEXT,
        role TEXT NOT NULL CHECK (role IN ('staff', 'temporary_staff', 'viewer')),
        is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        version INTEGER NOT NULL DEFAULT 1 CHECK (version >= 1)
    )
    """,
    """
    CREATE TABLE IF NOT EXISTS access_codes (
        id INTEGER PRIMARY KEY,
        user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
        code_hash TEXT NOT NULL UNIQUE,
        display_code TEXT NOT NULL UNIQUE,
        label TEXT,
        issued_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        expires_at TEXT NOT NULL CHECK (expires_at > issued_at),
        revoked_at TEXT CHECK (revoked_at IS NULL OR revoked_at >= issued_at),
        hidden_at TEXT CHECK (hidden_at IS NULL OR hidden_at >= issued_at),
        last_used_at TEXT,
        note TEXT,
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        version INTEGER NOT NULL DEFAULT 1 CHECK (version >= 1)
    )
    """,
    """
    CREATE TABLE IF NOT EXISTS user_sessions (
        id TEXT PRIMARY KEY,
        user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
        access_code_id INTEGER NOT NULL REFERENCES access_codes(id) ON DELETE RESTRICT,
        token_hash TEXT NOT NULL UNIQUE,
        issued_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        expires_at TEXT NOT NULL CHECK (expires_at > issued_at),
        last_seen_at TEXT NOT NULL CHECK (last_seen_at >= issued_at),
        revoked_at TEXT CHECK (revoked_at IS NULL OR revoked_at >= issued_at)
    )
    """,
    "CREATE INDEX IF NOT EXISTS ix_users_role_is_active ON users(role, is_active)",
    "CREATE INDEX IF NOT EXISTS ix_access_codes_user_id ON access_codes(user_id)",
    """
    CREATE INDEX IF NOT EXISTS ix_access_codes_lifecycle
    ON access_codes(expires_at, revoked_at, hidden_at)
    """,
    "CREATE INDEX IF NOT EXISTS ix_user_sessions_user_id ON user_sessions(user_id)",
    """
    CREATE INDEX IF NOT EXISTS ix_user_sessions_access_code_id
    ON user_sessions(access_code_id)
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_user_sessions_lifecycle
    ON user_sessions(expires_at, revoked_at)
    """,
)
