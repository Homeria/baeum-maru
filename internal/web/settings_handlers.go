package web

import (
	"net/http"
	"net/url"
	"strconv"

	"github.com/Homeria/baeum-maru/internal/config"
	"github.com/Homeria/baeum-maru/internal/service"
)

type settingsPageData struct {
	DisplayName string
	Version     string
	Message     string
	Error       string
	Config      config.Config
}

var settingsTemplate = templateMust("settings", `<!doctype html>
<html lang="ko">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>설정 - {{.DisplayName}}</title>
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
        <h1>설정</h1>
      </div>
    </section>
    {{if .Message}}<p class="alert success" role="status">{{.Message}}</p>{{end}}
    {{if .Error}}<p class="alert error" role="alert">{{.Error}}</p>{{end}}

    <form class="panel" method="post" action="/admin/settings">
      <h2>운영 설정</h2>
      <div class="form-grid">
        <label>백업 보관 일수 <input name="backup_keep_days" type="number" min="0" value="{{.Config.Backup.KeepDays}}"></label>
        <label style="display:flex;align-items:center;gap:8px;">
          <input name="open_browser_on_start" type="checkbox" value="true" {{if .Config.UI.OpenBrowserOnStart}}checked{{end}} style="width:auto;min-height:auto;">
          브라우저 자동 열기
        </label>
        <button type="submit">저장</button>
      </div>
    </form>

    <section class="panel">
      <h2>현재 설정</h2>
      <div class="table-wrap">
        <table>
          <tbody>
            <tr><th>앱 이름</th><td>{{.Config.App.DisplayName}}</td></tr>
            <tr><th>실행 모드</th><td>{{.Config.App.Mode}}</td></tr>
            <tr><th>서버 주소</th><td>{{.Config.Server.Host}}:{{.Config.Server.Port}}</td></tr>
            <tr><th>DB 경로</th><td>{{.Config.Database.Path}}</td></tr>
            <tr><th>백업 경로</th><td>{{.Config.Backup.Path}}</td></tr>
            <tr><th>엑셀 출력 경로</th><td>{{.Config.Export.Path}}</td></tr>
            <tr><th>로그 경로</th><td>{{.Config.Logging.Path}}</td></tr>
            <tr><th>로그 레벨</th><td>{{.Config.Logging.Level}}</td></tr>
          </tbody>
        </table>
      </div>
    </section>
    <footer class="footer">{{.DisplayName}} {{.Version}}</footer>
  </main>
</body>
</html>
`)

func settingsHandler(opts RouterOptions) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/admin/settings" {
			http.NotFound(w, r)
			return
		}
		if opts.Settings == nil {
			http.Error(w, "settings service is not configured", http.StatusServiceUnavailable)
			return
		}
		switch r.Method {
		case http.MethodGet:
			renderSettings(w, r, opts, r.URL.Query().Get("message"), "")
		case http.MethodPost:
			if err := r.ParseForm(); err != nil {
				http.Error(w, "invalid form", http.StatusBadRequest)
				return
			}
			keepDays, err := strconv.Atoi(r.FormValue("backup_keep_days"))
			if err != nil {
				renderSettings(w, r, opts, "", "백업 보관 일수는 숫자로 입력해야 합니다.")
				return
			}
			_, err = opts.Settings.Update(r.Context(), service.SettingsInput{
				BackupKeepDays:     keepDays,
				OpenBrowserOnStart: r.FormValue("open_browser_on_start") == "true",
			})
			if err != nil {
				renderSettings(w, r, opts, "", err.Error())
				return
			}
			http.Redirect(w, r, "/admin/settings?message="+url.QueryEscape("설정을 저장했습니다. 일부 항목은 앱 재시작 후 적용됩니다."), http.StatusSeeOther)
		default:
			w.Header().Set("Allow", "GET, POST")
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		}
	}
}

func renderSettings(w http.ResponseWriter, r *http.Request, opts RouterOptions, message string, errorMessage string) {
	cfg, err := opts.Settings.Get(r.Context())
	if err != nil {
		errorMessage = err.Error()
	}
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	if err := settingsTemplate.Execute(w, settingsPageData{
		DisplayName: opts.DisplayName,
		Version:     opts.Version,
		Message:     message,
		Error:       errorMessage,
		Config:      cfg,
	}); err != nil {
		opts.Logger.Error("render settings failed", "error", err)
	}
}
