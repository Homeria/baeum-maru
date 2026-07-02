ALTER TABLE users ADD COLUMN affiliation TEXT;
ALTER TABLE users ADD COLUMN contact_note TEXT;
ALTER TABLE users ADD COLUMN access_role TEXT NOT NULL DEFAULT 'staff' CHECK (access_role IN ('staff', 'temporary_staff', 'viewer'));
ALTER TABLE users ADD COLUMN status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'expired', 'disabled'));
ALTER TABLE users ADD COLUMN expires_at TEXT;
ALTER TABLE users ADD COLUMN last_login_at TEXT;

CREATE TABLE IF NOT EXISTS access_codes (
  id INTEGER PRIMARY KEY,
  user_id INTEGER NOT NULL REFERENCES users(id),
  code_hash TEXT NOT NULL UNIQUE,
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

CREATE INDEX IF NOT EXISTS idx_access_codes_hash ON access_codes(code_hash);
CREATE INDEX IF NOT EXISTS idx_access_codes_user ON access_codes(user_id);
CREATE INDEX IF NOT EXISTS idx_access_codes_expires_at ON access_codes(expires_at);
