package app

import (
	"fmt"

	"github.com/Homeria/baeum-maru/internal/config"
	"github.com/Homeria/baeum-maru/internal/logging"
)

type Runtime struct {
	Config config.Config
	Logger *logging.Logger
}

func Bootstrap(configPath string) (*Runtime, error) {
	cfg, err := config.LoadOrCreate(configPath)
	if err != nil {
		return nil, fmt.Errorf("load config: %w", err)
	}
	if err := config.EnsureRuntimeDirs(cfg); err != nil {
		return nil, fmt.Errorf("ensure runtime directories: %w", err)
	}

	logger, err := logging.NewFileLogger(cfg.Logging.Path, cfg.Logging.Level)
	if err != nil {
		return nil, fmt.Errorf("initialize logger: %w", err)
	}

	return &Runtime{
		Config: cfg,
		Logger: logger,
	}, nil
}

func (r *Runtime) Close() error {
	if r == nil || r.Logger == nil {
		return nil
	}
	return r.Logger.Close()
}
