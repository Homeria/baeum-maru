package config

import (
	"os"
	"path/filepath"
	"testing"
)

func TestLoadOrCreateCreatesDefaultConfig(t *testing.T) {
	path := filepath.Join(t.TempDir(), "config.json")

	cfg, err := LoadOrCreate(path)
	if err != nil {
		t.Fatalf("LoadOrCreate() error = %v", err)
	}
	if cfg.Server.Port != 18080 {
		t.Fatalf("Server.Port = %d, want 18080", cfg.Server.Port)
	}
	if _, err := os.Stat(path); err != nil {
		t.Fatalf("created config stat error = %v", err)
	}
}

func TestLoadReadsSavedConfig(t *testing.T) {
	path := filepath.Join(t.TempDir(), "config.json")
	cfg := Default()
	cfg.Server.Port = 18081

	if err := Save(path, cfg); err != nil {
		t.Fatalf("Save() error = %v", err)
	}

	loaded, err := Load(path)
	if err != nil {
		t.Fatalf("Load() error = %v", err)
	}
	if loaded.Server.Port != 18081 {
		t.Fatalf("Server.Port = %d, want 18081", loaded.Server.Port)
	}
}

func TestLoadFillsMissingExportConfig(t *testing.T) {
	path := filepath.Join(t.TempDir(), "config.json")
	data := []byte(`{
  "app": {"display_name": "배움마루", "english_name": "Baeum-Maru", "mode": "portable"},
  "server": {"host": "0.0.0.0", "port": 18080},
  "database": {"path": "./data/center.db"},
  "backup": {"path": "./backups", "keep_days": 30},
  "logging": {"path": "./logs/app.log", "level": "info"},
  "ui": {"open_browser_on_start": true}
}
`)
	if err := os.WriteFile(path, data, 0o644); err != nil {
		t.Fatalf("WriteFile() error = %v", err)
	}

	cfg, err := Load(path)
	if err != nil {
		t.Fatalf("Load() error = %v", err)
	}
	if cfg.Export.Path != "./exports" {
		t.Fatalf("Export.Path = %q, want ./exports", cfg.Export.Path)
	}
}

func TestValidateRejectsInvalidPort(t *testing.T) {
	cfg := Default()
	cfg.Server.Port = 70000

	if err := Validate(cfg); err == nil {
		t.Fatal("Validate() error = nil, want invalid port error")
	}
}

func TestEnsureRuntimeDirsCreatesDirectories(t *testing.T) {
	root := t.TempDir()
	previousWD, err := os.Getwd()
	if err != nil {
		t.Fatalf("Getwd() error = %v", err)
	}
	if err := os.Chdir(root); err != nil {
		t.Fatalf("Chdir() error = %v", err)
	}
	t.Cleanup(func() {
		if err := os.Chdir(previousWD); err != nil {
			t.Fatalf("restore working directory: %v", err)
		}
	})

	cfg := Default()
	cfg.Database.Path = filepath.Join(root, "data", "center.db")
	cfg.Backup.Path = filepath.Join(root, "backups")
	cfg.Export.Path = filepath.Join(root, "exports")
	cfg.Logging.Path = filepath.Join(root, "logs", "app.log")

	if err := EnsureRuntimeDirs(cfg); err != nil {
		t.Fatalf("EnsureRuntimeDirs() error = %v", err)
	}

	for _, dir := range []string{"data", "backups", "exports", "imports", "logs"} {
		if _, err := os.Stat(filepath.Join(root, dir)); err != nil {
			t.Fatalf("runtime dir %q stat error = %v", dir, err)
		}
	}
}
