package service

import (
	"context"
	"database/sql"
	"errors"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/Homeria/baeum-maru/internal/database"
	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/migration"
	"github.com/Homeria/baeum-maru/internal/repository"
)

func TestMVPWorkflowSimulation(t *testing.T) {
	ctx := context.Background()
	db, dbPath := newWorkflowTestDB(t)

	memberRepo := repository.NewMemberRepository(db)
	courseRepo := repository.NewCourseRepository(db)
	registrationRepo := repository.NewRegistrationRepository(db)
	lotteryRepo := repository.NewLotteryRepository(db)
	attendanceRepo := repository.NewAttendanceRepository(db)
	auditRepo := repository.NewAuditRepository(db)

	members := NewMemberService(memberRepo)
	courses := NewCourseService(courseRepo)
	registrations := NewRegistrationService(registrationRepo, memberRepo, courseRepo)
	lotteries := NewLotteryService(lotteryRepo, courseRepo)
	attendance := NewAttendanceService(attendanceRepo)
	audits := NewAuditService(auditRepo)
	exports := NewExportService(members, courses, registrations, filepath.Join(t.TempDir(), "exports"), lotteries, attendance)
	backups := NewBackupService(db, dbPath, filepath.Join(t.TempDir(), "backups"), 30)

	popular := createWorkflowOffering(t, ctx, courses, CourseOfferingInput{
		TermName:     "2026 여름학기",
		CategoryName: "건강",
		CourseTitle:  "요가 기초",
		Capacity:     1,
		Weekday:      1,
		StartTime:    "09:00",
		EndTime:      "10:00",
	})
	conflicting := createWorkflowOffering(t, ctx, courses, CourseOfferingInput{
		TermName:     "2026 여름학기",
		CategoryName: "건강",
		CourseTitle:  "라인댄스",
		Capacity:     10,
		Weekday:      1,
		StartTime:    "09:30",
		EndTime:      "10:30",
	})
	second := createWorkflowOffering(t, ctx, courses, CourseOfferingInput{
		TermName:     "2026 여름학기",
		CategoryName: "교양",
		CourseTitle:  "스마트폰 활용",
		Capacity:     10,
		Weekday:      2,
		StartTime:    "11:00",
		EndTime:      "12:00",
	})
	third := createWorkflowOffering(t, ctx, courses, CourseOfferingInput{
		TermName:     "2026 여름학기",
		CategoryName: "교양",
		CourseTitle:  "생활 영어",
		Capacity:     10,
		Weekday:      3,
		StartTime:    "13:00",
		EndTime:      "14:00",
	})
	setWorkflowTermLimit(t, ctx, db, "2026 여름학기", 2)

	memberA := createWorkflowMember(t, ctx, members, "M-001", "김배움")
	memberB := createWorkflowMember(t, ctx, members, "M-002", "이마루")
	memberC := createWorkflowMember(t, ctx, members, "M-003", "박학습")

	firstRegistration := createWorkflowRegistration(t, ctx, registrations, memberA.ID, popular.ID)
	if _, err := registrations.Create(ctx, RegistrationInput{MemberID: memberA.ID, OfferingID: popular.ID}); err == nil {
		t.Fatal("duplicate registration error = nil, want rule violation")
	} else {
		assertWorkflowErrorContains(t, err, "이미 신청한 강좌")
	}
	if _, err := registrations.Create(ctx, RegistrationInput{MemberID: memberA.ID, OfferingID: conflicting.ID}); err == nil {
		t.Fatal("time conflict error = nil, want rule violation")
	} else {
		assertWorkflowErrorContains(t, err, "같은 시간대")
	}
	createWorkflowRegistration(t, ctx, registrations, memberA.ID, second.ID)
	if _, err := registrations.Create(ctx, RegistrationInput{MemberID: memberA.ID, OfferingID: third.ID}); err == nil {
		t.Fatal("max registrations error = nil, want rule violation")
	} else {
		assertWorkflowErrorContains(t, err, "1인 최대 신청 강좌 수")
	}

	createWorkflowRegistration(t, ctx, registrations, memberB.ID, popular.ID)
	createWorkflowRegistration(t, ctx, registrations, memberC.ID, popular.ID)

	summary, err := lotteries.RunOfferingLottery(ctx, popular.ID)
	if err != nil {
		t.Fatalf("RunOfferingLottery() error = %v", err)
	}
	if summary.TotalCount != 3 || summary.SelectedCount != 1 || summary.WaitlistedCount != 2 {
		t.Fatalf("lottery summary = %+v, want total 3 selected 1 waitlisted 2", summary)
	}
	if _, err := lotteries.RunOfferingLottery(ctx, popular.ID); err == nil {
		t.Fatal("rerun guard error = nil, want LotteryRerunRequiredError")
	} else {
		var rerunErr *LotteryRerunRequiredError
		if !errors.As(err, &rerunErr) {
			t.Fatalf("rerun error = %T %v, want LotteryRerunRequiredError", err, err)
		}
	}

	popularRegistrations, err := registrations.ListByOffering(ctx, popular.ID)
	if err != nil {
		t.Fatalf("ListByOffering() error = %v", err)
	}
	selected := findWorkflowRegistrationByStatus(t, popularRegistrations, "selected")
	findWorkflowRegistrationByStatus(t, popularRegistrations, "waitlisted")

	confirmed, err := registrations.Confirm(ctx, selected.ID)
	if err != nil {
		t.Fatalf("Confirm() error = %v", err)
	}
	if confirmed.Status != "confirmed" {
		t.Fatalf("confirmed.Status = %q, want confirmed", confirmed.Status)
	}

	session, err := attendance.CreateSession(ctx, AttendanceSessionInput{
		OfferingID:  popular.ID,
		SessionDate: "2026-07-01",
		Note:        "1회차",
	})
	if err != nil {
		t.Fatalf("CreateSession() error = %v", err)
	}
	records, err := attendance.ListRecordsBySession(ctx, session.ID)
	if err != nil {
		t.Fatalf("ListRecordsBySession() error = %v", err)
	}
	if len(records) != 1 || records[0].RegistrationID != confirmed.ID {
		t.Fatalf("attendance records = %+v, want confirmed registration %d", records, confirmed.ID)
	}
	savedRecord, err := attendance.SaveRecord(ctx, AttendanceRecordInput{
		SessionID:      session.ID,
		RegistrationID: confirmed.ID,
		Status:         "present",
		Note:           "정상 출석",
	})
	if err != nil {
		t.Fatalf("SaveRecord() error = %v", err)
	}
	if savedRecord.Status != "present" {
		t.Fatalf("savedRecord.Status = %q, want present", savedRecord.Status)
	}

	assertWorkflowExport(t, ctx, exports.ExportMembers)
	assertWorkflowExport(t, ctx, exports.ExportCourseOfferings)
	assertWorkflowExport(t, ctx, exports.ExportRegistrations)
	assertWorkflowExport(t, ctx, func(ctx context.Context) (ExportResult, error) {
		return exports.ExportLotteryResults(ctx, summary.RunID)
	})
	assertWorkflowExport(t, ctx, func(ctx context.Context) (ExportResult, error) {
		return exports.ExportAttendanceSession(ctx, session.ID)
	})
	assertWorkflowExport(t, ctx, func(ctx context.Context) (ExportResult, error) {
		return exports.ExportAttendanceOffering(ctx, popular.ID)
	})

	change, err := registrations.CancelWithPromotion(ctx, confirmed.ID)
	if err != nil {
		t.Fatalf("CancelWithPromotion() error = %v", err)
	}
	if change.Registration.Status != "cancelled" {
		t.Fatalf("cancelled status = %q, want cancelled", change.Registration.Status)
	}
	if change.Promoted == nil {
		t.Fatal("promoted = nil, want next waitlisted registration")
	}
	if change.Promoted.Status != "selected" {
		t.Fatalf("promoted.Status = %q, want selected", change.Promoted.Status)
	}

	if err := audits.Record(ctx, AuditEvent{Action: "workflow.simulation", EntityType: "registration", EntityID: firstRegistration.ID, Summary: "MVP 업무 흐름 시뮬레이션"}); err != nil {
		t.Fatalf("Audit Record() error = %v", err)
	}
	auditLogs, err := audits.ListRecent(ctx, 5)
	if err != nil {
		t.Fatalf("Audit ListRecent() error = %v", err)
	}
	if len(auditLogs) == 0 || auditLogs[0].Action != "workflow.simulation" {
		t.Fatalf("auditLogs = %+v, want workflow.simulation", auditLogs)
	}

	backup, err := backups.CreateBackup(ctx)
	if err != nil {
		t.Fatalf("CreateBackup() error = %v", err)
	}
	assertWorkflowFile(t, backup.Path)
	status, err := backups.Status(ctx)
	if err != nil {
		t.Fatalf("Backup Status() error = %v", err)
	}
	if status.Latest == nil || status.TotalCount != 1 || !status.RetentionOn {
		t.Fatalf("backup status = %+v, want one retained backup", status)
	}
}

func newWorkflowTestDB(t *testing.T) (*sql.DB, string) {
	t.Helper()

	ctx := context.Background()
	path := filepath.Join(t.TempDir(), "center.db")
	db, err := database.Open(ctx, database.Options{Path: path})
	if err != nil {
		t.Fatalf("database.Open() error = %v", err)
	}
	t.Cleanup(func() {
		if err := db.Close(); err != nil {
			t.Fatalf("close db: %v", err)
		}
	})
	if err := migration.Run(ctx, db, nil); err != nil {
		t.Fatalf("migration.Run() error = %v", err)
	}
	return db, path
}

func createWorkflowOffering(t *testing.T, ctx context.Context, courses *CourseService, input CourseOfferingInput) domain.CourseOffering {
	t.Helper()
	offering, err := courses.CreateOffering(ctx, input)
	if err != nil {
		t.Fatalf("CreateOffering(%q) error = %v", input.CourseTitle, err)
	}
	return offering
}

func setWorkflowTermLimit(t *testing.T, ctx context.Context, db *sql.DB, termName string, limit int) {
	t.Helper()
	result, err := db.ExecContext(ctx, "UPDATE terms SET max_registrations_per_member = ? WHERE name = ?;", limit, termName)
	if err != nil {
		t.Fatalf("set term limit: %v", err)
	}
	affected, err := result.RowsAffected()
	if err != nil {
		t.Fatalf("read affected term limit rows: %v", err)
	}
	if affected != 1 {
		t.Fatalf("affected term limit rows = %d, want 1", affected)
	}
}

func createWorkflowMember(t *testing.T, ctx context.Context, members *MemberService, memberNo string, name string) domain.Member {
	t.Helper()
	member, err := members.Create(ctx, MemberInput{MemberNo: memberNo, Name: name})
	if err != nil {
		t.Fatalf("Create member %q error = %v", name, err)
	}
	return member
}

func createWorkflowRegistration(t *testing.T, ctx context.Context, registrations *RegistrationService, memberID int64, offeringID int64) domain.Registration {
	t.Helper()
	registration, err := registrations.Create(ctx, RegistrationInput{MemberID: memberID, OfferingID: offeringID})
	if err != nil {
		t.Fatalf("Create registration member=%d offering=%d error = %v", memberID, offeringID, err)
	}
	return registration
}

func assertWorkflowErrorContains(t *testing.T, err error, want string) {
	t.Helper()
	if !strings.Contains(err.Error(), want) {
		t.Fatalf("error = %q, want substring %q", err.Error(), want)
	}
}

func findWorkflowRegistrationByStatus(t *testing.T, registrations []domain.Registration, status string) domain.Registration {
	t.Helper()
	for _, registration := range registrations {
		if registration.Status == status {
			return registration
		}
	}
	t.Fatalf("registrations = %+v, want status %q", registrations, status)
	return domain.Registration{}
}

func assertWorkflowExport(t *testing.T, ctx context.Context, export func(context.Context) (ExportResult, error)) {
	t.Helper()
	result, err := export(ctx)
	if err != nil {
		t.Fatalf("export error = %v", err)
	}
	assertWorkflowFile(t, result.Path)
	if result.FileName == "" {
		t.Fatal("export FileName is empty")
	}
}

func assertWorkflowFile(t *testing.T, path string) {
	t.Helper()
	info, err := os.Stat(path)
	if err != nil {
		t.Fatalf("stat %s: %v", path, err)
	}
	if info.Size() == 0 {
		t.Fatalf("%s size = 0, want non-empty file", path)
	}
}
