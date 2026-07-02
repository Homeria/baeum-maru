// Package web contains HTTP handlers, middleware, and route wiring.
package web

import (
	"context"
	"github.com/Homeria/baeum-maru/internal/config"
	"html/template"
	"io"
	"log/slog"
	"net/http"
	"strconv"

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

type pageData struct {
	Title       string
	DisplayName string
	Version     string
	Heading     string
	Description string
}

var pageTemplate = mustPageTemplate("page", "page.html", nil)

func NewRouter(opts RouterOptions) http.Handler {
	if opts.DisplayName == "" {
		opts.DisplayName = "배움마루"
	}
	if opts.Version == "" {
		opts.Version = "dev"
	}
	if opts.Logger == nil {
		opts.Logger = slog.Default()
	}

	mux := http.NewServeMux()
	mux.HandleFunc("/", exactPath("/", renderPage(opts, pageData{
		Title:       "홈",
		Heading:     "배움마루",
		Description: "로컬 호스팅 수강신청 업무 도구가 실행 중입니다.",
	})))
	mux.HandleFunc("/admin", exactPath("/admin", renderPage(opts, pageData{
		Title:       "관리",
		Heading:     "관리 화면",
		Description: "회원, 강좌, 신청 현황, 추첨, 출력을 관리하는 화면입니다.",
	})))
	mux.HandleFunc("/reception", receptionHandler(opts))
	mux.HandleFunc("/admin/members", membersHandler(opts))
	mux.HandleFunc("/admin/courses", coursesHandler(opts))
	mux.HandleFunc("/admin/registrations", registrationsHandler(opts))
	mux.HandleFunc("/admin/registrations/status", registrationStatusHandler(opts))
	mux.HandleFunc("/admin/lottery", lotteryHandler(opts))
	mux.HandleFunc("/admin/lottery/run", runLotteryHandler(opts))
	mux.HandleFunc("/admin/exports", exportsHandler(opts))
	mux.HandleFunc("/admin/exports/members", exportMembersHandler(opts))
	mux.HandleFunc("/admin/exports/courses", exportCoursesHandler(opts))
	mux.HandleFunc("/admin/exports/registrations", exportRegistrationsHandler(opts))
	mux.HandleFunc("/admin/exports/lottery-results", exportLotteryResultsHandler(opts))
	mux.HandleFunc("/admin/exports/attendance-session", exportAttendanceSessionHandler(opts))
	mux.HandleFunc("/admin/exports/attendance-offering", exportAttendanceOfferingHandler(opts))
	mux.HandleFunc("/admin/imports", importsHandler(opts))
	mux.HandleFunc("/admin/imports/members", importMembersHandler(opts))
	mux.HandleFunc("/admin/imports/courses", importCoursesHandler(opts))
	mux.HandleFunc("/admin/imports/members/template", memberImportTemplateHandler(opts))
	mux.HandleFunc("/admin/imports/courses/template", courseImportTemplateHandler(opts))
	mux.HandleFunc("/admin/backups", backupsHandler(opts))
	mux.HandleFunc("/admin/backups/create", createBackupHandler(opts))
	mux.HandleFunc("/admin/backups/download", downloadBackupHandler(opts))
	mux.HandleFunc("/admin/backups/restore", restoreBackupHandler(opts))
	mux.HandleFunc("/admin/attendance", attendanceHandler(opts))
	mux.HandleFunc("/admin/attendance/session", createAttendanceSessionHandler(opts))
	mux.HandleFunc("/admin/attendance/record", saveAttendanceRecordHandler(opts))
	mux.HandleFunc("/admin/settings", settingsHandler(opts))
	mux.HandleFunc("/admin/audit-logs", auditLogsHandler(opts))
	mux.HandleFunc("/reception/cancel", cancelRegistrationHandler(opts))
	mux.Handle("/static/", http.StripPrefix("/static/", staticFileHandler()))
	mux.HandleFunc("/healthz", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodGet {
			w.Header().Set("Allow", http.MethodGet)
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		w.Header().Set("Content-Type", "text/plain; charset=utf-8")
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte("ok\n"))
	})

	return mux
}

func exactPath(path string, next http.HandlerFunc) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != path {
			http.NotFound(w, r)
			return
		}
		if r.Method != http.MethodGet {
			w.Header().Set("Allow", http.MethodGet)
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		next(w, r)
	}
}

type membersPageData struct {
	DisplayName string
	Version     string
	Query       string
	Error       string
	Members     []domain.Member
}

var membersTemplate = mustPageTemplate("members", "members.html", nil)

func membersHandler(opts RouterOptions) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if opts.Members == nil {
			http.Error(w, "member service is not configured", http.StatusServiceUnavailable)
			return
		}

		switch r.Method {
		case http.MethodGet:
			renderMembers(w, r, opts, "")
		case http.MethodPost:
			if err := r.ParseForm(); err != nil {
				http.Error(w, "invalid form", http.StatusBadRequest)
				return
			}
			created, err := opts.Members.Create(r.Context(), service.MemberInput{
				MemberNo:   r.FormValue("member_no"),
				Name:       r.FormValue("name"),
				GenderCode: r.FormValue("gender_code"),
				BirthDate:  r.FormValue("birth_date"),
				Phone:      r.FormValue("phone"),
				Note:       r.FormValue("note"),
			})
			if err != nil {
				renderMembers(w, r, opts, err.Error())
				return
			}
			recordAudit(r, opts, "member.create", "member", created.ID, "회원 등록 #"+strconv.FormatInt(created.ID, 10))
			http.Redirect(w, r, "/admin/members", http.StatusSeeOther)
		default:
			w.Header().Set("Allow", "GET, POST")
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		}
	}
}

func renderMembers(w http.ResponseWriter, r *http.Request, opts RouterOptions, message string) {
	query := r.URL.Query().Get("q")
	members, err := opts.Members.Search(r.Context(), query, 50)
	if err != nil {
		message = err.Error()
	}

	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	if err := membersTemplate.ExecuteTemplate(w, "members", membersPageData{
		DisplayName: opts.DisplayName,
		Version:     opts.Version,
		Query:       query,
		Error:       message,
		Members:     members,
	}); err != nil {
		opts.Logger.Error("render members failed", "error", err)
	}
}

type coursesPageData struct {
	DisplayName string
	Version     string
	Error       string
	Offerings   []domain.CourseOffering
}

var coursesTemplate = mustPageTemplate("courses", "courses.html", template.FuncMap{"weekdayLabel": weekdayLabel})

func coursesHandler(opts RouterOptions) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if opts.Courses == nil {
			http.Error(w, "course service is not configured", http.StatusServiceUnavailable)
			return
		}

		switch r.Method {
		case http.MethodGet:
			renderCourses(w, r, opts, "")
		case http.MethodPost:
			if err := r.ParseForm(); err != nil {
				http.Error(w, "invalid form", http.StatusBadRequest)
				return
			}
			capacity, err := strconv.Atoi(r.FormValue("capacity"))
			if err != nil {
				renderCourses(w, r, opts, "정원은 숫자로 입력해야 합니다.")
				return
			}
			weekday, err := strconv.Atoi(r.FormValue("weekday"))
			if err != nil {
				renderCourses(w, r, opts, "요일 값이 올바르지 않습니다.")
				return
			}
			created, err := opts.Courses.CreateOffering(r.Context(), service.CourseOfferingInput{
				TermName:       r.FormValue("term_name"),
				CategoryName:   r.FormValue("category_name"),
				CourseTitle:    r.FormValue("course_title"),
				InstructorName: r.FormValue("instructor_name"),
				ClassroomName:  r.FormValue("classroom_name"),
				Capacity:       capacity,
				Weekday:        weekday,
				StartTime:      r.FormValue("start_time"),
				EndTime:        r.FormValue("end_time"),
				Note:           r.FormValue("note"),
			})
			if err != nil {
				renderCourses(w, r, opts, err.Error())
				return
			}
			recordAudit(r, opts, "course.create", "course_offering", created.ID, "강좌 개설 등록 #"+strconv.FormatInt(created.ID, 10))
			http.Redirect(w, r, "/admin/courses", http.StatusSeeOther)
		default:
			w.Header().Set("Allow", "GET, POST")
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		}
	}
}

func renderCourses(w http.ResponseWriter, r *http.Request, opts RouterOptions, message string) {
	offerings, err := opts.Courses.ListOfferings(r.Context(), 100)
	if err != nil {
		message = err.Error()
	}

	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	if err := coursesTemplate.ExecuteTemplate(w, "courses", coursesPageData{
		DisplayName: opts.DisplayName,
		Version:     opts.Version,
		Error:       message,
		Offerings:   offerings,
	}); err != nil {
		opts.Logger.Error("render courses failed", "error", err)
	}
}

func weekdayLabel(weekday int) string {
	switch weekday {
	case 0:
		return "일"
	case 1:
		return "월"
	case 2:
		return "화"
	case 3:
		return "수"
	case 4:
		return "목"
	case 5:
		return "금"
	case 6:
		return "토"
	default:
		return "?"
	}
}

func renderPage(opts RouterOptions, data pageData) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		data.DisplayName = opts.DisplayName
		data.Version = opts.Version

		w.Header().Set("Content-Type", "text/html; charset=utf-8")
		if err := pageTemplate.ExecuteTemplate(w, "page", data); err != nil {
			opts.Logger.Error("render page failed", "path", r.URL.Path, "error", err)
		}
	}
}
