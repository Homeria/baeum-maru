package web

import (
	"fmt"
	"html/template"
	"net/http"
	"path/filepath"
	"strconv"

	"github.com/Homeria/baeum-maru/internal/service"
)

type exportsPageData struct {
	DisplayName string
	Version     string
	Error       string
}

var exportsTemplate = template.Must(template.New("exports").Funcs(uiTemplateFuncs(nil)).Parse(`<!doctype html>
<html lang="ko">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>엑셀 내보내기 - {{.DisplayName}}</title>
  <style>{{appStyles}}</style>
</head>
<body>
  <header class="topbar">
    <a class="brand" href="/admin">{{.DisplayName}}</a>
    <nav class="topnav">
      <a href="/admin/members">회원 관리</a>
      <a href="/admin/courses">강좌 관리</a>
      <a href="/admin/registrations">신청 현황</a>
      <a href="/admin/lottery">추첨</a>
      <a href="/admin/imports">엑셀 가져오기</a>
      <a href="/admin/exports">엑셀 내보내기</a>
      <a href="/admin/backups">백업</a>
      <a href="/admin/attendance">출석</a>
      <a href="/admin/settings">설정</a>
      <a href="/reception">접수 화면</a>
    </nav>
  </header>
  <main class="page">
    <section class="page-header">
      <div>
        <h1>엑셀 내보내기</h1>
      </div>
    </section>
    {{if .Error}}<p class="alert error" role="alert">{{.Error}}</p>{{end}}
    <section class="panel">
      <div class="actions">
        <a class="button" href="/admin/exports/members">회원 목록 다운로드</a>
        <a class="button" href="/admin/exports/courses">강좌 개설 목록 다운로드</a>
        <a class="button" href="/admin/exports/registrations">신청 현황 다운로드</a>
      </div>
    </section>
    <footer class="footer">{{.DisplayName}} {{.Version}}</footer>
  </main>
</body>
</html>
`))

func exportsHandler(opts RouterOptions) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/admin/exports" {
			http.NotFound(w, r)
			return
		}
		if r.Method != http.MethodGet {
			w.Header().Set("Allow", http.MethodGet)
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		renderExports(w, opts, "")
	}
}

func exportMembersHandler(opts RouterOptions) http.HandlerFunc {
	return exportHandler(opts, func(exports ExportService, r *http.Request) (service.ExportResult, error) {
		return exports.ExportMembers(r.Context())
	})
}

func exportCoursesHandler(opts RouterOptions) http.HandlerFunc {
	return exportHandler(opts, func(exports ExportService, r *http.Request) (service.ExportResult, error) {
		return exports.ExportCourseOfferings(r.Context())
	})
}

func exportRegistrationsHandler(opts RouterOptions) http.HandlerFunc {
	return exportHandler(opts, func(exports ExportService, r *http.Request) (service.ExportResult, error) {
		return exports.ExportRegistrations(r.Context())
	})
}

func exportLotteryResultsHandler(opts RouterOptions) http.HandlerFunc {
	return exportHandler(opts, func(exports ExportService, r *http.Request) (service.ExportResult, error) {
		runID, err := strconv.ParseInt(r.URL.Query().Get("run_id"), 10, 64)
		if err != nil {
			return service.ExportResult{}, err
		}
		return exports.ExportLotteryResults(r.Context(), runID)
	})
}

func exportAttendanceSessionHandler(opts RouterOptions) http.HandlerFunc {
	return exportHandler(opts, func(exports ExportService, r *http.Request) (service.ExportResult, error) {
		sessionID, err := strconv.ParseInt(r.URL.Query().Get("session_id"), 10, 64)
		if err != nil {
			return service.ExportResult{}, err
		}
		return exports.ExportAttendanceSession(r.Context(), sessionID)
	})
}

func exportAttendanceOfferingHandler(opts RouterOptions) http.HandlerFunc {
	return exportHandler(opts, func(exports ExportService, r *http.Request) (service.ExportResult, error) {
		offeringID, err := strconv.ParseInt(r.URL.Query().Get("offering_id"), 10, 64)
		if err != nil {
			return service.ExportResult{}, err
		}
		return exports.ExportAttendanceOffering(r.Context(), offeringID)
	})
}

func exportHandler(opts RouterOptions, create func(ExportService, *http.Request) (service.ExportResult, error)) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodGet {
			w.Header().Set("Allow", http.MethodGet)
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		if opts.Exports == nil {
			http.Error(w, "export service is not configured", http.StatusServiceUnavailable)
			return
		}

		result, err := create(opts.Exports, r)
		if err != nil {
			opts.Logger.Error("create export failed", "path", r.URL.Path, "error", err)
			http.Error(w, "failed to create export", http.StatusInternalServerError)
			return
		}

		w.Header().Set("Content-Disposition", fmt.Sprintf("attachment; filename=%q", filepath.Base(result.FileName)))
		w.Header().Set("Content-Type", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
		http.ServeFile(w, r, result.Path)
	}
}

func renderExports(w http.ResponseWriter, opts RouterOptions, message string) {
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	if err := exportsTemplate.Execute(w, exportsPageData{
		DisplayName: opts.DisplayName,
		Version:     opts.Version,
		Error:       message,
	}); err != nil {
		opts.Logger.Error("render exports failed", "error", err)
	}
}
