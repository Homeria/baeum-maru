package config

import (
	"errors"
	"fmt"
)

func Validate(cfg Config) error {
	if cfg.App.DisplayName == "" {
		return errors.New("config app.display_name is required")
	}
	if cfg.App.EnglishName == "" {
		return errors.New("config app.english_name is required")
	}
	if cfg.App.Mode == "" {
		return errors.New("config app.mode is required")
	}
	if cfg.Server.Host == "" {
		return errors.New("config server.host is required")
	}
	if cfg.Server.Port < 1 || cfg.Server.Port > 65535 {
		return fmt.Errorf("config server.port must be between 1 and 65535: %d", cfg.Server.Port)
	}
	if cfg.Database.Path == "" {
		return errors.New("config database.path is required")
	}
	if cfg.Backup.Path == "" {
		return errors.New("config backup.path is required")
	}
	if cfg.Backup.KeepDays < 0 {
		return fmt.Errorf("config backup.keep_days must be zero or greater: %d", cfg.Backup.KeepDays)
	}
	if cfg.Logging.Path == "" {
		return errors.New("config logging.path is required")
	}
	if cfg.Logging.Level == "" {
		return errors.New("config logging.level is required")
	}
	return nil
}
