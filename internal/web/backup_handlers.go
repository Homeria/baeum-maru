package web

import (
	"fmt"
	"net/http"
	"net/url"
	"path/filepath"

	"github.com/Homeria/baeum-maru/internal/domain"
)

type backupsPageData struct {
	DisplayName string
	Version     string
	Permissions permissionSet
	Message     string
	Error       string
	Status      domain.BackupStatus
	Backups     []domain.BackupFile
}

var backupsTemplate = mustPageTemplate("backups", "backups.html", nil)

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
	if err := backupsTemplate.ExecuteTemplate(w, "backups", backupsPageData{
		DisplayName: opts.DisplayName,
		Version:     opts.Version,
		Permissions: pagePermissions(r),
		Message:     message,
		Error:       errorMessage,
		Status:      status,
		Backups:     backups,
	}); err != nil {
		opts.Logger.Error("render backups failed", "error", err)
	}
}
