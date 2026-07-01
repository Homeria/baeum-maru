package migration

import (
	"context"
	"database/sql"
	"testing"
	"testing/fstest"

	_ "modernc.org/sqlite"
)

func TestLoadReturnsSortedMigrations(t *testing.T) {
	files := fstest.MapFS{
		"sql/002_second.sql": {Data: []byte("SELECT 2;")},
		"sql/001_first.sql":  {Data: []byte("SELECT 1;")},
	}

	migrations, err := Load(files)
	if err != nil {
		t.Fatalf("Load() error = %v", err)
	}
	if len(migrations) != 2 {
		t.Fatalf("len = %d, want 2", len(migrations))
	}
	if migrations[0].Version != "001_first" {
		t.Fatalf("first version = %q, want 001_first", migrations[0].Version)
	}
}

func TestRunAppliesMigrationsOnce(t *testing.T) {
	ctx := context.Background()
	db, err := sql.Open("sqlite", ":memory:")
	if err != nil {
		t.Fatalf("Open() error = %v", err)
	}
	defer db.Close()

	files := fstest.MapFS{
		"sql/001_init.sql": {Data: []byte(`
CREATE TABLE members (
	id INTEGER PRIMARY KEY,
	name TEXT NOT NULL
);
`)},
	}

	if err := Run(ctx, db, files); err != nil {
		t.Fatalf("Run() first error = %v", err)
	}
	if err := Run(ctx, db, files); err != nil {
		t.Fatalf("Run() second error = %v", err)
	}

	var count int
	if err := db.QueryRowContext(ctx, "SELECT COUNT(*) FROM schema_migrations;").Scan(&count); err != nil {
		t.Fatalf("count migrations: %v", err)
	}
	if count != 1 {
		t.Fatalf("count = %d, want 1", count)
	}
}

func TestRunEmbeddedMigrationsCreatesCoreTables(t *testing.T) {
	ctx := context.Background()
	db, err := sql.Open("sqlite", ":memory:")
	if err != nil {
		t.Fatalf("Open() error = %v", err)
	}
	defer db.Close()

	if err := Run(ctx, db, nil); err != nil {
		t.Fatalf("Run() error = %v", err)
	}

	for _, table := range []string{"members", "courses", "course_offerings", "registrations"} {
		t.Run(table, func(t *testing.T) {
			var name string
			err := db.QueryRowContext(ctx,
				"SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?;",
				table,
			).Scan(&name)
			if err != nil {
				t.Fatalf("find table %s: %v", table, err)
			}
			if name != table {
				t.Fatalf("table = %q, want %q", name, table)
			}
		})
	}
}
