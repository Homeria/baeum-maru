"""건물, 층, 공간과 공간 역할 테이블 DDL."""

STATEMENTS = (
    """
    CREATE TABLE IF NOT EXISTS buildings (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL UNIQUE,
        description TEXT,
        sort_order INTEGER NOT NULL DEFAULT 0,
        is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        version INTEGER NOT NULL DEFAULT 1 CHECK (version >= 1)
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
        version INTEGER NOT NULL DEFAULT 1 CHECK (version >= 1),
        UNIQUE (building_id, label)
    )
    """,
    """
    CREATE TABLE IF NOT EXISTS location_roles (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL UNIQUE,
        is_course_eligible INTEGER NOT NULL DEFAULT 0
            CHECK (is_course_eligible IN (0, 1)),
        sort_order INTEGER NOT NULL DEFAULT 0,
        is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        version INTEGER NOT NULL DEFAULT 1 CHECK (version >= 1)
    )
    """,
    """
    CREATE TABLE IF NOT EXISTS locations (
        id INTEGER PRIMARY KEY,
        building_floor_id INTEGER NOT NULL
            REFERENCES building_floors(id) ON DELETE RESTRICT,
        name TEXT NOT NULL,
        description TEXT,
        sort_order INTEGER NOT NULL DEFAULT 0,
        is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        version INTEGER NOT NULL DEFAULT 1 CHECK (version >= 1),
        UNIQUE (building_floor_id, name)
    )
    """,
    """
    CREATE TABLE IF NOT EXISTS location_role_assignments (
        location_id INTEGER NOT NULL REFERENCES locations(id) ON DELETE CASCADE,
        role_id INTEGER NOT NULL REFERENCES location_roles(id) ON DELETE CASCADE,
        PRIMARY KEY (location_id, role_id)
    )
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_building_floors_building_id
    ON building_floors(building_id)
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_locations_floor_active
    ON locations(building_floor_id, is_active)
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_location_role_assignments_role_id
    ON location_role_assignments(role_id)
    """,
)
