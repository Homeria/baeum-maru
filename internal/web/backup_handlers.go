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
	Status      domain.BackupStatus
	Backups     []domain.BackupFile
}

var backupsTemplate = template.Must(template.New("backups").Funcs(uiTemplateFuncs(nil)).Parse(`<!doctype html>
<html lang="ko">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>백업 - {{.DisplayName}}</title>
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
      <a href="/admin/audit-logs">감사 로그</a>
      <a href="/reception">접수 화면</a>
    </nav>
  </header>
  <main class="page">
    <section class="page-header">
      <div>
        <h1>백업</h1>
      </div>
      <form class="inline-form" method="post" action="/admin/backups/create">
        <button type="submit">백업 생성</button>
      </form>
    </section>
    {{if .Message}}<p class="alert success" role="status">{{.Message}}</p>{{end}}
    {{if .Error}}<p class="alert error" role="alert">{{.Error}}</p>{{end}}
    <section class="panel">
      <h2>백업 상태</h2>
      <div class="actions">
        {{if .Status.Latest}}
          <span class="badge completed">최근 백업 {{.Status.Latest.FileName}}</span>
          <span class="badge">{{humanBytes .Status.Latest.SizeBytes}}</span>
          <span class="badge">{{.Status.Latest.CreatedAt}}</span>
        {{else}}
          <span class="badge failed">백업 없음</span>
        {{end}}
        <span class="badge">전체 {{.Status.TotalCount}}개</span>
        <span class="badge">총 {{humanBytes .Status.TotalBytes}}</span>
        {{if .Status.RetentionOn}}
          <span class="badge">보관 {{.Status.KeepDays}}일</span>
        {{else}}
          <span class="badge">자동 정리 꺼짐</span>
        {{end}}
      </div>
    </section>
    <section class="panel">
      <div class="table-wrap">
        <table>
          <thead>
            <tr><th>파일명</th><th>크기</th><th>생성일</th><th>작업</th></tr>
          </thead>
          <tbody>
            {{range .Backups}}
              <tr>
                <td>{{.FileName}}</td>
                <td>{{humanBytes .SizeBytes}}</td>
                <td>{{.CreatedAt}}</td>
                <td>
                  <div class="actions">
                    <a class="button secondary" href="/admin/backups/download?file={{urlquery .FileName}}">다운로드</a>
                    <form class="inline-form" method="post" action="/admin/backups/restore">
                      <input type="hidden" name="file" value="{{.FileName}}">
                      <button class="danger" type="submit">복원 예약</button>
                    </form>
                  </div>
                </td>
              </tr>
            {{else}}
              <tr><td class="empty" colspan="4">백업 파일이 없습니다.</td></tr>
            {{end}}
          </tbody>
        </table>
      </div>
    </section>
    <footer class="footer">{{.DisplayName}} {{.Version}}</footer>
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
		cleanup, err := opts.Backups.PruneOldBackups(r.Context())
		if err != nil {
			renderBackups(w, r, opts, message, err.Error())
			return
		}
		if cleanup.DeletedCount > 0 {
			message += " / 오래된 백업 " + fmt.Sprint(cleanup.DeletedCount) + "개 정리"
		}
		recordAudit(r, opts, "backup.create", "backup", 0, "백업 생성: "+created.FileName)
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
		recordAudit(r, opts, "backup.restore_queue", "backup", 0, "백업 복원 예약: "+plan.FileName)
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
	status, err := opts.Backups.Status(r.Context())
	if err != nil {
		errorMessage = err.Error()
	}

	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	if err := backupsTemplate.Execute(w, backupsPageData{
		DisplayName: opts.DisplayName,
		Version:     opts.Version,
		Message:     message,
		Error:       errorMessage,
		Status:      status,
		Backups:     backups,
	}); err != nil {
		opts.Logger.Error("render backups failed", "error", err)
	}
}
