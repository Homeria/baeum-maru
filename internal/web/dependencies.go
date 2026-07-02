package web

import (
	"context"
	"io"
	"log/slog"

	"github.com/Homeria/baeum-maru/internal/config"
	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/service"
)

type RouterOptions struct {
	DisplayName   string
	Version       string
	Logger        *slog.Logger
	Members       MemberService
	Courses       CourseService
	Registrations RegistrationService
	Lotteries     LotteryService
	Exports       ExportService
	Imports       ImportService
	Backups       BackupService
	Attendance    AttendanceService
	Settings      SettingsService
	Audits        AuditService
}

type MemberService interface {
	Create(context.Context, service.MemberInput) (domain.Member, error)
	Search(context.Context, string, int) ([]domain.Member, error)
}

type CourseService interface {
	CreateOffering(context.Context, service.CourseOfferingInput) (domain.CourseOffering, error)
	ListOfferings(context.Context, int) ([]domain.CourseOffering, error)
}

type RegistrationService interface {
	Create(context.Context, service.RegistrationInput) (domain.Registration, error)
	Cancel(context.Context, int64) (domain.Registration, error)
	Confirm(context.Context, int64) (domain.Registration, error)
	CancelWithPromotion(context.Context, int64) (domain.RegistrationStatusChange, error)
	ListByMember(context.Context, int64) ([]domain.Registration, error)
	ListRecent(context.Context, int) ([]domain.Registration, error)
}

type ExportService interface {
	ExportMembers(context.Context) (service.ExportResult, error)
	ExportCourseOfferings(context.Context) (service.ExportResult, error)
	ExportRegistrations(context.Context) (service.ExportResult, error)
	ExportLotteryResults(context.Context, int64) (service.ExportResult, error)
	ExportAttendanceSession(context.Context, int64) (service.ExportResult, error)
	ExportAttendanceOffering(context.Context, int64) (service.ExportResult, error)
}

type ImportService interface {
	ImportMembers(context.Context, io.Reader) (service.ImportResult, error)
	ImportCourseOfferings(context.Context, io.Reader) (service.ImportResult, error)
	MemberTemplate() (service.ImportTemplate, error)
	CourseOfferingTemplate() (service.ImportTemplate, error)
}

type LotteryService interface {
	RunOfferingLottery(context.Context, int64, ...service.LotteryRunOptions) (domain.LotteryRunSummary, error)
	ListRuns(context.Context, int) ([]domain.LotteryRun, error)
}

type BackupService interface {
	CreateBackup(context.Context) (domain.BackupFile, error)
	ListBackups(context.Context) ([]domain.BackupFile, error)
	Status(context.Context) (domain.BackupStatus, error)
	PruneOldBackups(context.Context) (domain.BackupCleanup, error)
	ResolveBackupPath(string) (string, error)
	QueueRestore(context.Context, string) (domain.RestorePlan, error)
}

type AttendanceService interface {
	CreateSession(context.Context, service.AttendanceSessionInput) (domain.AttendanceSession, error)
	ListSessions(context.Context, int64, int) ([]domain.AttendanceSession, error)
	ListConfirmedByOffering(context.Context, int64) ([]domain.Registration, error)
	ListRecordsBySession(context.Context, int64) ([]domain.AttendanceRecord, error)
	SaveRecord(context.Context, service.AttendanceRecordInput) (domain.AttendanceRecord, error)
}

type SettingsService interface {
	Get(context.Context) (config.Config, error)
	Update(context.Context, service.SettingsInput) (config.Config, error)
}

type AuditService interface {
	Record(context.Context, service.AuditEvent) error
	ListRecent(context.Context, int) ([]domain.AuditLog, error)
}
