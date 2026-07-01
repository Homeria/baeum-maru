// Package migration runs versioned database schema migrations.
package migration

import (
	"context"
	"database/sql"
	"embed"
	"fmt"
	"io/fs"
	"path/filepath"
	"sort"
	"strings"
)

//go:embed sql/*.sql
var Files embed.FS

type Migration struct {
	Version string
	Name    string
	SQL     string
}

func Run(ctx context.Context, db *sql.DB, files fs.FS) error {
	if files == nil {
		files = Files
	}
	if err := ensureSchemaMigrations(ctx, db); err != nil {
		return err
	}

	migrations, err := Load(files)
	if err != nil {
		return err
	}

	for _, migration := range migrations {
		applied, err := isApplied(ctx, db, migration.Version)
		if err != nil {
			return err
		}
		if applied {
			continue
		}

		if err := apply(ctx, db, migration); err != nil {
			return err
		}
	}
	return nil
}

func Load(files fs.FS) ([]Migration, error) {
	entries, err := fs.Glob(files, "sql/*.sql")
	if err != nil {
		return nil, fmt.Errorf("glob migrations: %w", err)
	}
	sort.Strings(entries)

	migrations := make([]Migration, 0, len(entries))
	for _, path := range entries {
		name := filepath.Base(path)
		version := strings.TrimSuffix(name, filepath.Ext(name))
		content, err := fs.ReadFile(files, path)
		if err != nil {
			return nil, fmt.Errorf("read migration %s: %w", path, err)
		}
		migrations = append(migrations, Migration{
			Version: version,
			Name:    name,
			SQL:     string(content),
		})
	}
	return migrations, nil
}

func ensureSchemaMigrations(ctx context.Context, db *sql.DB) error {
	_, err := db.ExecContext(ctx, `
CREATE TABLE IF NOT EXISTS schema_migrations (
	version TEXT PRIMARY KEY,
	name TEXT NOT NULL,
	applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);`)
	if err != nil {
		return fmt.Errorf("ensure schema_migrations: %w", err)
	}
	return nil
}

func isApplied(ctx context.Context, db *sql.DB, version string) (bool, error) {
	var exists int
	err := db.QueryRowContext(ctx, "SELECT 1 FROM schema_migrations WHERE version = ?;", version).Scan(&exists)
	if err == nil {
		return true, nil
	}
	if err == sql.ErrNoRows {
		return false, nil
	}
	return false, fmt.Errorf("check migration %s: %w", version, err)
}

func apply(ctx context.Context, db *sql.DB, migration Migration) error {
	tx, err := db.BeginTx(ctx, nil)
	if err != nil {
		return fmt.Errorf("begin migration %s: %w", migration.Name, err)
	}

	if _, err := tx.ExecContext(ctx, migration.SQL); err != nil {
		_ = tx.Rollback()
		return fmt.Errorf("apply migration %s: %w", migration.Name, err)
	}
	if _, err := tx.ExecContext(ctx,
		"INSERT INTO schema_migrations (version, name) VALUES (?, ?);",
		migration.Version,
		migration.Name,
	); err != nil {
		_ = tx.Rollback()
		return fmt.Errorf("record migration %s: %w", migration.Name, err)
	}

	if err := tx.Commit(); err != nil {
		return fmt.Errorf("commit migration %s: %w", migration.Name, err)
	}
	return nil
}
