"""강좌 기준정보, 개설 강좌와 시간표 테이블 DDL."""

_CAPACITY_CHECK = """
    (
        capacity_type = 'fixed'
        AND capacity_total IS NOT NULL AND capacity_total > 0
        AND male_capacity IS NULL AND female_capacity IS NULL
    ) OR (
        capacity_type = 'open'
        AND capacity_total IS NULL
        AND male_capacity IS NULL AND female_capacity IS NULL
    ) OR (
        capacity_type = 'gender_split'
        AND capacity_total IS NULL
        AND male_capacity IS NOT NULL AND male_capacity >= 0
        AND female_capacity IS NOT NULL AND female_capacity >= 0
        AND male_capacity + female_capacity > 0
    )
"""

STATEMENTS = (
    """
    CREATE TABLE IF NOT EXISTS course_categories (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL UNIQUE,
        sort_order INTEGER NOT NULL DEFAULT 0,
        is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
    )
    """,
    """
    CREATE TABLE IF NOT EXISTS course_levels (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL UNIQUE,
        sort_order INTEGER NOT NULL DEFAULT 0,
        is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
    )
    """,
    """
    CREATE TABLE IF NOT EXISTS courses (
        id INTEGER PRIMARY KEY,
        category_id INTEGER NOT NULL REFERENCES course_categories(id) ON DELETE RESTRICT,
        level_id INTEGER REFERENCES course_levels(id) ON DELETE RESTRICT,
        name TEXT NOT NULL,
        description TEXT,
        is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
    )
    """,
    """
    CREATE TABLE IF NOT EXISTS instructors (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        phone TEXT,
        is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
    )
    """,
    """
    CREATE TABLE IF NOT EXISTS terms (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL UNIQUE,
        starts_on TEXT,
        ends_on TEXT,
        registration_opens_at TEXT,
        registration_closes_at TEXT,
        max_registrations_per_member INTEGER NOT NULL DEFAULT 0
            CHECK (max_registrations_per_member >= 0),
        status TEXT NOT NULL DEFAULT 'draft'
            CHECK (status IN ('draft', 'open', 'closed', 'finalized')),
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        CHECK (starts_on IS NULL OR ends_on IS NULL OR starts_on <= ends_on),
        CHECK (
            registration_opens_at IS NULL OR registration_closes_at IS NULL
            OR registration_opens_at < registration_closes_at
        )
    )
    """,
    """
    CREATE TABLE IF NOT EXISTS time_slots (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL UNIQUE,
        start_time TEXT NOT NULL,
        end_time TEXT NOT NULL,
        sort_order INTEGER NOT NULL DEFAULT 0,
        is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        CHECK (start_time < end_time),
        UNIQUE (start_time, end_time)
    )
    """,
    f"""
    CREATE TABLE IF NOT EXISTS course_offerings (
        id INTEGER PRIMARY KEY,
        term_id INTEGER NOT NULL REFERENCES terms(id) ON DELETE RESTRICT,
        course_id INTEGER NOT NULL REFERENCES courses(id) ON DELETE RESTRICT,
        section_label TEXT CHECK (
            section_label IS NULL OR length(trim(section_label)) > 0
        ),
        instructor_id INTEGER REFERENCES instructors(id) ON DELETE RESTRICT,
        capacity_type TEXT NOT NULL DEFAULT 'fixed'
            CHECK (capacity_type IN ('fixed', 'open', 'gender_split')),
        capacity_total INTEGER,
        male_capacity INTEGER,
        female_capacity INTEGER,
        status TEXT NOT NULL DEFAULT 'draft'
            CHECK (status IN ('draft', 'open', 'closed', 'cancelled')),
        sort_order INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        CHECK ({_CAPACITY_CHECK})
    )
    """,
    """
    CREATE TABLE IF NOT EXISTS course_schedules (
        id INTEGER PRIMARY KEY,
        offering_id INTEGER NOT NULL REFERENCES course_offerings(id) ON DELETE CASCADE,
        weekday INTEGER NOT NULL CHECK (weekday BETWEEN 1 AND 7),
        time_slot_id INTEGER NOT NULL REFERENCES time_slots(id) ON DELETE RESTRICT,
        space_id INTEGER NOT NULL REFERENCES spaces(id) ON DELETE RESTRICT,
        UNIQUE (offering_id, weekday, time_slot_id)
    )
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_courses_category_active
    ON courses(category_id, is_active)
    """,
    "CREATE INDEX IF NOT EXISTS ix_courses_level_id ON courses(level_id)",
    """
    CREATE UNIQUE INDEX IF NOT EXISTS uq_courses_category_name_no_level
    ON courses(category_id, name) WHERE level_id IS NULL
    """,
    """
    CREATE UNIQUE INDEX IF NOT EXISTS uq_courses_category_name_level
    ON courses(category_id, name, level_id) WHERE level_id IS NOT NULL
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_course_offerings_term_status
    ON course_offerings(term_id, status)
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_course_offerings_course_id
    ON course_offerings(course_id)
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_course_offerings_instructor_id
    ON course_offerings(instructor_id)
    """,
    """
    CREATE UNIQUE INDEX IF NOT EXISTS uq_course_offerings_term_course_no_section
    ON course_offerings(term_id, course_id) WHERE section_label IS NULL
    """,
    """
    CREATE UNIQUE INDEX IF NOT EXISTS uq_course_offerings_term_course_section
    ON course_offerings(term_id, course_id, section_label) WHERE section_label IS NOT NULL
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_course_schedules_offering_id
    ON course_schedules(offering_id)
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_course_schedules_weekday_slot
    ON course_schedules(weekday, time_slot_id)
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_course_schedules_space_id
    ON course_schedules(space_id)
    """,
)
