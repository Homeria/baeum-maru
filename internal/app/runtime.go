package app

import (
	"context"
	"database/sql"
	"fmt"

	"github.com/Homeria/baeum-maru/internal/config"
	"github.com/Homeria/baeum-maru/internal/database"
	"github.com/Homeria/baeum-maru/internal/logging"
	"github.com/Homeria/baeum-maru/internal/migration"
	"github.com/Homeria/baeum-maru/internal/repository"
	"github.com/Homeria/baeum-maru/internal/service"
)

type Runtime struct {
	Config  config.Config
	Logger  *logging.Logger
	DB      *sql.DB
	Members *service.MemberService
	Courses *service.CourseService
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

	memberRepository := repository.NewMemberRepository(db)
	courseRepository := repository.NewCourseRepository(db)

	return &Runtime{
		Config:  cfg,
		Logger:  logger,
		DB:      db,
		Members: service.NewMemberService(memberRepository),
		Courses: service.NewCourseService(courseRepository),
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
