package web

import (
	"fmt"
	"net/http"
	"path/filepath"
	"strconv"

	"github.com/Homeria/baeum-maru/internal/service"
)

type exportsPageData struct {
	DisplayName string
	Version     string
	Permissions permissionSet
	Error       string
}

var exportsTemplate = mustPageTemplate("exports", "exports.html", nil)

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
		renderExports(w, r, opts, "")
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

func renderExports(w http.ResponseWriter, r *http.Request, opts RouterOptions, message string) {
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	if err := exportsTemplate.ExecuteTemplate(w, "exports", exportsPageData{
		DisplayName: opts.DisplayName,
		Version:     opts.Version,
		Permissions: pagePermissions(r),
		Error:       message,
	}); err != nil {
		opts.Logger.Error("render exports failed", "error", err)
	}
}
