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

var settingsTemplate = mustPageTemplate("settings", "settings.html", nil)

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
			recordAudit(r, opts, "settings.update", "settings", 0, "운영 설정 변경")
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
	if err := settingsTemplate.ExecuteTemplate(w, "settings", settingsPageData{
		DisplayName: opts.DisplayName,
		Version:     opts.Version,
		Message:     message,
		Error:       errorMessage,
		Config:      cfg,
	}); err != nil {
		opts.Logger.Error("render settings failed", "error", err)
	}
}
