package service

import (
	"context"
	"path/filepath"
	"testing"

	"github.com/Homeria/baeum-maru/internal/config"
)

func TestSettingsServiceUpdatesConfig(t *testing.T) {
	path := filepath.Join(t.TempDir(), "config.json")
	cfg := config.Default()
	service := NewSettingsService(path, cfg)

	updated, err := service.Update(context.Background(), SettingsInput{
		BackupKeepDays:     14,
		OpenBrowserOnStart: false,
	})
	if err != nil {
		t.Fatalf("Update() error = %v", err)
	}
	if updated.Backup.KeepDays != 14 {
		t.Fatalf("Backup.KeepDays = %d, want 14", updated.Backup.KeepDays)
	}
	if updated.UI.OpenBrowserOnStart {
		t.Fatal("OpenBrowserOnStart = true, want false")
	}

	loaded, err := config.Load(path)
	if err != nil {
		t.Fatalf("config.Load() error = %v", err)
	}
	if loaded.Backup.KeepDays != 14 || loaded.UI.OpenBrowserOnStart {
		t.Fatalf("loaded = %+v, want persisted settings", loaded)
	}
}

func TestSettingsServiceRejectsNegativeKeepDays(t *testing.T) {
	service := NewSettingsService(filepath.Join(t.TempDir(), "config.json"), config.Default())
	if _, err := service.Update(context.Background(), SettingsInput{BackupKeepDays: -1}); err == nil {
		t.Fatal("Update() error = nil, want validation error")
	}
}
