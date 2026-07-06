ALTER TABLE users ADD COLUMN user_kind TEXT NOT NULL DEFAULT 'access' CHECK (user_kind IN ('permanent', 'access'));

ALTER TABLE registration_status_history ADD COLUMN actor_kind TEXT NOT NULL DEFAULT 'user' CHECK (actor_kind IN ('user', 'access', 'system'));
ALTER TABLE registration_status_history ADD COLUMN actor_display_name TEXT;
ALTER TABLE registration_status_history ADD COLUMN metadata_json TEXT;

ALTER TABLE audit_logs ADD COLUMN actor_kind TEXT NOT NULL DEFAULT 'user' CHECK (actor_kind IN ('user', 'access', 'system'));
ALTER TABLE audit_logs ADD COLUMN actor_display_name TEXT;
ALTER TABLE audit_logs ADD COLUMN metadata_json TEXT;

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

CREATE INDEX IF NOT EXISTS idx_users_user_kind ON users(user_kind);
CREATE INDEX IF NOT EXISTS idx_users_status ON users(status);
CREATE INDEX IF NOT EXISTS idx_access_codes_status ON access_codes(status);
CREATE INDEX IF NOT EXISTS idx_registration_status_history_registration ON registration_status_history(registration_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_entity ON audit_logs(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_operation_jobs_type_status ON operation_jobs(job_type, status);
CREATE INDEX IF NOT EXISTS idx_operation_job_errors_job ON operation_job_errors(job_id);
