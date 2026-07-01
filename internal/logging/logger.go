// Package logging initializes structured file logging.
package logging

import (
	"fmt"
	"io"
	"log/slog"
	"os"
	"path/filepath"
	"strings"
)

type Logger struct {
	*slog.Logger
	close func() error
}

func NewFileLogger(path string, level string) (*Logger, error) {
	if path == "" {
		return nil, fmt.Errorf("log path is required")
	}
	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil && filepath.Dir(path) != "." {
		return nil, fmt.Errorf("create log directory: %w", err)
	}

	file, err := os.OpenFile(path, os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0o644)
	if err != nil {
		return nil, fmt.Errorf("open log file: %w", err)
	}

	handler := slog.NewTextHandler(file, &slog.HandlerOptions{
		Level: parseLevel(level),
	})
	return &Logger{
		Logger: slog.New(handler),
		close:  file.Close,
	}, nil
}

func NewWriterLogger(w io.Writer, level string) *slog.Logger {
	return slog.New(slog.NewTextHandler(w, &slog.HandlerOptions{
		Level: parseLevel(level),
	}))
}

func (l *Logger) Close() error {
	if l == nil || l.close == nil {
		return nil
	}
	return l.close()
}

func parseLevel(level string) slog.Level {
	switch strings.ToLower(strings.TrimSpace(level)) {
	case "debug":
		return slog.LevelDebug
	case "warn", "warning":
		return slog.LevelWarn
	case "error":
		return slog.LevelError
	default:
		return slog.LevelInfo
	}
}
