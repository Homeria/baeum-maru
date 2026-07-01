package web

import (
	"fmt"
	"html/template"
	"net/http"
	"net/url"
	"path/filepath"

	"github.com/Homeria/baeum-maru/internal/domain"
)

type backupsPageData struct {
	DisplayName string
	Version     string
	Message     string
	Error       string
	Backups     []domain.BackupFile
}

var backupsTemplate = template.Must(template.New("backups").Parse(`<!doctype html>
<html lang="ko">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>백업 - {{.DisplayName}}</title>
</head>
<body>
  <main>
    <nav><a href="/admin">관리</a> <a href="/admin/registrations">신청 현황</a> <a href="/admin/exports">엑셀 내보내기</a></nav>
    <h1>백업</h1>
    {{if .Message}}<p role="status">{{.Message}}</p>{{end}}
    {{if .Error}}<p role="alert">{{.Error}}</p>{{end}}
    <form method="post" action="/admin/backups/create">
      <button type="submit">백업 생성</button>
    </form>
    <table>
      <thead>
        <tr><th>파일명</th><th>크기</th><th>생성일</th><th>작업</th></tr>
      </thead>
      <tbody>
        {{range .Backups}}
          <tr>
            <td>{{.FileName}}</td>
            <td>{{.SizeBytes}}</td>
            <td>{{.CreatedAt}}</td>
            <td>
              <a href="/admin/backups/download?file={{urlquery .FileName}}">다운로드</a>
              <form method="post" action="/admin/backups/restore">
                <input type="hidden" name="file" value="{{.FileName}}">
                <button type="submit">복원 예약</button>
              </form>
            </td>
          </tr>
        {{else}}
          <tr><td colspan="4">백업 파일이 없습니다.</td></tr>
        {{end}}
      </tbody>
    </table>
    <small>{{.DisplayName}} {{.Version}}</small>
  </main>
</body>
</html>
`))

func backupsHandler(opts RouterOptions) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/admin/backups" {
			http.NotFound(w, r)
			return
		}
		if r.Method != http.MethodGet {
			w.Header().Set("Allow", http.MethodGet)
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		renderBackups(w, r, opts, r.URL.Query().Get("message"), "")
	}
}

func createBackupHandler(opts RouterOptions) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if opts.Backups == nil {
			http.Error(w, "backup service is not configured", http.StatusServiceUnavailable)
			return
		}
		if r.Method != http.MethodPost {
			w.Header().Set("Allow", http.MethodPost)
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		created, err := opts.Backups.CreateBackup(r.Context())
		if err != nil {
			renderBackups(w, r, opts, "", err.Error())
			return
		}
		message := "백업을 생성했습니다: " + created.FileName
		http.Redirect(w, r, "/admin/backups?message="+url.QueryEscape(message), http.StatusSeeOther)
	}
}

func downloadBackupHandler(opts RouterOptions) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if opts.Backups == nil {
			http.Error(w, "backup service is not configured", http.StatusServiceUnavailable)
			return
		}
		if r.Method != http.MethodGet {
			w.Header().Set("Allow", http.MethodGet)
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		fileName := r.URL.Query().Get("file")
		path, err := opts.Backups.ResolveBackupPath(fileName)
		if err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
		w.Header().Set("Content-Disposition", fmt.Sprintf("attachment; filename=%q", filepath.Base(fileName)))
		w.Header().Set("Content-Type", "application/octet-stream")
		http.ServeFile(w, r, path)
	}
}

func restoreBackupHandler(opts RouterOptions) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if opts.Backups == nil {
			http.Error(w, "backup service is not configured", http.StatusServiceUnavailable)
			return
		}
		if r.Method != http.MethodPost {
			w.Header().Set("Allow", http.MethodPost)
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		if err := r.ParseForm(); err != nil {
			http.Error(w, "invalid form", http.StatusBadRequest)
			return
		}
		plan, err := opts.Backups.QueueRestore(r.Context(), r.FormValue("file"))
		if err != nil {
			renderBackups(w, r, opts, "", err.Error())
			return
		}
		message := "복원을 예약했습니다: " + plan.FileName + " / 앱 재시작 후 적용"
		http.Redirect(w, r, "/admin/backups?message="+url.QueryEscape(message), http.StatusSeeOther)
	}
}

func renderBackups(w http.ResponseWriter, r *http.Request, opts RouterOptions, message string, errorMessage string) {
	if opts.Backups == nil {
		http.Error(w, "backup service is not configured", http.StatusServiceUnavailable)
		return
	}
	backups, err := opts.Backups.ListBackups(r.Context())
	if err != nil {
		errorMessage = err.Error()
	}

	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	if err := backupsTemplate.Execute(w, backupsPageData{
		DisplayName: opts.DisplayName,
		Version:     opts.Version,
		Message:     message,
		Error:       errorMessage,
		Backups:     backups,
	}); err != nil {
		opts.Logger.Error("render backups failed", "error", err)
	}
}
