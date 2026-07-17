"""회원 테이블 DDL."""

STATEMENTS = (
    """
    CREATE TABLE IF NOT EXISTS members (
        id INTEGER PRIMARY KEY,
        member_no TEXT NOT NULL UNIQUE,
        name TEXT NOT NULL,
        gender TEXT NOT NULL CHECK (gender IN ('male', 'female')),
        phone TEXT NOT NULL,
        is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
    )
    """,
    "CREATE INDEX IF NOT EXISTS ix_members_name ON members(name)",
    "CREATE INDEX IF NOT EXISTS ix_members_phone ON members(phone)",
    "CREATE INDEX IF NOT EXISTS ix_members_is_active ON members(is_active)",
)
