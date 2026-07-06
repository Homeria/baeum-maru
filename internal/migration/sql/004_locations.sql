CREATE TABLE IF NOT EXISTS buildings (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1))
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

INSERT OR IGNORE INTO location_roles (name, sort_order) VALUES
  ('classroom', 10),
  ('office', 20),
  ('reception', 30),
  ('event', 40),
  ('storage', 50),
  ('common', 60);

CREATE INDEX IF NOT EXISTS idx_locations_building ON locations(building_id);
CREATE INDEX IF NOT EXISTS idx_locations_is_active ON locations(is_active);
CREATE INDEX IF NOT EXISTS idx_location_role_assignments_role ON location_role_assignments(role_id);
