package web

import (
	"bytes"
	"context"
	"io"
	"mime/multipart"
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
			runs: []domain.LotteryRun{{ID: 7, TermName: "2026 여름", OfferingID: 1, CourseTitle: "요가 기초", TotalCount: 2, SelectedCount: 1, WaitlistedCount: 1}},
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
	if !strings.Contains(rec.Body.String(), "재추첨 확인") {
		t.Fatalf("body = %q, want rerun confirmation", rec.Body.String())
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

func TestRouterRunsForcedLotteryRerun(t *testing.T) {
	lotteries := &fakeLotteryService{}
	router := NewRouter(RouterOptions{
		Courses:   &fakeCourseService{},
		Lotteries: lotteries,
	})
	form := url.Values{"offering_id": {"7"}, "force_rerun": {"true"}, "confirm_rerun": {"true"}}
	req := httptest.NewRequest(http.MethodPost, "/admin/lottery/run", strings.NewReader(form.Encode()))
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusSeeOther {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusSeeOther)
	}
	if !lotteries.forceRerun {
		t.Fatal("forceRerun = false, want true")
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

func TestRouterServesImportsPage(t *testing.T) {
	router := NewRouter(RouterOptions{
		DisplayName: "배움마루",
		Version:     "test",
	})
	req := httptest.NewRequest(http.MethodGet, "/admin/imports", nil)
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusOK)
	}
	if !strings.Contains(rec.Body.String(), "엑셀 가져오기") {
		t.Fatalf("body = %q, want import page", rec.Body.String())
	}
}

func TestRouterDownloadsMemberImportTemplate(t *testing.T) {
	router := NewRouter(RouterOptions{
		Imports: &fakeImportService{
			memberTemplate: service.ImportTemplate{FileName: "member-import-template.xlsx", Content: []byte("xlsx")},
		},
	})
	req := httptest.NewRequest(http.MethodGet, "/admin/imports/members/template", nil)
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusOK)
	}
	if got := rec.Header().Get("Content-Disposition"); got != `attachment; filename="member-import-template.xlsx"` {
		t.Fatalf("Content-Disposition = %q, want template attachment", got)
	}
	if rec.Body.String() != "xlsx" {
		t.Fatalf("body = %q, want template contents", rec.Body.String())
	}
}

func TestRouterImportsMembers(t *testing.T) {
	imports := &fakeImportService{
		memberResult: service.ImportResult{Kind: "회원", CreatedCount: 2},
	}
	router := NewRouter(RouterOptions{Imports: imports})
	body, contentType := multipartBody(t, "members.xlsx", []byte("xlsx"))
	req := httptest.NewRequest(http.MethodPost, "/admin/imports/members", body)
	req.Header.Set("Content-Type", contentType)
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusOK)
	}
	if !imports.memberImportCalled {
		t.Fatal("member import was not called")
	}
	if !strings.Contains(rec.Body.String(), "성공 2건") {
		t.Fatalf("body = %q, want import result", rec.Body.String())
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
			files:  []domain.BackupFile{{FileName: "backup.db", SizeBytes: 10, CreatedAt: "2026-07-01T10:00:00Z"}},
			status: domain.BackupStatus{Latest: &domain.BackupFile{FileName: "backup.db", SizeBytes: 10, CreatedAt: "2026-07-01T10:00:00Z"}, TotalCount: 1, TotalBytes: 10, KeepDays: 30, RetentionOn: true},
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
	if !strings.Contains(rec.Body.String(), "보관 30일") {
		t.Fatalf("body = %q, want retention status", rec.Body.String())
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
	if !backups.pruneCalled {
		t.Fatal("pruneCalled = false, want true")
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

func TestRouterServesAttendancePage(t *testing.T) {
	router := NewRouter(RouterOptions{
		DisplayName: "배움마루",
		Version:     "test",
		Courses: &fakeCourseService{
			offerings: []domain.CourseOffering{{ID: 1, CourseTitle: "요가 기초", TermName: "2026 여름", Weekday: 1, StartTime: "09:00", EndTime: "10:00"}},
		},
		Attendance: &fakeAttendanceService{
			sessions:  []domain.AttendanceSession{{ID: 2, OfferingID: 1, CourseTitle: "요가 기초", SessionDate: "2026-07-01"}},
			confirmed: []domain.Registration{{ID: 3, MemberName: "김배움", Status: "confirmed"}},
			records:   []domain.AttendanceRecord{{SessionID: 2, RegistrationID: 3, MemberName: "김배움", Status: "present"}},
		},
	})
	req := httptest.NewRequest(http.MethodGet, "/admin/attendance?offering_id=1&session_id=2", nil)
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusOK)
	}
	if !strings.Contains(rec.Body.String(), "김배움") {
		t.Fatalf("body = %q, want participant", rec.Body.String())
	}
	if !strings.Contains(rec.Body.String(), "출석 입력") {
		t.Fatalf("body = %q, want attendance entry section", rec.Body.String())
	}
	if !strings.Contains(rec.Body.String(), "/admin/exports/attendance-session?session_id=2") {
		t.Fatalf("body = %q, want attendance session export link", rec.Body.String())
	}
	if !strings.Contains(rec.Body.String(), "/admin/exports/attendance-offering?offering_id=1") {
		t.Fatalf("body = %q, want attendance offering export link", rec.Body.String())
	}
}

func TestRouterCreatesAttendanceSession(t *testing.T) {
	attendance := &fakeAttendanceService{}
	router := NewRouter(RouterOptions{
		Courses:    &fakeCourseService{},
		Attendance: attendance,
	})
	form := url.Values{
		"offering_id":  {"1"},
		"session_date": {"2026-07-01"},
		"note":         {"1회차"},
	}
	req := httptest.NewRequest(http.MethodPost, "/admin/attendance/session", strings.NewReader(form.Encode()))
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusSeeOther {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusSeeOther)
	}
	if attendance.createdSession.OfferingID != 1 || attendance.createdSession.SessionDate != "2026-07-01" {
		t.Fatalf("createdSession = %+v, want offering/date", attendance.createdSession)
	}
}

func TestRouterSavesAttendanceRecord(t *testing.T) {
	attendance := &fakeAttendanceService{}
	router := NewRouter(RouterOptions{
		Courses:    &fakeCourseService{},
		Attendance: attendance,
	})
	form := url.Values{
		"offering_id":     {"1"},
		"session_id":      {"2"},
		"registration_id": {"3"},
		"status":          {"present"},
	}
	req := httptest.NewRequest(http.MethodPost, "/admin/attendance/record", strings.NewReader(form.Encode()))
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusSeeOther {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusSeeOther)
	}
	if attendance.savedRecord.SessionID != 2 || attendance.savedRecord.RegistrationID != 3 || attendance.savedRecord.Status != "present" {
		t.Fatalf("savedRecord = %+v, want session registration status", attendance.savedRecord)
	}
}

func TestRouterDownloadsAttendanceSessionExport(t *testing.T) {
	filePath := filepath.Join(t.TempDir(), "attendance-session.xlsx")
	if err := os.WriteFile(filePath, []byte("xlsx"), 0o644); err != nil {
		t.Fatalf("WriteFile() error = %v", err)
	}
	exports := &fakeExportService{
		attendanceSession: service.ExportResult{Path: filePath, FileName: "attendance-session.xlsx"},
	}
	router := NewRouter(RouterOptions{
		Exports: exports,
	})
	req := httptest.NewRequest(http.MethodGet, "/admin/exports/attendance-session?session_id=2", nil)
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusOK)
	}
	if exports.attendanceSessionID != 2 {
		t.Fatalf("attendanceSessionID = %d, want 2", exports.attendanceSessionID)
	}
	if rec.Body.String() != "xlsx" {
		t.Fatalf("body = %q, want file contents", rec.Body.String())
	}
}

func TestRouterDownloadsAttendanceOfferingExport(t *testing.T) {
	filePath := filepath.Join(t.TempDir(), "attendance-offering.xlsx")
	if err := os.WriteFile(filePath, []byte("xlsx"), 0o644); err != nil {
		t.Fatalf("WriteFile() error = %v", err)
	}
	exports := &fakeExportService{
		attendanceOffering: service.ExportResult{Path: filePath, FileName: "attendance-offering.xlsx"},
	}
	router := NewRouter(RouterOptions{
		Exports: exports,
	})
	req := httptest.NewRequest(http.MethodGet, "/admin/exports/attendance-offering?offering_id=1", nil)
	rec := httptest.NewRecorder()

	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusOK)
	}
	if exports.attendanceOfferingID != 1 {
		t.Fatalf("attendanceOfferingID = %d, want 1", exports.attendanceOfferingID)
	}
	if rec.Body.String() != "xlsx" {
		t.Fatalf("body = %q, want file contents", rec.Body.String())
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
	forceRerun bool
	runs       []domain.LotteryRun
}

func (f *fakeLotteryService) RunOfferingLottery(_ context.Context, offeringID int64, options ...service.LotteryRunOptions) (domain.LotteryRunSummary, error) {
	f.offeringID = offeringID
	if len(options) > 0 {
		f.forceRerun = options[0].ForceRerun
	}
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
	members              service.ExportResult
	courses              service.ExportResult
	registrations        service.ExportResult
	lotteryResults       service.ExportResult
	attendanceSession    service.ExportResult
	attendanceOffering   service.ExportResult
	lotteryRunID         int64
	attendanceSessionID  int64
	attendanceOfferingID int64
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

func (f *fakeExportService) ExportAttendanceSession(_ context.Context, sessionID int64) (service.ExportResult, error) {
	f.attendanceSessionID = sessionID
	return f.attendanceSession, nil
}

func (f *fakeExportService) ExportAttendanceOffering(_ context.Context, offeringID int64) (service.ExportResult, error) {
	f.attendanceOfferingID = offeringID
	return f.attendanceOffering, nil
}

type fakeImportService struct {
	memberResult       service.ImportResult
	courseResult       service.ImportResult
	memberTemplate     service.ImportTemplate
	courseTemplate     service.ImportTemplate
	memberImportCalled bool
	courseImportCalled bool
}

func (f *fakeImportService) ImportMembers(context.Context, io.Reader) (service.ImportResult, error) {
	f.memberImportCalled = true
	return f.memberResult, nil
}

func (f *fakeImportService) ImportCourseOfferings(context.Context, io.Reader) (service.ImportResult, error) {
	f.courseImportCalled = true
	return f.courseResult, nil
}

func (f *fakeImportService) MemberTemplate() (service.ImportTemplate, error) {
	return f.memberTemplate, nil
}

func (f *fakeImportService) CourseOfferingTemplate() (service.ImportTemplate, error) {
	return f.courseTemplate, nil
}

func multipartBody(t *testing.T, fileName string, content []byte) (*bytes.Buffer, string) {
	t.Helper()
	var body bytes.Buffer
	writer := multipart.NewWriter(&body)
	part, err := writer.CreateFormFile("file", fileName)
	if err != nil {
		t.Fatalf("CreateFormFile() error = %v", err)
	}
	if _, err := part.Write(content); err != nil {
		t.Fatalf("Write() error = %v", err)
	}
	if err := writer.Close(); err != nil {
		t.Fatalf("Close() error = %v", err)
	}
	return &body, writer.FormDataContentType()
}

type fakeBackupService struct {
	files        []domain.BackupFile
	status       domain.BackupStatus
	cleanup      domain.BackupCleanup
	created      domain.BackupFile
	createCalled bool
	pruneCalled  bool
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

func (f *fakeBackupService) Status(context.Context) (domain.BackupStatus, error) {
	return f.status, nil
}

func (f *fakeBackupService) PruneOldBackups(context.Context) (domain.BackupCleanup, error) {
	f.pruneCalled = true
	return f.cleanup, nil
}

func (f *fakeBackupService) ResolveBackupPath(string) (string, error) {
	return f.resolvedPath, nil
}

func (f *fakeBackupService) QueueRestore(_ context.Context, fileName string) (domain.RestorePlan, error) {
	f.restoreFile = fileName
	return domain.RestorePlan{FileName: fileName}, nil
}

type fakeAttendanceService struct {
	createdSession service.AttendanceSessionInput
	savedRecord    service.AttendanceRecordInput
	sessions       []domain.AttendanceSession
	confirmed      []domain.Registration
	records        []domain.AttendanceRecord
}

func (f *fakeAttendanceService) CreateSession(_ context.Context, input service.AttendanceSessionInput) (domain.AttendanceSession, error) {
	f.createdSession = input
	return domain.AttendanceSession{ID: 2, OfferingID: input.OfferingID, SessionDate: input.SessionDate}, nil
}

func (f *fakeAttendanceService) ListSessions(context.Context, int64, int) ([]domain.AttendanceSession, error) {
	return f.sessions, nil
}

func (f *fakeAttendanceService) ListConfirmedByOffering(context.Context, int64) ([]domain.Registration, error) {
	return f.confirmed, nil
}

func (f *fakeAttendanceService) ListRecordsBySession(context.Context, int64) ([]domain.AttendanceRecord, error) {
	return f.records, nil
}

func (f *fakeAttendanceService) SaveRecord(_ context.Context, input service.AttendanceRecordInput) (domain.AttendanceRecord, error) {
	f.savedRecord = input
	return domain.AttendanceRecord{SessionID: input.SessionID, RegistrationID: input.RegistrationID, Status: input.Status}, nil
}
