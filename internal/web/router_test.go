package web

import (
	"context"
	"net/http"
	"net/http/httptest"
	"net/url"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/service"
)

func TestRouterServesBasicPages(t *testing.T) {
	router := NewRouter(RouterOptions{
		DisplayName: "배움마루",
		Version:     "test",
	})

	tests := []struct {
		path string
		want string
	}{
		{path: "/", want: "로컬 호스팅 수강신청 업무 도구"},
		{path: "/admin", want: "관리 화면"},
		{path: "/reception", want: "접수 화면"},
	}

	for _, tt := range tests {
		t.Run(tt.path, func(t *testing.T) {
			req := httptest.NewRequest(http.MethodGet, tt.path, nil)
			rec := httptest.NewRecorder()

			router.ServeHTTP(rec, req)

			if rec.Code != http.StatusOK {
				t.Fatalf("status = %d, want %d", rec.Code, http.StatusOK)
			}
			if !strings.Contains(rec.Body.String(), tt.want) {
				t.Fatalf("body = %q, want substring %q", rec.Body.String(), tt.want)
			}
		})
	}
}

func TestRouterServesHealthCheck(t *testing.T) {
	router := NewRouter(RouterOptions{})
	req := httptest.NewRequest(http.MethodGet, "/healthz", nil)
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusOK)
	}
	if rec.Body.String() != "ok\n" {
		t.Fatalf("body = %q, want ok", rec.Body.String())
	}
}

func TestRouterRejectsUnsupportedMethod(t *testing.T) {
	router := NewRouter(RouterOptions{})
	req := httptest.NewRequest(http.MethodPost, "/admin", nil)
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusMethodNotAllowed {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusMethodNotAllowed)
	}
}

func TestRouterServesMemberManagement(t *testing.T) {
	router := NewRouter(RouterOptions{
		DisplayName: "배움마루",
		Version:     "test",
		Members: &fakeMemberService{
			members: []domain.Member{{ID: 1, Name: "김배움"}},
		},
	})
	req := httptest.NewRequest(http.MethodGet, "/admin/members?q=김", nil)
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusOK)
	}
	if !strings.Contains(rec.Body.String(), "김배움") {
		t.Fatalf("body = %q, want member name", rec.Body.String())
	}
}

func TestRouterCreatesMember(t *testing.T) {
	members := &fakeMemberService{}
	router := NewRouter(RouterOptions{
		Members: members,
	})
	form := url.Values{"name": {"김배움"}}
	req := httptest.NewRequest(http.MethodPost, "/admin/members", strings.NewReader(form.Encode()))
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusSeeOther {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusSeeOther)
	}
	if members.created.Name != "김배움" {
		t.Fatalf("created.Name = %q, want 김배움", members.created.Name)
	}
}

func TestRouterServesCourseManagement(t *testing.T) {
	router := NewRouter(RouterOptions{
		DisplayName: "배움마루",
		Version:     "test",
		Courses: &fakeCourseService{
			offerings: []domain.CourseOffering{{ID: 1, CourseTitle: "요가 기초", Weekday: 1, StartTime: "09:00", EndTime: "10:00"}},
		},
	})
	req := httptest.NewRequest(http.MethodGet, "/admin/courses", nil)
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusOK)
	}
	if !strings.Contains(rec.Body.String(), "요가 기초") {
		t.Fatalf("body = %q, want course title", rec.Body.String())
	}
}

func TestRouterCreatesCourseOffering(t *testing.T) {
	courses := &fakeCourseService{}
	router := NewRouter(RouterOptions{
		Courses: courses,
	})
	form := url.Values{
		"course_title": {"요가 기초"},
		"capacity":     {"20"},
		"weekday":      {"1"},
		"start_time":   {"09:00"},
		"end_time":     {"10:00"},
	}
	req := httptest.NewRequest(http.MethodPost, "/admin/courses", strings.NewReader(form.Encode()))
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusSeeOther {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusSeeOther)
	}
	if courses.created.CourseTitle != "요가 기초" {
		t.Fatalf("created.CourseTitle = %q, want 요가 기초", courses.created.CourseTitle)
	}
}

func TestRouterServesReceptionWithSelectedMember(t *testing.T) {
	router := NewRouter(RouterOptions{
		DisplayName: "배움마루",
		Version:     "test",
		Members: &fakeMemberService{
			members: []domain.Member{{ID: 1, Name: "김배움"}},
		},
		Courses: &fakeCourseService{
			offerings: []domain.CourseOffering{{ID: 2, CourseTitle: "요가 기초", TermName: "기본 회차", Weekday: 1, StartTime: "09:00", EndTime: "10:00", Capacity: 20}},
		},
		Registrations: &fakeRegistrationService{
			byMember: []domain.Registration{{ID: 3, MemberID: 1, CourseTitle: "요가 기초", Status: "applied"}},
		},
	})
	req := httptest.NewRequest(http.MethodGet, "/reception?member_id=1", nil)
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusOK)
	}
	if !strings.Contains(rec.Body.String(), "요가 기초") {
		t.Fatalf("body = %q, want course title", rec.Body.String())
	}
}

func TestRouterCreatesRegistration(t *testing.T) {
	registrations := &fakeRegistrationService{}
	router := NewRouter(RouterOptions{
		Members:       &fakeMemberService{},
		Courses:       &fakeCourseService{},
		Registrations: registrations,
	})
	form := url.Values{
		"member_id":   {"1"},
		"offering_id": {"2"},
	}
	req := httptest.NewRequest(http.MethodPost, "/reception", strings.NewReader(form.Encode()))
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusSeeOther {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusSeeOther)
	}
	if registrations.created.MemberID != 1 || registrations.created.OfferingID != 2 {
		t.Fatalf("created = %+v, want member 1 offering 2", registrations.created)
	}
}

func TestRouterCancelsRegistration(t *testing.T) {
	registrations := &fakeRegistrationService{}
	router := NewRouter(RouterOptions{
		Registrations: registrations,
	})
	form := url.Values{
		"registration_id": {"3"},
		"member_id":       {"1"},
	}
	req := httptest.NewRequest(http.MethodPost, "/reception/cancel", strings.NewReader(form.Encode()))
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusSeeOther {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusSeeOther)
	}
	if registrations.cancelledID != 3 {
		t.Fatalf("cancelledID = %d, want 3", registrations.cancelledID)
	}
}

func TestRouterServesRegistrationManagement(t *testing.T) {
	router := NewRouter(RouterOptions{
		DisplayName: "배움마루",
		Version:     "test",
		Registrations: &fakeRegistrationService{
			recent: []domain.Registration{{ID: 1, MemberName: "김배움", CourseTitle: "요가 기초", Status: "selected"}},
		},
	})
	req := httptest.NewRequest(http.MethodGet, "/admin/registrations", nil)
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusOK)
	}
	if !strings.Contains(rec.Body.String(), "김배움") {
		t.Fatalf("body = %q, want member name", rec.Body.String())
	}
	if !strings.Contains(rec.Body.String(), `value="confirm"`) {
		t.Fatalf("body = %q, want confirm action", rec.Body.String())
	}
}

func TestRouterConfirmsRegistrationStatus(t *testing.T) {
	registrations := &fakeRegistrationService{}
	router := NewRouter(RouterOptions{
		Registrations: registrations,
	})
	form := url.Values{
		"registration_id": {"3"},
		"action":          {"confirm"},
	}
	req := httptest.NewRequest(http.MethodPost, "/admin/registrations/status", strings.NewReader(form.Encode()))
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusSeeOther {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusSeeOther)
	}
	if registrations.confirmedID != 3 {
		t.Fatalf("confirmedID = %d, want 3", registrations.confirmedID)
	}
}

func TestRouterCancelsRegistrationStatus(t *testing.T) {
	registrations := &fakeRegistrationService{}
	router := NewRouter(RouterOptions{
		Registrations: registrations,
	})
	form := url.Values{
		"registration_id": {"3"},
		"action":          {"cancel"},
	}
	req := httptest.NewRequest(http.MethodPost, "/admin/registrations/status", strings.NewReader(form.Encode()))
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusSeeOther {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusSeeOther)
	}
	if registrations.cancelledID != 3 {
		t.Fatalf("cancelledID = %d, want 3", registrations.cancelledID)
	}
}

func TestRouterServesLotteryPage(t *testing.T) {
	router := NewRouter(RouterOptions{
		DisplayName: "배움마루",
		Version:     "test",
		Courses: &fakeCourseService{
			offerings: []domain.CourseOffering{{ID: 1, CourseTitle: "요가 기초", Capacity: 1, RegistrationCount: 2, Weekday: 1, StartTime: "09:00", EndTime: "10:00"}},
		},
		Lotteries: &fakeLotteryService{
			runs: []domain.LotteryRun{{ID: 7, TermName: "2026 여름", CourseTitle: "요가 기초", TotalCount: 2, SelectedCount: 1, WaitlistedCount: 1}},
		},
	})
	req := httptest.NewRequest(http.MethodGet, "/admin/lottery", nil)
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusOK)
	}
	if !strings.Contains(rec.Body.String(), "요가 기초") {
		t.Fatalf("body = %q, want course title", rec.Body.String())
	}
	if !strings.Contains(rec.Body.String(), "/admin/exports/lottery-results?run_id=7") {
		t.Fatalf("body = %q, want lottery export link", rec.Body.String())
	}
}

func TestRouterRunsLottery(t *testing.T) {
	lotteries := &fakeLotteryService{}
	router := NewRouter(RouterOptions{
		Courses:   &fakeCourseService{},
		Lotteries: lotteries,
	})
	form := url.Values{"offering_id": {"7"}}
	req := httptest.NewRequest(http.MethodPost, "/admin/lottery/run", strings.NewReader(form.Encode()))
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusSeeOther {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusSeeOther)
	}
	if lotteries.offeringID != 7 {
		t.Fatalf("offeringID = %d, want 7", lotteries.offeringID)
	}
	if location := rec.Header().Get("Location"); !strings.Contains(location, "/admin/lottery?message=") {
		t.Fatalf("Location = %q, want lottery redirect", location)
	}
}

func TestRouterServesExportsPage(t *testing.T) {
	router := NewRouter(RouterOptions{
		DisplayName: "배움마루",
		Version:     "test",
	})
	req := httptest.NewRequest(http.MethodGet, "/admin/exports", nil)
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusOK)
	}
	if !strings.Contains(rec.Body.String(), "엑셀 내보내기") {
		t.Fatalf("body = %q, want export page", rec.Body.String())
	}
}

func TestRouterDownloadsMemberExport(t *testing.T) {
	filePath := filepath.Join(t.TempDir(), "members.xlsx")
	if err := os.WriteFile(filePath, []byte("xlsx"), 0o644); err != nil {
		t.Fatalf("WriteFile() error = %v", err)
	}
	router := NewRouter(RouterOptions{
		Exports: &fakeExportService{
			members: service.ExportResult{Path: filePath, FileName: "members.xlsx"},
		},
	})
	req := httptest.NewRequest(http.MethodGet, "/admin/exports/members", nil)
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusOK)
	}
	if got := rec.Header().Get("Content-Disposition"); got != `attachment; filename="members.xlsx"` {
		t.Fatalf("Content-Disposition = %q, want attachment", got)
	}
	if rec.Body.String() != "xlsx" {
		t.Fatalf("body = %q, want file contents", rec.Body.String())
	}
}

func TestRouterDownloadsLotteryResultsExport(t *testing.T) {
	filePath := filepath.Join(t.TempDir(), "lottery-results.xlsx")
	if err := os.WriteFile(filePath, []byte("xlsx"), 0o644); err != nil {
		t.Fatalf("WriteFile() error = %v", err)
	}
	exports := &fakeExportService{
		lotteryResults: service.ExportResult{Path: filePath, FileName: "lottery-results.xlsx"},
	}
	router := NewRouter(RouterOptions{
		Exports: exports,
	})
	req := httptest.NewRequest(http.MethodGet, "/admin/exports/lottery-results?run_id=7", nil)
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusOK)
	}
	if exports.lotteryRunID != 7 {
		t.Fatalf("lotteryRunID = %d, want 7", exports.lotteryRunID)
	}
	if rec.Body.String() != "xlsx" {
		t.Fatalf("body = %q, want file contents", rec.Body.String())
	}
}

func TestRouterServesBackupsPage(t *testing.T) {
	router := NewRouter(RouterOptions{
		DisplayName: "배움마루",
		Version:     "test",
		Backups: &fakeBackupService{
			files: []domain.BackupFile{{FileName: "backup.db", SizeBytes: 10, CreatedAt: "2026-07-01T10:00:00Z"}},
		},
	})
	req := httptest.NewRequest(http.MethodGet, "/admin/backups", nil)
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusOK)
	}
	if !strings.Contains(rec.Body.String(), "backup.db") {
		t.Fatalf("body = %q, want backup file", rec.Body.String())
	}
}

func TestRouterCreatesBackup(t *testing.T) {
	backups := &fakeBackupService{
		created: domain.BackupFile{FileName: "backup.db"},
	}
	router := NewRouter(RouterOptions{
		Backups: backups,
	})
	req := httptest.NewRequest(http.MethodPost, "/admin/backups/create", nil)
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusSeeOther {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusSeeOther)
	}
	if !backups.createCalled {
		t.Fatal("createCalled = false, want true")
	}
}

func TestRouterDownloadsBackup(t *testing.T) {
	filePath := filepath.Join(t.TempDir(), "backup.db")
	if err := os.WriteFile(filePath, []byte("backup"), 0o644); err != nil {
		t.Fatalf("WriteFile() error = %v", err)
	}
	router := NewRouter(RouterOptions{
		Backups: &fakeBackupService{resolvedPath: filePath},
	})
	req := httptest.NewRequest(http.MethodGet, "/admin/backups/download?file=backup.db", nil)
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusOK)
	}
	if rec.Body.String() != "backup" {
		t.Fatalf("body = %q, want backup contents", rec.Body.String())
	}
}

func TestRouterQueuesBackupRestore(t *testing.T) {
	backups := &fakeBackupService{}
	router := NewRouter(RouterOptions{
		Backups: backups,
	})
	form := url.Values{"file": {"backup.db"}}
	req := httptest.NewRequest(http.MethodPost, "/admin/backups/restore", strings.NewReader(form.Encode()))
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusSeeOther {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusSeeOther)
	}
	if backups.restoreFile != "backup.db" {
		t.Fatalf("restoreFile = %q, want backup.db", backups.restoreFile)
	}
}

type fakeMemberService struct {
	created service.MemberInput
	members []domain.Member
}

func (f *fakeMemberService) Create(_ context.Context, input service.MemberInput) (domain.Member, error) {
	f.created = input
	return domain.Member{ID: 1, Name: input.Name}, nil
}

func (f *fakeMemberService) Search(_ context.Context, _ string, _ int) ([]domain.Member, error) {
	return f.members, nil
}

type fakeCourseService struct {
	created   service.CourseOfferingInput
	offerings []domain.CourseOffering
}

func (f *fakeCourseService) CreateOffering(_ context.Context, input service.CourseOfferingInput) (domain.CourseOffering, error) {
	f.created = input
	return domain.CourseOffering{ID: 1, CourseTitle: input.CourseTitle}, nil
}

func (f *fakeCourseService) ListOfferings(_ context.Context, _ int) ([]domain.CourseOffering, error) {
	return f.offerings, nil
}

type fakeRegistrationService struct {
	created     service.RegistrationInput
	cancelledID int64
	confirmedID int64
	byMember    []domain.Registration
	recent      []domain.Registration
}

func (f *fakeRegistrationService) Create(_ context.Context, input service.RegistrationInput) (domain.Registration, error) {
	f.created = input
	return domain.Registration{ID: 1, MemberID: input.MemberID, OfferingID: input.OfferingID, Status: "applied"}, nil
}

func (f *fakeRegistrationService) Cancel(_ context.Context, id int64) (domain.Registration, error) {
	f.cancelledID = id
	return domain.Registration{ID: id, Status: "cancelled"}, nil
}

func (f *fakeRegistrationService) Confirm(_ context.Context, id int64) (domain.Registration, error) {
	f.confirmedID = id
	return domain.Registration{ID: id, Status: "confirmed"}, nil
}

func (f *fakeRegistrationService) CancelWithPromotion(_ context.Context, id int64) (domain.RegistrationStatusChange, error) {
	f.cancelledID = id
	return domain.RegistrationStatusChange{
		Registration: domain.Registration{ID: id, Status: "cancelled"},
	}, nil
}

func (f *fakeRegistrationService) ListByMember(_ context.Context, _ int64) ([]domain.Registration, error) {
	return f.byMember, nil
}

func (f *fakeRegistrationService) ListRecent(_ context.Context, _ int) ([]domain.Registration, error) {
	return f.recent, nil
}

type fakeLotteryService struct {
	offeringID int64
	runs       []domain.LotteryRun
}

func (f *fakeLotteryService) RunOfferingLottery(_ context.Context, offeringID int64) (domain.LotteryRunSummary, error) {
	f.offeringID = offeringID
	return domain.LotteryRunSummary{
		OfferingID:      offeringID,
		CourseTitle:     "요가 기초",
		SelectedCount:   1,
		WaitlistedCount: 1,
	}, nil
}

func (f *fakeLotteryService) ListRuns(context.Context, int) ([]domain.LotteryRun, error) {
	return f.runs, nil
}

type fakeExportService struct {
	members        service.ExportResult
	courses        service.ExportResult
	registrations  service.ExportResult
	lotteryResults service.ExportResult
	lotteryRunID   int64
}

func (f *fakeExportService) ExportMembers(context.Context) (service.ExportResult, error) {
	return f.members, nil
}

func (f *fakeExportService) ExportCourseOfferings(context.Context) (service.ExportResult, error) {
	return f.courses, nil
}

func (f *fakeExportService) ExportRegistrations(context.Context) (service.ExportResult, error) {
	return f.registrations, nil
}

func (f *fakeExportService) ExportLotteryResults(_ context.Context, runID int64) (service.ExportResult, error) {
	f.lotteryRunID = runID
	return f.lotteryResults, nil
}

type fakeBackupService struct {
	files        []domain.BackupFile
	created      domain.BackupFile
	createCalled bool
	resolvedPath string
	restoreFile  string
}

func (f *fakeBackupService) CreateBackup(context.Context) (domain.BackupFile, error) {
	f.createCalled = true
	return f.created, nil
}

func (f *fakeBackupService) ListBackups(context.Context) ([]domain.BackupFile, error) {
	return f.files, nil
}

func (f *fakeBackupService) ResolveBackupPath(string) (string, error) {
	return f.resolvedPath, nil
}

func (f *fakeBackupService) QueueRestore(_ context.Context, fileName string) (domain.RestorePlan, error) {
	f.restoreFile = fileName
	return domain.RestorePlan{FileName: fileName}, nil
}
