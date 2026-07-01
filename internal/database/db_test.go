package database

import (
	"context"
	"database/sql"
	"path/filepath"
	"testing"
)

func TestOpenCreatesSQLiteDatabase(t *testing.T) {
	ctx := context.Background()
	path := filepath.Join(t.TempDir(), "center.db")

	db, err := Open(ctx, Options{Path: path})
	if err != nil {
		t.Fatalf("Open() error = %v", err)
	}
	defer db.Close()

	if err := db.PingContext(ctx); err != nil {
		t.Fatalf("PingContext() error = %v", err)
	}
}

func TestOpenEnablesForeignKeys(t *testing.T) {
	ctx := context.Background()
	path := filepath.Join(t.TempDir(), "center.db")

	db, err := Open(ctx, Options{Path: path})
	if err != nil {
		t.Fatalf("Open() error = %v", err)
	}
	defer db.Close()

	var enabled int
	if err := db.QueryRowContext(ctx, "PRAGMA foreign_keys;").Scan(&enabled); err != nil {
		t.Fatalf("scan foreign_keys pragma: %v", err)
	}
	if enabled != 1 {
		t.Fatalf("foreign_keys = %d, want 1", enabled)
	}
}

func TestWithTxCommits(t *testing.T) {
	ctx := context.Background()
	path := filepath.Join(t.TempDir(), "center.db")

	db, err := Open(ctx, Options{Path: path})
	if err != nil {
		t.Fatalf("Open() error = %v", err)
	}
	defer db.Close()

	if _, err := db.ExecContext(ctx, "CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT NOT NULL);"); err != nil {
		t.Fatalf("create table: %v", err)
	}

	if err := WithTx(ctx, db, func(tx *sql.Tx) error {
		_, err := tx.ExecContext(ctx, "INSERT INTO items (name) VALUES (?);", "first")
		return err
	}); err != nil {
		t.Fatalf("WithTx() error = %v", err)
	}

	var count int
	if err := db.QueryRowContext(ctx, "SELECT COUNT(*) FROM items;").Scan(&count); err != nil {
		t.Fatalf("count items: %v", err)
	}
	if count != 1 {
		t.Fatalf("count = %d, want 1", count)
	}
}
