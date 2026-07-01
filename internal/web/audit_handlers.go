package web

import (
	"net/http"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/service"
)

type auditLogsPageData struct {
	DisplayName string
	Version     string
	Error       string
	Logs        []domain.AuditLog
}

var auditLogsTemplate = templateMust("auditLogs", `<!doctype html>
<html lang="ko">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>감사 로그 - {{.DisplayName}}</title>
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
        <h1>감사 로그</h1>
      </div>
    </section>
    {{if .Error}}<p class="alert error" role="alert">{{.Error}}</p>{{end}}
    <section class="panel">
      <div class="table-wrap">
        <table>
          <thead><tr><th>ID</th><th>시간</th><th>작업</th><th>대상</th><th>대상 ID</th><th>요약</th></tr></thead>
          <tbody>
            {{range .Logs}}
              <tr>
                <td>{{.ID}}</td>
                <td>{{.CreatedAt}}</td>
                <td>{{.Action}}</td>
                <td>{{.EntityType}}</td>
                <td>{{if .EntityID}}{{.EntityID}}{{else}}-{{end}}</td>
                <td>{{.Summary}}</td>
              </tr>
            {{else}}
              <tr><td class="empty" colspan="6">감사 로그가 없습니다.</td></tr>
            {{end}}
          </tbody>
        </table>
      </div>
    </section>
    <footer class="footer">{{.DisplayName}} {{.Version}}</footer>
  </main>
</body>
</html>
`)

func auditLogsHandler(opts RouterOptions) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/admin/audit-logs" {
			http.NotFound(w, r)
			return
		}
		if opts.Audits == nil {
			http.Error(w, "audit service is not configured", http.StatusServiceUnavailable)
			return
		}
		if r.Method != http.MethodGet {
			w.Header().Set("Allow", http.MethodGet)
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}

		logs, err := opts.Audits.ListRecent(r.Context(), 200)
		errorMessage := ""
		if err != nil {
			errorMessage = err.Error()
		}

		w.Header().Set("Content-Type", "text/html; charset=utf-8")
		if err := auditLogsTemplate.Execute(w, auditLogsPageData{
			DisplayName: opts.DisplayName,
			Version:     opts.Version,
			Error:       errorMessage,
			Logs:        logs,
		}); err != nil {
			opts.Logger.Error("render audit logs failed", "error", err)
		}
	}
}

func recordAudit(r *http.Request, opts RouterOptions, action string, entityType string, entityID int64, summary string) {
	if opts.Audits == nil {
		return
	}
	if err := opts.Audits.Record(r.Context(), service.AuditEvent{
		Action:     action,
		EntityType: entityType,
		EntityID:   entityID,
		Summary:    summary,
	}); err != nil {
		opts.Logger.Warn("record audit failed", "action", action, "entity_type", entityType, "entity_id", entityID, "error", err)
	}
}
