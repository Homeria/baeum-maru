// Package database manages SQLite connections and transaction helpers.
package database

import (
	"context"
	"database/sql"
	"fmt"
	"path/filepath"

	_ "modernc.org/sqlite"
)

type Options struct {
	Path        string
	BusyTimeout int
}

func Open(ctx context.Context, opts Options) (*sql.DB, error) {
	if opts.Path == "" {
		return nil, fmt.Errorf("database path is required")
	}
	if opts.BusyTimeout == 0 {
		opts.BusyTimeout = 5000
	}

	db, err := sql.Open("sqlite", filepath.Clean(opts.Path))
	if err != nil {
		return nil, fmt.Errorf("open sqlite database: %w", err)
	}

	if err := configure(ctx, db, opts); err != nil {
		_ = db.Close()
		return nil, err
	}
	return db, nil
}

func configure(ctx context.Context, db *sql.DB, opts Options) error {
	pragmas := []string{
		"PRAGMA journal_mode = WAL;",
		fmt.Sprintf("PRAGMA busy_timeout = %d;", opts.BusyTimeout),
		"PRAGMA foreign_keys = ON;",
	}

	for _, pragma := range pragmas {
		if _, err := db.ExecContext(ctx, pragma); err != nil {
			return fmt.Errorf("apply sqlite pragma %q: %w", pragma, err)
		}
	}
	return nil
}
