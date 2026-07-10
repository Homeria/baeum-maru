package service

import (
	"context"
	"database/sql"
	"os"
	"path/filepath"
	"testing"

	backupstore "github.com/Homeria/baeum-maru/internal/backup"
	"github.com/Homeria/baeum-maru/internal/database"
	"github.com/Homeria/baeum-maru/internal/migration"
	"github.com/Homeria/baeum-maru/internal/repository"
)

func TestPrototypeBaselineRestoresWorkflowDataAcrossRestart(t *testing.T) {
	ctx := context.Background()
	root := t.TempDir()
	databasePath := filepath.Join(root, "data", "center.db")
	backupDir := filepath.Join(root, "backups")

	db := openPrototypeBaselineDB(t, ctx, databasePath)
	members := NewMemberService(repository.NewMemberRepository(db))
	courses := NewCourseService(repository.NewCourseRepository(db))
	registrations := NewRegistrationService(
		repository.NewRegistrationRepository(db),
		repository.NewMemberRepository(db),
		repository.NewCourseRepository(db),
	)

	offering := createWorkflowOffering(t, ctx, courses, CourseOfferingInput{
		TermName:     "2026 baseline term",
		CategoryName: "education",
		CourseTitle:  "computer basics",
		Capacity:     20,
		Weekday:      1,
		StartTime:    "09:00",
		EndTime:      "09:50",
	})
	baselineMember := createWorkflowMember(t, ctx, members, "M-BASE", "baseline member")
	baselineRegistration := createWorkflowRegistration(t, ctx, registrations, baselineMember.ID, offering.ID)

	backups := NewBackupService(db, databasePath, backupDir, 30)
	created, err := backups.CreateBackup(ctx)
	if err != nil {
		t.Fatalf("CreateBackup() error = %v", err)
	}
	if _, err := backups.QueueRestore(ctx, created.FileName); err != nil {
		t.Fatalf("QueueRestore() error = %v", err)
	}
	createWorkflowMember(t, ctx, members, "M-AFTER", "post-backup member")

	if err := db.Close(); err != nil {
		t.Fatalf("close database before restore: %v", err)
	}
	if err := backupstore.ApplyPendingRestore(databasePath, backupDir); err != nil {
		t.Fatalf("ApplyPendingRestore() error = %v", err)
	}

	restoredDB := openPrototypeBaselineDB(t, ctx, databasePath)
	t.Cleanup(func() {
		if err := restoredDB.Close(); err != nil {
			t.Errorf("close restored database: %v", err)
		}
	})
	restoredMembers := repository.NewMemberRepository(restoredDB)
	restoredRegistrations := repository.NewRegistrationRepository(restoredDB)

	baselineMatches, err := restoredMembers.Search(ctx, "M-BASE", 10)
	if err != nil {
		t.Fatalf("search restored baseline member: %v", err)
	}
	if len(baselineMatches) != 1 || baselineMatches[0].ID != baselineMember.ID {
		t.Fatalf("restored baseline members = %+v, want member %d", baselineMatches, baselineMember.ID)
	}

	postBackupMatches, err := restoredMembers.Search(ctx, "M-AFTER", 10)
	if err != nil {
		t.Fatalf("search post-backup member: %v", err)
	}
	if len(postBackupMatches) != 0 {
		t.Fatalf("post-backup members = %+v, want none after restore", postBackupMatches)
	}

	restoredRegistration, err := restoredRegistrations.Get(ctx, baselineRegistration.ID)
	if err != nil {
		t.Fatalf("get restored registration: %v", err)
	}
	if restoredRegistration.MemberID != baselineMember.ID || restoredRegistration.OfferingID != offering.ID {
		t.Fatalf("restored registration = %+v, want member %d and offering %d", restoredRegistration, baselineMember.ID, offering.ID)
	}
}

func openPrototypeBaselineDB(t *testing.T, ctx context.Context, path string) *sql.DB {
	t.Helper()

	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		t.Fatalf("create database directory: %v", err)
	}
	db, err := database.Open(ctx, database.Options{Path: path})
	if err != nil {
		t.Fatalf("database.Open() error = %v", err)
	}
	if err := migration.Run(ctx, db, nil); err != nil {
		_ = db.Close()
		t.Fatalf("migration.Run() error = %v", err)
	}
	return db
}
