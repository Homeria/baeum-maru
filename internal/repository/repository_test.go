package repository

import (
	"context"
	"database/sql"
	"path/filepath"
	"testing"

	"github.com/Homeria/baeum-maru/internal/database"
	"github.com/Homeria/baeum-maru/internal/migration"
)

func newTestDB(t *testing.T) *sql.DB {
	t.Helper()

	ctx := context.Background()
	db, err := database.Open(ctx, database.Options{
		Path: filepath.Join(t.TempDir(), "center.db"),
	})
	if err != nil {
		t.Fatalf("database.Open() error = %v", err)
	}
	t.Cleanup(func() {
		if err := db.Close(); err != nil {
			t.Fatalf("close db: %v", err)
		}
	})

	if err := migration.Run(ctx, db, nil); err != nil {
		t.Fatalf("migration.Run() error = %v", err)
	}
	return db
}
