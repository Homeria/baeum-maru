package logging

import (
	"bytes"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestNewFileLoggerWritesLogFile(t *testing.T) {
	path := filepath.Join(t.TempDir(), "logs", "app.log")

	logger, err := NewFileLogger(path, "info")
	if err != nil {
		t.Fatalf("NewFileLogger() error = %v", err)
	}
	logger.Info("hello", "component", "test")
	if err := logger.Close(); err != nil {
		t.Fatalf("Close() error = %v", err)
	}

	data, err := os.ReadFile(path)
	if err != nil {
		t.Fatalf("ReadFile() error = %v", err)
	}
	if !strings.Contains(string(data), "hello") {
		t.Fatalf("log file = %q, want message", string(data))
	}
}

func TestNewWriterLoggerHonorsLevel(t *testing.T) {
	var buf bytes.Buffer
	logger := NewWriterLogger(&buf, "warn")

	logger.Info("hidden")
	logger.Warn("visible")

	output := buf.String()
	if strings.Contains(output, "hidden") {
		t.Fatalf("output = %q, did not expect info log", output)
	}
	if !strings.Contains(output, "visible") {
		t.Fatalf("output = %q, want warn log", output)
	}
}
