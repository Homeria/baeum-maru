package backup

import (
	"os"
	"path/filepath"
	"testing"
	"time"
)

func TestQueueAndApplyPendingRestore(t *testing.T) {
	root := t.TempDir()
	backupDir := filepath.Join(root, "backups")
	databasePath := filepath.Join(root, "data", "center.db")
	if err := os.MkdirAll(filepath.Dir(databasePath), 0o755); err != nil {
		t.Fatalf("MkdirAll() error = %v", err)
	}
	if err := os.MkdirAll(backupDir, 0o755); err != nil {
		t.Fatalf("MkdirAll() error = %v", err)
	}
	if err := os.WriteFile(databasePath, []byte("current"), 0o644); err != nil {
		t.Fatalf("WriteFile(current) error = %v", err)
	}
	if err := os.WriteFile(filepath.Join(backupDir, "backup.db"), []byte("restored"), 0o644); err != nil {
		t.Fatalf("WriteFile(backup) error = %v", err)
	}

	if err := QueueRestore(backupDir, "backup.db", time.Date(2026, 7, 1, 10, 0, 0, 0, time.UTC)); err != nil {
		t.Fatalf("QueueRestore() error = %v", err)
	}
	if _, err := os.Stat(RestoreRequestPath(backupDir)); err != nil {
		t.Fatalf("restore request stat error = %v", err)
	}

	if err := ApplyPendingRestore(databasePath, backupDir); err != nil {
		t.Fatalf("ApplyPendingRestore() error = %v", err)
	}
	data, err := os.ReadFile(databasePath)
	if err != nil {
		t.Fatalf("ReadFile(database) error = %v", err)
	}
	if string(data) != "restored" {
		t.Fatalf("database contents = %q, want restored", string(data))
	}
	if _, err := os.Stat(RestoreRequestPath(backupDir)); !os.IsNotExist(err) {
		t.Fatalf("restore request still exists, err = %v", err)
	}
}
