package service

import (
	"context"
	"os"
	"path/filepath"
	"testing"
	"time"

	"github.com/Homeria/baeum-maru/internal/database"
)

func TestBackupServiceCreatesListsAndQueuesRestore(t *testing.T) {
	ctx := context.Background()
	root := t.TempDir()
	dbPath := filepath.Join(root, "data", "center.db")
	backupDir := filepath.Join(root, "backups")
	if err := os.MkdirAll(filepath.Dir(dbPath), 0o755); err != nil {
		t.Fatalf("MkdirAll() error = %v", err)
	}
	db, err := database.Open(ctx, database.Options{Path: dbPath})
	if err != nil {
		t.Fatalf("database.Open() error = %v", err)
	}
	defer db.Close()
	if _, err := db.ExecContext(ctx, "CREATE TABLE sample (id INTEGER PRIMARY KEY, name TEXT); INSERT INTO sample (name) VALUES ('backup');"); err != nil {
		t.Fatalf("seed database: %v", err)
	}

	service := NewBackupService(db, dbPath, backupDir)
	service.now = func() time.Time { return time.Date(2026, 7, 1, 10, 0, 0, 0, time.Local) }

	created, err := service.CreateBackup(ctx)
	if err != nil {
		t.Fatalf("CreateBackup() error = %v", err)
	}
	if created.FileName != "baeum-maru-20260701-100000.db" {
		t.Fatalf("FileName = %q, want timestamped backup", created.FileName)
	}
	if _, err := os.Stat(created.Path); err != nil {
		t.Fatalf("created backup stat error = %v", err)
	}

	files, err := service.ListBackups(ctx)
	if err != nil {
		t.Fatalf("ListBackups() error = %v", err)
	}
	if len(files) != 1 {
		t.Fatalf("len(files) = %d, want 1", len(files))
	}

	plan, err := service.QueueRestore(ctx, created.FileName)
	if err != nil {
		t.Fatalf("QueueRestore() error = %v", err)
	}
	if plan.FileName != created.FileName {
		t.Fatalf("plan.FileName = %q, want created backup", plan.FileName)
	}

	status, err := service.Status(ctx)
	if err != nil {
		t.Fatalf("Status() error = %v", err)
	}
	if status.Latest == nil || status.Latest.FileName != created.FileName {
		t.Fatalf("status.Latest = %#v, want created backup", status.Latest)
	}
	if status.TotalCount != 1 || status.TotalBytes == 0 {
		t.Fatalf("status = %+v, want one non-empty backup", status)
	}
}

func TestBackupServicePrunesOldBackups(t *testing.T) {
	ctx := context.Background()
	root := t.TempDir()
	backupDir := filepath.Join(root, "backups")
	if err := os.MkdirAll(backupDir, 0o755); err != nil {
		t.Fatalf("MkdirAll() error = %v", err)
	}
	oldPath := filepath.Join(backupDir, "old.db")
	newPath := filepath.Join(backupDir, "new.db")
	if err := os.WriteFile(oldPath, []byte("old"), 0o644); err != nil {
		t.Fatalf("WriteFile(old) error = %v", err)
	}
	if err := os.WriteFile(newPath, []byte("new"), 0o644); err != nil {
		t.Fatalf("WriteFile(new) error = %v", err)
	}
	now := time.Date(2026, 7, 1, 10, 0, 0, 0, time.Local)
	if err := os.Chtimes(oldPath, now.AddDate(0, 0, -31), now.AddDate(0, 0, -31)); err != nil {
		t.Fatalf("Chtimes(old) error = %v", err)
	}
	if err := os.Chtimes(newPath, now.AddDate(0, 0, -2), now.AddDate(0, 0, -2)); err != nil {
		t.Fatalf("Chtimes(new) error = %v", err)
	}

	service := NewBackupService(nil, "", backupDir, 30)
	service.now = func() time.Time { return now }
	cleanup, err := service.PruneOldBackups(ctx)
	if err != nil {
		t.Fatalf("PruneOldBackups() error = %v", err)
	}
	if cleanup.DeletedCount != 1 || cleanup.DeletedFiles[0] != "old.db" {
		t.Fatalf("cleanup = %+v, want old.db deleted", cleanup)
	}
	if _, err := os.Stat(oldPath); !os.IsNotExist(err) {
		t.Fatalf("old backup still exists or stat error = %v", err)
	}
	if _, err := os.Stat(newPath); err != nil {
		t.Fatalf("new backup stat error = %v", err)
	}
}
