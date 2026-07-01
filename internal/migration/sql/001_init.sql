CREATE TABLE IF NOT EXISTS gender_codes (
  code TEXT PRIMARY KEY,
  label TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS terms (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  registration_start_at TEXT,
  registration_end_at TEXT,
  max_registrations_per_member INTEGER NOT NULL DEFAULT 0 CHECK (max_registrations_per_member >= 0),
  status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'open', 'closed', 'finalized')),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS course_categories (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS members (
  id INTEGER PRIMARY KEY,
  member_no TEXT UNIQUE,
  name TEXT NOT NULL,
  gender_code TEXT REFERENCES gender_codes(code),
  birth_date TEXT,
  phone TEXT,
  note TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS courses (
  id INTEGER PRIMARY KEY,
  title TEXT NOT NULL,
  category_id INTEGER REFERENCES course_categories(id),
  description TEXT,
  is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS instructors (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  phone TEXT,
  note TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS classrooms (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  note TEXT
);

CREATE TABLE IF NOT EXISTS course_offerings (
  id INTEGER PRIMARY KEY,
  term_id INTEGER NOT NULL REFERENCES terms(id),
  course_id INTEGER NOT NULL REFERENCES courses(id),
  instructor_id INTEGER REFERENCES instructors(id),
  classroom_id INTEGER REFERENCES classrooms(id),
  capacity INTEGER NOT NULL CHECK (capacity >= 0),
  registration_enabled INTEGER NOT NULL DEFAULT 1 CHECK (registration_enabled IN (0, 1)),
  status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'open', 'closed', 'cancelled')),
  note TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (term_id, course_id)
);

CREATE TABLE IF NOT EXISTS course_meetings (
  id INTEGER PRIMARY KEY,
  offering_id INTEGER NOT NULL REFERENCES course_offerings(id) ON DELETE CASCADE,
  weekday INTEGER NOT NULL CHECK (weekday >= 0 AND weekday <= 6),
  start_time TEXT NOT NULL,
  end_time TEXT NOT NULL,
  CHECK (start_time < end_time),
  UNIQUE (offering_id, weekday, start_time, end_time)
);

CREATE TABLE IF NOT EXISTS registrations (
  id INTEGER PRIMARY KEY,
  member_id INTEGER NOT NULL REFERENCES members(id),
  offering_id INTEGER NOT NULL REFERENCES course_offerings(id),
  status TEXT NOT NULL DEFAULT 'applied' CHECK (status IN ('applied', 'cancelled', 'selected', 'waitlisted', 'rejected', 'confirmed')),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  cancelled_at TEXT,
  UNIQUE (member_id, offering_id)
);

CREATE TABLE IF NOT EXISTS registration_status_history (
  id INTEGER PRIMARY KEY,
  registration_id INTEGER NOT NULL REFERENCES registrations(id) ON DELETE CASCADE,
  from_status TEXT,
  to_status TEXT NOT NULL,
  reason TEXT,
  changed_by_user_id INTEGER REFERENCES users(id),
  changed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS users (
  id INTEGER PRIMARY KEY,
  username TEXT NOT NULL UNIQUE,
  password_hash TEXT NOT NULL,
  display_name TEXT NOT NULL,
  role TEXT NOT NULL CHECK (role IN ('admin', 'staff')),
  is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS lottery_runs (
  id INTEGER PRIMARY KEY,
  term_id INTEGER NOT NULL REFERENCES terms(id),
  seed INTEGER NOT NULL,
  status TEXT NOT NULL DEFAULT 'prepared' CHECK (status IN ('prepared', 'completed', 'cancelled')),
  started_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  completed_at TEXT,
  executed_by_user_id INTEGER REFERENCES users(id),
  note TEXT
);

CREATE TABLE IF NOT EXISTS lottery_results (
  id INTEGER PRIMARY KEY,
  lottery_run_id INTEGER NOT NULL REFERENCES lottery_runs(id) ON DELETE CASCADE,
  registration_id INTEGER NOT NULL REFERENCES registrations(id),
  result TEXT NOT NULL CHECK (result IN ('selected', 'waitlisted', 'rejected')),
  result_order INTEGER NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (lottery_run_id, registration_id)
);

CREATE TABLE IF NOT EXISTS attendance_sessions (
  id INTEGER PRIMARY KEY,
  offering_id INTEGER NOT NULL REFERENCES course_offerings(id) ON DELETE CASCADE,
  session_date TEXT NOT NULL,
  note TEXT,
  UNIQUE (offering_id, session_date)
);

CREATE TABLE IF NOT EXISTS attendance_records (
  id INTEGER PRIMARY KEY,
  attendance_session_id INTEGER NOT NULL REFERENCES attendance_sessions(id) ON DELETE CASCADE,
  registration_id INTEGER NOT NULL REFERENCES registrations(id),
  status TEXT NOT NULL CHECK (status IN ('present', 'absent', 'late', 'excused')),
  note TEXT,
  UNIQUE (attendance_session_id, registration_id)
);

CREATE TABLE IF NOT EXISTS audit_logs (
  id INTEGER PRIMARY KEY,
  actor_user_id INTEGER REFERENCES users(id),
  action TEXT NOT NULL,
  entity_type TEXT NOT NULL,
  entity_id INTEGER,
  summary TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT OR IGNORE INTO gender_codes (code, label) VALUES
  ('male', '남성'),
  ('female', '여성'),
  ('unknown', '미상');

CREATE INDEX IF NOT EXISTS idx_members_name ON members(name);
CREATE INDEX IF NOT EXISTS idx_members_phone ON members(phone);
CREATE INDEX IF NOT EXISTS idx_courses_title ON courses(title);
CREATE INDEX IF NOT EXISTS idx_course_offerings_term ON course_offerings(term_id);
CREATE INDEX IF NOT EXISTS idx_registrations_member ON registrations(member_id);
CREATE INDEX IF NOT EXISTS idx_registrations_offering ON registrations(offering_id);
CREATE INDEX IF NOT EXISTS idx_registrations_status ON registrations(status);
CREATE INDEX IF NOT EXISTS idx_lottery_results_run ON lottery_results(lottery_run_id);
