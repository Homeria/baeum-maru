CREATE TABLE IF NOT EXISTS locations (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  building TEXT,
  floor TEXT,
  type TEXT NOT NULL DEFAULT 'other' CHECK (type IN ('classroom', 'office', 'storage', 'hall', 'reception', 'other')),
  is_classroom INTEGER NOT NULL DEFAULT 0 CHECK (is_classroom IN (0, 1)),
  is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
  note TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_locations_type ON locations(type);
CREATE INDEX IF NOT EXISTS idx_locations_is_classroom ON locations(is_classroom);
CREATE INDEX IF NOT EXISTS idx_locations_is_active ON locations(is_active);
