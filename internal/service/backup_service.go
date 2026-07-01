package service

import (
	"context"
	"database/sql"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"

	"github.com/Homeria/baeum-maru/internal/backup"
	"github.com/Homeria/baeum-maru/internal/domain"
)

type BackupService struct {
	db           *sql.DB
	databasePath string
	dir          string
	keepDays     int
	now          func() time.Time
}

func NewBackupService(db *sql.DB, databasePath string, dir string, keepDays ...int) *BackupService {
	retentionDays := 0
	if len(keepDays) > 0 {
		retentionDays = keepDays[0]
	}
	return &BackupService{
		db:           db,
		databasePath: databasePath,
		dir:          dir,
		keepDays:     retentionDays,
		now:          time.Now,
	}
}

func (s *BackupService) CreateBackup(ctx context.Context) (domain.BackupFile, error) {
	if err := os.MkdirAll(s.dir, 0o755); err != nil {
		return domain.BackupFile{}, fmt.Errorf("create backup directory: %w", err)
	}
	fileName := "baeum-maru-" + s.now().Format("20060102-150405") + ".db"
	path := filepath.Join(s.dir, fileName)
	if err := s.vacuumInto(ctx, path); err != nil {
		return domain.BackupFile{}, err
	}
	return backupFileFromPath(path)
}

func (s *BackupService) Status(ctx context.Context) (domain.BackupStatus, error) {
	files, err := s.ListBackups(ctx)
	if err != nil {
		return domain.BackupStatus{}, err
	}
	status := domain.BackupStatus{
		TotalCount:  len(files),
		KeepDays:    s.keepDays,
		RetentionOn: s.keepDays > 0,
	}
	if len(files) > 0 {
		latest := files[0]
		status.Latest = &latest
	}
	for _, file := range files {
		status.TotalBytes += file.SizeBytes
	}
	return status, nil
}

func (s *BackupService) PruneOldBackups(context.Context) (domain.BackupCleanup, error) {
	if s.keepDays <= 0 {
		return domain.BackupCleanup{}, nil
	}
	entries, err := os.ReadDir(s.dir)
	if err != nil {
		if os.IsNotExist(err) {
			return domain.BackupCleanup{}, nil
		}
		return domain.BackupCleanup{}, fmt.Errorf("read backup directory for cleanup: %w", err)
	}

	cutoff := s.now().AddDate(0, 0, -s.keepDays)
	cleanup := domain.BackupCleanup{}
	for _, entry := range entries {
		if entry.IsDir() || strings.ToLower(filepath.Ext(entry.Name())) != ".db" {
			continue
		}
		path := filepath.Join(s.dir, entry.Name())
		info, err := os.Stat(path)
		if err != nil {
			return domain.BackupCleanup{}, fmt.Errorf("stat backup file for cleanup: %w", err)
		}
		if !info.ModTime().Before(cutoff) {
			continue
		}
		if err := os.Remove(path); err != nil {
			return domain.BackupCleanup{}, fmt.Errorf("delete old backup %s: %w", entry.Name(), err)
		}
		cleanup.DeletedCount++
		cleanup.DeletedFiles = append(cleanup.DeletedFiles, entry.Name())
	}
	return cleanup, nil
}

func (s *BackupService) ListBackups(context.Context) ([]domain.BackupFile, error) {
	entries, err := os.ReadDir(s.dir)
	if err != nil {
		if os.IsNotExist(err) {
			return nil, nil
		}
		return nil, fmt.Errorf("read backup directory: %w", err)
	}

	files := make([]domain.BackupFile, 0, len(entries))
	for _, entry := range entries {
		if entry.IsDir() || strings.ToLower(filepath.Ext(entry.Name())) != ".db" {
			continue
		}
		file, err := backupFileFromPath(filepath.Join(s.dir, entry.Name()))
		if err != nil {
			return nil, err
		}
		files = append(files, file)
	}
	sort.Slice(files, func(i, j int) bool {
		return files[i].CreatedAt > files[j].CreatedAt
	})
	return files, nil
}

func (s *BackupService) ResolveBackupPath(fileName string) (string, error) {
	path, err := backup.ResolveBackupPath(s.dir, fileName)
	if err != nil {
		return "", err
	}
	if _, err := os.Stat(path); err != nil {
		return "", fmt.Errorf("stat backup file: %w", err)
	}
	return path, nil
}

func (s *BackupService) QueueRestore(_ context.Context, fileName string) (domain.RestorePlan, error) {
	path, err := backup.ResolveBackupPath(s.dir, fileName)
	if err != nil {
		return domain.RestorePlan{}, err
	}
	if err := backup.QueueRestore(s.dir, fileName, s.now()); err != nil {
		return domain.RestorePlan{}, err
	}
	return domain.RestorePlan{FileName: filepath.Base(fileName), Path: path}, nil
}

func (s *BackupService) vacuumInto(ctx context.Context, path string) error {
	if s.db == nil {
		return fmt.Errorf("database is not configured")
	}
	quotedPath := strings.ReplaceAll(path, "'", "''")
	if _, err := s.db.ExecContext(ctx, "VACUUM INTO '"+quotedPath+"';"); err != nil {
		return fmt.Errorf("create sqlite backup: %w", err)
	}
	return nil
}

func backupFileFromPath(path string) (domain.BackupFile, error) {
	info, err := os.Stat(path)
	if err != nil {
		return domain.BackupFile{}, fmt.Errorf("stat backup file: %w", err)
	}
	return domain.BackupFile{
		FileName:  filepath.Base(path),
		Path:      path,
		SizeBytes: info.Size(),
		CreatedAt: info.ModTime().Format(time.RFC3339),
	}, nil
}
