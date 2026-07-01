package app

import (
	"context"
	"database/sql"
	"fmt"

	"github.com/Homeria/baeum-maru/internal/config"
	"github.com/Homeria/baeum-maru/internal/database"
	"github.com/Homeria/baeum-maru/internal/logging"
	"github.com/Homeria/baeum-maru/internal/migration"
)

type Runtime struct {
	Config config.Config
	Logger *logging.Logger
	DB     *sql.DB
}

func Bootstrap(configPath string) (*Runtime, error) {
	ctx := context.Background()

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

	db, err := database.Open(ctx, database.Options{
		Path: cfg.Database.Path,
	})
	if err != nil {
		_ = logger.Close()
		return nil, fmt.Errorf("open database: %w", err)
	}
	if err := migration.Run(ctx, db, nil); err != nil {
		_ = db.Close()
		_ = logger.Close()
		return nil, fmt.Errorf("run migrations: %w", err)
	}

	return &Runtime{
		Config: cfg,
		Logger: logger,
		DB:     db,
	}, nil
}

func (r *Runtime) Close() error {
	if r == nil {
		return nil
	}
	var closeErr error
	if r.DB != nil {
		if err := r.DB.Close(); err != nil {
			closeErr = err
		}
	}
	if r.Logger != nil {
		if err := r.Logger.Close(); err != nil && closeErr == nil {
			closeErr = err
		}
	}
	return closeErr
}
