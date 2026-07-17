"""건물과 층 테이블 DDL."""

STATEMENTS = (
    """
    CREATE TABLE IF NOT EXISTS buildings (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL UNIQUE,
        description TEXT,
        sort_order INTEGER NOT NULL DEFAULT 0,
        is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
    )
    """,
    """
    CREATE TABLE IF NOT EXISTS building_floors (
        id INTEGER PRIMARY KEY,
        building_id INTEGER NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
        label TEXT NOT NULL,
        sort_order INTEGER NOT NULL DEFAULT 0,
        is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        UNIQUE (building_id, label)
    )
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_building_floors_building_id
    ON building_floors(building_id)
    """,
)
