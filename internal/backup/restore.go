package backup

import (
	"encoding/json"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"
	"time"
)

const RestoreRequestFileName = "restore-request.json"

type RestoreRequest struct {
	FileName string `json:"file_name"`
	QueuedAt string `json:"queued_at"`
}

func QueueRestore(backupDir string, fileName string, now time.Time) error {
	path, err := ResolveBackupPath(backupDir, fileName)
	if err != nil {
		return err
	}
	if _, err := os.Stat(path); err != nil {
		return fmt.Errorf("stat restore backup: %w", err)
	}

	request := RestoreRequest{
		FileName: filepath.Base(fileName),
		QueuedAt: now.Format(time.RFC3339),
	}
	data, err := json.MarshalIndent(request, "", "  ")
	if err != nil {
		return fmt.Errorf("marshal restore request: %w", err)
	}
	data = append(data, '\n')

	if err := os.MkdirAll(backupDir, 0o755); err != nil {
		return fmt.Errorf("create backup directory: %w", err)
	}
	if err := os.WriteFile(RestoreRequestPath(backupDir), data, 0o644); err != nil {
		return fmt.Errorf("write restore request: %w", err)
	}
	return nil
}

func ApplyPendingRestore(databasePath string, backupDir string) error {
	requestPath := RestoreRequestPath(backupDir)
	data, err := os.ReadFile(requestPath)
	if err != nil {
		if os.IsNotExist(err) {
			return nil
		}
		return fmt.Errorf("read restore request: %w", err)
	}

	var request RestoreRequest
	if err := json.Unmarshal(data, &request); err != nil {
		return fmt.Errorf("parse restore request: %w", err)
	}
	sourcePath, err := ResolveBackupPath(backupDir, request.FileName)
	if err != nil {
		return err
	}
	if _, err := os.Stat(sourcePath); err != nil {
		return fmt.Errorf("stat restore source: %w", err)
	}

	if err := os.MkdirAll(filepath.Dir(databasePath), 0o755); err != nil && filepath.Dir(databasePath) != "." {
		return fmt.Errorf("create database directory: %w", err)
	}
	if _, err := os.Stat(databasePath); err == nil {
		preRestoreName := "pre-restore-" + time.Now().Format("20060102-150405") + ".db"
		if err := copyFile(databasePath, filepath.Join(backupDir, preRestoreName)); err != nil {
			return fmt.Errorf("create pre-restore backup: %w", err)
		}
	} else if !os.IsNotExist(err) {
		return fmt.Errorf("stat current database: %w", err)
	}

	if err := copyFile(sourcePath, databasePath); err != nil {
		return fmt.Errorf("restore database: %w", err)
	}
	if err := os.Remove(requestPath); err != nil {
		return fmt.Errorf("remove restore request: %w", err)
	}
	return nil
}

func ResolveBackupPath(backupDir string, fileName string) (string, error) {
	if strings.TrimSpace(fileName) == "" {
		return "", fmt.Errorf("backup file name is required")
	}
	if filepath.Base(fileName) != fileName {
		return "", fmt.Errorf("backup file name must not include directories")
	}
	if filepath.Ext(fileName) != ".db" {
		return "", fmt.Errorf("backup file must be a .db file")
	}
	return filepath.Join(backupDir, fileName), nil
}

func RestoreRequestPath(backupDir string) string {
	return filepath.Join(backupDir, RestoreRequestFileName)
}

func copyFile(sourcePath string, targetPath string) error {
	source, err := os.Open(sourcePath)
	if err != nil {
		return err
	}
	defer source.Close()

	if err := os.MkdirAll(filepath.Dir(targetPath), 0o755); err != nil && filepath.Dir(targetPath) != "." {
		return err
	}
	target, err := os.Create(targetPath)
	if err != nil {
		return err
	}
	defer target.Close()

	if _, err := io.Copy(target, source); err != nil {
		return err
	}
	return target.Sync()
}
