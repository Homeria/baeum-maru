package config

import (
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"path/filepath"
)

const DefaultPath = "config.json"

func Load(path string) (Config, error) {
	if path == "" {
		path = DefaultPath
	}

	data, err := os.ReadFile(path)
	if err != nil {
		return Config{}, fmt.Errorf("read config: %w", err)
	}

	var cfg Config
	if err := json.Unmarshal(data, &cfg); err != nil {
		return Config{}, fmt.Errorf("parse config: %w", err)
	}
	applyMissingDefaults(&cfg)
	if err := Validate(cfg); err != nil {
		return Config{}, err
	}
	return cfg, nil
}

func LoadOrCreate(path string) (Config, error) {
	if path == "" {
		path = DefaultPath
	}

	cfg, err := Load(path)
	if err == nil {
		return cfg, nil
	}
	if !errors.Is(err, os.ErrNotExist) {
		return Config{}, err
	}

	cfg = Default()
	if err := Save(path, cfg); err != nil {
		return Config{}, err
	}
	return cfg, nil
}

func Save(path string, cfg Config) error {
	if path == "" {
		path = DefaultPath
	}
	if err := Validate(cfg); err != nil {
		return err
	}

	data, err := json.MarshalIndent(cfg, "", "  ")
	if err != nil {
		return fmt.Errorf("marshal config: %w", err)
	}
	data = append(data, '\n')

	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil && filepath.Dir(path) != "." {
		return fmt.Errorf("create config directory: %w", err)
	}
	if err := os.WriteFile(path, data, 0o644); err != nil {
		return fmt.Errorf("write config: %w", err)
	}
	return nil
}

func EnsureRuntimeDirs(cfg Config) error {
	dirs := []string{
		filepath.Dir(cfg.Database.Path),
		cfg.Backup.Path,
		cfg.Export.Path,
		"./imports",
		filepath.Dir(cfg.Logging.Path),
	}

	for _, dir := range dirs {
		if dir == "." || dir == "" {
			continue
		}
		if err := os.MkdirAll(dir, 0o755); err != nil {
			return fmt.Errorf("create runtime directory %q: %w", dir, err)
		}
	}
	return nil
}

func applyMissingDefaults(cfg *Config) {
	defaults := Default()
	if cfg.Export.Path == "" {
		cfg.Export.Path = defaults.Export.Path
	}
	if cfg.Auth.AdminPassword == "" {
		cfg.Auth.AdminPassword = defaults.Auth.AdminPassword
	}
	if cfg.Auth.SessionSecret == "" {
		cfg.Auth.SessionSecret = defaults.Auth.SessionSecret
	}
	if cfg.Auth.SessionMaxAgeMinutes == 0 {
		cfg.Auth.SessionMaxAgeMinutes = defaults.Auth.SessionMaxAgeMinutes
	}
}
