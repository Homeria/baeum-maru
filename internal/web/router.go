// Package web contains HTTP handlers, middleware, and route wiring.
package web

import (
	"log/slog"
	"net/http"
)

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

	appMux := http.NewServeMux()
	appMux.HandleFunc("/", exactPath("/", renderPage(opts, pageData{
		Title:       "홈",
		Heading:     "배움마루",
		Description: "로컬 호스팅 수강신청 업무 도구가 실행 중입니다.",
	})))
	appMux.HandleFunc("/admin", exactPath("/admin", renderPage(opts, pageData{
		Title:       "관리",
		Heading:     "관리 화면",
		Description: "회원, 강좌, 신청 현황, 추첨, 출력을 관리하는 화면입니다.",
	})))
	appMux.HandleFunc("/reception", receptionHandler(opts))
	appMux.HandleFunc("/admin/members", membersHandler(opts))
	appMux.HandleFunc("/admin/courses", coursesHandler(opts))
	appMux.HandleFunc("/admin/locations", locationsHandler(opts))
	appMux.HandleFunc("/admin/registrations", registrationsHandler(opts))
	appMux.HandleFunc("/admin/registrations/status", registrationStatusHandler(opts))
	appMux.HandleFunc("/admin/lottery", lotteryHandler(opts))
	appMux.HandleFunc("/admin/lottery/run", runLotteryHandler(opts))
	appMux.HandleFunc("/admin/exports", exportsHandler(opts))
	appMux.HandleFunc("/admin/exports/members", exportMembersHandler(opts))
	appMux.HandleFunc("/admin/exports/courses", exportCoursesHandler(opts))
	appMux.HandleFunc("/admin/exports/registrations", exportRegistrationsHandler(opts))
	appMux.HandleFunc("/admin/exports/lottery-results", exportLotteryResultsHandler(opts))
	appMux.HandleFunc("/admin/exports/attendance-session", exportAttendanceSessionHandler(opts))
	appMux.HandleFunc("/admin/exports/attendance-offering", exportAttendanceOfferingHandler(opts))
	appMux.HandleFunc("/admin/imports", importsHandler(opts))
	appMux.HandleFunc("/admin/imports/members", importMembersHandler(opts))
	appMux.HandleFunc("/admin/imports/courses", importCoursesHandler(opts))
	appMux.HandleFunc("/admin/imports/members/template", memberImportTemplateHandler(opts))
	appMux.HandleFunc("/admin/imports/courses/template", courseImportTemplateHandler(opts))
	appMux.HandleFunc("/admin/backups", backupsHandler(opts))
	appMux.HandleFunc("/admin/backups/create", createBackupHandler(opts))
	appMux.HandleFunc("/admin/backups/download", downloadBackupHandler(opts))
	appMux.HandleFunc("/admin/backups/restore", restoreBackupHandler(opts))
	appMux.HandleFunc("/admin/attendance", attendanceHandler(opts))
	appMux.HandleFunc("/admin/attendance/session", createAttendanceSessionHandler(opts))
	appMux.HandleFunc("/admin/attendance/record", saveAttendanceRecordHandler(opts))
	appMux.HandleFunc("/admin/settings", settingsHandler(opts))
	appMux.HandleFunc("/admin/audit-logs", auditLogsHandler(opts))
	appMux.HandleFunc("/reception/cancel", cancelRegistrationHandler(opts))

	mux := http.NewServeMux()
	mux.Handle("/static/", http.StripPrefix("/static/", staticFileHandler()))
	mux.HandleFunc("/healthz", healthHandler())
	mux.HandleFunc("/login", loginHandler(opts))
	mux.HandleFunc("/logout", logoutHandler(opts))
	mux.Handle("/", requireAuth(opts, requirePermission(opts, appMux)))

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
