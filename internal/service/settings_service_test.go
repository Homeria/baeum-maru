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
		ServerHost:         "127.0.0.1",
		ServerPort:         19090,
		BackupKeepDays:     14,
		OpenBrowserOnStart: false,
	})
	if err != nil {
		t.Fatalf("Update() error = %v", err)
	}
	if updated.Backup.KeepDays != 14 {
		t.Fatalf("Backup.KeepDays = %d, want 14", updated.Backup.KeepDays)
	}
	if updated.Server.Host != "127.0.0.1" || updated.Server.Port != 19090 {
		t.Fatalf("Server = %+v, want 127.0.0.1:19090", updated.Server)
	}
	if updated.UI.OpenBrowserOnStart {
		t.Fatal("OpenBrowserOnStart = true, want false")
	}

	loaded, err := config.Load(path)
	if err != nil {
		t.Fatalf("config.Load() error = %v", err)
	}
	if loaded.Server.Host != "127.0.0.1" || loaded.Server.Port != 19090 || loaded.Backup.KeepDays != 14 || loaded.UI.OpenBrowserOnStart {
		t.Fatalf("loaded = %+v, want persisted settings", loaded)
	}
}

func TestSettingsServiceRejectsNegativeKeepDays(t *testing.T) {
	service := NewSettingsService(filepath.Join(t.TempDir(), "config.json"), config.Default())
	if _, err := service.Update(context.Background(), SettingsInput{BackupKeepDays: -1}); err == nil {
		t.Fatal("Update() error = nil, want validation error")
	}
}

func TestSettingsServiceRejectsInvalidServerPort(t *testing.T) {
	service := NewSettingsService(filepath.Join(t.TempDir(), "config.json"), config.Default())
	if _, err := service.Update(context.Background(), SettingsInput{ServerPort: 70000}); err == nil {
		t.Fatal("Update() error = nil, want validation error")
	}
}
