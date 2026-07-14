"""성별 기준 코드와 회원 테이블 DDL."""

STATEMENTS = (
    """
    CREATE TABLE IF NOT EXISTS gender_codes (
        code TEXT PRIMARY KEY,
        label TEXT NOT NULL UNIQUE,
        sort_order INTEGER NOT NULL DEFAULT 0
    )
    """,
    """
    CREATE TABLE IF NOT EXISTS members (
        id INTEGER PRIMARY KEY,
        member_no TEXT NOT NULL UNIQUE,
        name TEXT NOT NULL,
        gender_code TEXT NOT NULL DEFAULT 'unknown'
            REFERENCES gender_codes(code) ON DELETE RESTRICT,
        phone TEXT NOT NULL,
        note TEXT,
        is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        version INTEGER NOT NULL DEFAULT 1 CHECK (version >= 1)
    )
    """,
    "CREATE INDEX IF NOT EXISTS ix_members_name ON members(name)",
    "CREATE INDEX IF NOT EXISTS ix_members_phone ON members(phone)",
    "CREATE INDEX IF NOT EXISTS ix_members_is_active ON members(is_active)",
)

SEED_STATEMENTS = (
    """
    INSERT OR IGNORE INTO gender_codes(code, label, sort_order)
    VALUES ('male', '남성', 10), ('female', '여성', 20), ('unknown', '미상', 30)
    """,
)
