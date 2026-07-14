"""기관 설정 테이블 DDL."""

STATEMENTS = (
    """
    CREATE TABLE IF NOT EXISTS organization_settings (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        organization_name TEXT NOT NULL,
        logo_relative_path TEXT,
        default_max_registrations INTEGER NOT NULL DEFAULT 4
            CHECK (default_max_registrations >= 0),
        default_access_code_ttl_minutes INTEGER NOT NULL DEFAULT 480
            CHECK (default_access_code_ttl_minutes BETWEEN 1 AND 10080),
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        version INTEGER NOT NULL DEFAULT 1 CHECK (version >= 1)
    )
    """,
)
