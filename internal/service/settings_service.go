package service

import (
	"context"
	"errors"
	"fmt"
	"strings"

	"github.com/Homeria/baeum-maru/internal/config"
)

type SettingsService struct {
	path string
	cfg  config.Config
}

type SettingsInput struct {
	ServerHost         string
	ServerPort         int
	BackupKeepDays     int
	OpenBrowserOnStart bool
}

func NewSettingsService(path string, cfg config.Config) *SettingsService {
	if path == "" {
		path = config.DefaultPath
	}
	return &SettingsService{path: path, cfg: cfg}
}

func (s *SettingsService) Get(context.Context) (config.Config, error) {
	if s == nil {
		return config.Config{}, errors.New("settings service is not configured")
	}
	return s.cfg, nil
}

func (s *SettingsService) Update(ctx context.Context, input SettingsInput) (config.Config, error) {
	if s == nil {
		return config.Config{}, errors.New("settings service is not configured")
	}
	if input.BackupKeepDays < 0 {
		return config.Config{}, errors.New("backup keep days must be zero or greater")
	}
	updated := s.cfg
	if strings.TrimSpace(input.ServerHost) != "" {
		updated.Server.Host = strings.TrimSpace(input.ServerHost)
	}
	if input.ServerPort != 0 {
		if input.ServerPort < 1 || input.ServerPort > 65535 {
			return config.Config{}, fmt.Errorf("server port must be between 1 and 65535: %d", input.ServerPort)
		}
		updated.Server.Port = input.ServerPort
	}
	updated.Backup.KeepDays = input.BackupKeepDays
	updated.UI.OpenBrowserOnStart = input.OpenBrowserOnStart
	if err := config.Save(s.path, updated); err != nil {
		return config.Config{}, err
	}
	s.cfg = updated
	return s.cfg, nil
}
