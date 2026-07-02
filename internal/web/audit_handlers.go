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

var auditLogsTemplate = mustPageTemplate("audit_logs", "audit_logs.html", nil)

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
		if err := auditLogsTemplate.ExecuteTemplate(w, "audit_logs", auditLogsPageData{
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
	var actorUserID int64
	if identity, ok := currentAuthIdentity(r); ok {
		actorUserID = identity.UserID
	}
	if err := opts.Audits.Record(r.Context(), service.AuditEvent{
		ActorUserID: actorUserID,
		Action:      action,
		EntityType:  entityType,
		EntityID:    entityID,
		Summary:     summary,
	}); err != nil {
		opts.Logger.Warn("record audit failed", "action", action, "entity_type", entityType, "entity_id", entityID, "error", err)
	}
}
