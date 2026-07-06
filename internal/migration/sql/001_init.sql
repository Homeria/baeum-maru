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
  sort_order INTEGER NOT NULL DEFAULT 0,
  is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1))
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
  name TEXT NOT NULL,
  category_id INTEGER NOT NULL REFERENCES course_categories(id),
  description TEXT,
  is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (category_id, name)
);

CREATE TABLE IF NOT EXISTS instructors (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  phone TEXT,
  note TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS buildings (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1))
);

CREATE TABLE IF NOT EXISTS building_floors (
  id INTEGER PRIMARY KEY,
  building_id INTEGER NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
  label TEXT NOT NULL,
  sort_order INTEGER NOT NULL DEFAULT 0,
  is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
  UNIQUE (building_id, label)
);

CREATE TABLE IF NOT EXISTS locations (
  id INTEGER PRIMARY KEY,
  building_id INTEGER REFERENCES buildings(id),
  name TEXT NOT NULL,
  floor_label TEXT,
  description TEXT,
  is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (building_id, name, floor_label)
);

CREATE TABLE IF NOT EXISTS location_roles (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
  sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS location_role_assignments (
  location_id INTEGER NOT NULL REFERENCES locations(id) ON DELETE CASCADE,
  role_id INTEGER NOT NULL REFERENCES location_roles(id),
  PRIMARY KEY (location_id, role_id)
);

CREATE TABLE IF NOT EXISTS time_slots (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  start_time TEXT NOT NULL,
  end_time TEXT NOT NULL,
  sort_order INTEGER NOT NULL DEFAULT 0,
  is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
  CHECK (start_time < end_time)
);

CREATE TABLE IF NOT EXISTS course_offerings (
  id INTEGER PRIMARY KEY,
  term_id INTEGER NOT NULL REFERENCES terms(id),
  course_id INTEGER NOT NULL REFERENCES courses(id),
  display_name TEXT NOT NULL,
  level_label TEXT,
  section_label TEXT,
  instructor_id INTEGER REFERENCES instructors(id),
  location_id INTEGER REFERENCES locations(id),
  capacity_type TEXT NOT NULL DEFAULT 'fixed' CHECK (capacity_type IN ('fixed', 'open', 'gender_split')),
  capacity_total INTEGER CHECK (capacity_total IS NULL OR capacity_total >= 0),
  male_capacity INTEGER CHECK (male_capacity IS NULL OR male_capacity >= 0),
  female_capacity INTEGER CHECK (female_capacity IS NULL OR female_capacity >= 0),
  registration_enabled INTEGER NOT NULL DEFAULT 1 CHECK (registration_enabled IN (0, 1)),
  status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'open', 'closed', 'cancelled')),
  note TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS course_schedules (
  id INTEGER PRIMARY KEY,
  offering_id INTEGER NOT NULL REFERENCES course_offerings(id) ON DELETE CASCADE,
  weekday INTEGER NOT NULL CHECK (weekday >= 1 AND weekday <= 7),
  time_slot_id INTEGER NOT NULL REFERENCES time_slots(id),
  UNIQUE (offering_id, weekday, time_slot_id)
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
  actor_kind TEXT NOT NULL DEFAULT 'user' CHECK (actor_kind IN ('user', 'access', 'system')),
  actor_display_name TEXT,
  metadata_json TEXT,
  changed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS users (
  id INTEGER PRIMARY KEY,
  username TEXT NOT NULL UNIQUE,
  password_hash TEXT NOT NULL,
  display_name TEXT NOT NULL,
  role TEXT NOT NULL CHECK (role IN ('admin', 'staff')),
  affiliation TEXT,
  contact_note TEXT,
  access_role TEXT NOT NULL DEFAULT 'staff' CHECK (access_role IN ('staff', 'temporary_staff', 'viewer')),
  status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'expired', 'disabled')),
  user_kind TEXT NOT NULL DEFAULT 'access' CHECK (user_kind IN ('permanent', 'access')),
  expires_at TEXT,
  last_login_at TEXT,
  is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS access_codes (
  id INTEGER PRIMARY KEY,
  user_id INTEGER NOT NULL REFERENCES users(id),
  code_hash TEXT NOT NULL UNIQUE,
  display_code TEXT,
  label TEXT,
  status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'expired', 'revoked')),
  issued_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  expires_at TEXT NOT NULL,
  revoked_at TEXT,
  last_used_at TEXT,
  note TEXT,
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

CREATE TABLE IF NOT EXISTS lottery_run_targets (
  lottery_run_id INTEGER NOT NULL REFERENCES lottery_runs(id) ON DELETE CASCADE,
  offering_id INTEGER NOT NULL REFERENCES course_offerings(id),
  PRIMARY KEY (lottery_run_id, offering_id)
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
  actor_kind TEXT NOT NULL DEFAULT 'user' CHECK (actor_kind IN ('user', 'access', 'system')),
  actor_display_name TEXT,
  action TEXT NOT NULL,
  entity_type TEXT NOT NULL,
  entity_id INTEGER,
  summary TEXT NOT NULL,
  metadata_json TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS operation_jobs (
  id INTEGER PRIMARY KEY,
  job_type TEXT NOT NULL CHECK (job_type IN ('import', 'export', 'backup', 'restore', 'notification')),
  status TEXT NOT NULL DEFAULT 'queued' CHECK (status IN ('queued', 'running', 'completed', 'failed', 'cancelled')),
  source_name TEXT,
  output_path TEXT,
  requested_by_user_id INTEGER REFERENCES users(id),
  total_count INTEGER NOT NULL DEFAULT 0 CHECK (total_count >= 0),
  success_count INTEGER NOT NULL DEFAULT 0 CHECK (success_count >= 0),
  failure_count INTEGER NOT NULL DEFAULT 0 CHECK (failure_count >= 0),
  error_summary TEXT,
  metadata_json TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  started_at TEXT,
  completed_at TEXT
);

CREATE TABLE IF NOT EXISTS operation_job_errors (
  id INTEGER PRIMARY KEY,
  job_id INTEGER NOT NULL REFERENCES operation_jobs(id) ON DELETE CASCADE,
  row_number INTEGER,
  field_name TEXT,
  message TEXT NOT NULL,
  raw_value TEXT,
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

INSERT OR IGNORE INTO location_roles (name, sort_order) VALUES
  ('classroom', 10),
  ('office', 20),
  ('reception', 30),
  ('event', 40),
  ('storage', 50),
  ('common', 60);

CREATE INDEX IF NOT EXISTS idx_members_name ON members(name);
CREATE INDEX IF NOT EXISTS idx_members_phone ON members(phone);
CREATE INDEX IF NOT EXISTS idx_courses_name ON courses(name);
CREATE INDEX IF NOT EXISTS idx_users_user_kind ON users(user_kind);
CREATE INDEX IF NOT EXISTS idx_users_status ON users(status);
CREATE INDEX IF NOT EXISTS idx_access_codes_hash ON access_codes(code_hash);
CREATE INDEX IF NOT EXISTS idx_access_codes_user ON access_codes(user_id);
CREATE INDEX IF NOT EXISTS idx_access_codes_status ON access_codes(status);
CREATE INDEX IF NOT EXISTS idx_access_codes_expires_at ON access_codes(expires_at);
CREATE INDEX IF NOT EXISTS idx_course_offerings_term ON course_offerings(term_id);
CREATE INDEX IF NOT EXISTS idx_course_offerings_course ON course_offerings(course_id);
CREATE INDEX IF NOT EXISTS idx_course_offerings_location ON course_offerings(location_id);
CREATE INDEX IF NOT EXISTS idx_course_schedules_offering ON course_schedules(offering_id);
CREATE INDEX IF NOT EXISTS idx_registrations_member ON registrations(member_id);
CREATE INDEX IF NOT EXISTS idx_registrations_offering ON registrations(offering_id);
CREATE INDEX IF NOT EXISTS idx_registrations_status ON registrations(status);
CREATE INDEX IF NOT EXISTS idx_registration_status_history_registration ON registration_status_history(registration_id);
CREATE INDEX IF NOT EXISTS idx_lottery_run_targets_offering ON lottery_run_targets(offering_id);
CREATE INDEX IF NOT EXISTS idx_lottery_results_run ON lottery_results(lottery_run_id);
CREATE INDEX IF NOT EXISTS idx_building_floors_building ON building_floors(building_id);
CREATE INDEX IF NOT EXISTS idx_locations_building ON locations(building_id);
CREATE INDEX IF NOT EXISTS idx_locations_is_active ON locations(is_active);
CREATE INDEX IF NOT EXISTS idx_location_role_assignments_role ON location_role_assignments(role_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_entity ON audit_logs(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_operation_jobs_type_status ON operation_jobs(job_type, status);
CREATE INDEX IF NOT EXISTS idx_operation_job_errors_job ON operation_job_errors(job_id);
