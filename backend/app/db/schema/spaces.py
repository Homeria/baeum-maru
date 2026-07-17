"""장소 유형과 장소(강의실·사무실 등) 테이블 DDL."""

STATEMENTS = (
    """
    CREATE TABLE IF NOT EXISTS space_types (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL UNIQUE,
        is_course_eligible INTEGER NOT NULL DEFAULT 1
            CHECK (is_course_eligible IN (0, 1)),
        sort_order INTEGER NOT NULL DEFAULT 0,
        is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
    )
    """,
    """
    CREATE TABLE IF NOT EXISTS spaces (
        id INTEGER PRIMARY KEY,
        building_floor_id INTEGER NOT NULL
            REFERENCES building_floors(id) ON DELETE RESTRICT,
        space_type_id INTEGER NOT NULL
            REFERENCES space_types(id) ON DELETE RESTRICT,
        name TEXT NOT NULL,
        sort_order INTEGER NOT NULL DEFAULT 0,
        is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        UNIQUE (building_floor_id, name)
    )
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_spaces_floor_active
    ON spaces(building_floor_id, is_active)
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_spaces_space_type_id
    ON spaces(space_type_id)
    """,
)

SEED_STATEMENTS = (
    """
    INSERT OR IGNORE INTO space_types(id, name, is_course_eligible, sort_order)
    VALUES (1, '강의실', 1, 10), (2, '사무실', 0, 20),
           (3, '다목적실', 1, 30), (4, '기타', 0, 40)
    """,
)
