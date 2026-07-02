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
	mux.HandleFunc("/healthz", healthHandler())

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
