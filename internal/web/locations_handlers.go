package web

import (
	"html/template"
	"net/http"
	"strconv"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/service"
)

type locationsPageData struct {
	DisplayName     string
	Version         string
	Permissions     permissionSet
	Query           string
	IncludeInactive bool
	Error           string
	Locations       []domain.Location
}

var locationsTemplate = mustPageTemplate("locations", "locations.html", template.FuncMap{"locationStatusLabel": locationStatusLabel})

func locationsHandler(opts RouterOptions) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if opts.Locations == nil {
			http.Error(w, "location service is not configured", http.StatusServiceUnavailable)
			return
		}

		switch r.Method {
		case http.MethodGet:
			renderLocations(w, r, opts, "")
		case http.MethodPost:
			if err := r.ParseForm(); err != nil {
				http.Error(w, "invalid form", http.StatusBadRequest)
				return
			}
			input := locationInputFromRequest(r)
			if r.FormValue("action") == "update" {
				id, err := strconv.ParseInt(r.FormValue("location_id"), 10, 64)
				if err != nil {
					renderLocations(w, r, opts, "장소 선택이 올바르지 않습니다.")
					return
				}
				updated, err := opts.Locations.Update(r.Context(), id, input)
				if err != nil {
					renderLocations(w, r, opts, err.Error())
					return
				}
				recordAudit(r, opts, "location.update", "location", updated.ID, "장소 수정 #"+strconv.FormatInt(updated.ID, 10))
				http.Redirect(w, r, "/admin/locations", http.StatusSeeOther)
				return
			}
			created, err := opts.Locations.Create(r.Context(), input)
			if err != nil {
				renderLocations(w, r, opts, err.Error())
				return
			}
			recordAudit(r, opts, "location.create", "location", created.ID, "장소 등록 #"+strconv.FormatInt(created.ID, 10))
			http.Redirect(w, r, "/admin/locations", http.StatusSeeOther)
		default:
			w.Header().Set("Allow", "GET, POST")
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		}
	}
}

func locationInputFromRequest(r *http.Request) service.LocationInput {
	return service.LocationInput{
		Name:        r.FormValue("name"),
		Building:    r.FormValue("building"),
		Floor:       r.FormValue("floor"),
		Type:        domain.LocationTypeClassroom,
		IsClassroom: true,
		IsActive:    r.FormValue("is_active") == "true" || r.FormValue("action") != "update",
		Note:        r.FormValue("note"),
	}
}

func renderLocations(w http.ResponseWriter, r *http.Request, opts RouterOptions, message string) {
	query := r.URL.Query().Get("q")
	includeInactive := r.URL.Query().Get("include_inactive") == "true"
	locations, err := opts.Locations.List(r.Context(), service.LocationListInput{
		Query:           query,
		ClassroomOnly:   true,
		IncludeInactive: includeInactive,
		Limit:           500,
	})
	if err != nil {
		message = err.Error()
	}

	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	if err := locationsTemplate.ExecuteTemplate(w, "locations", locationsPageData{
		DisplayName:     opts.DisplayName,
		Version:         opts.Version,
		Permissions:     pagePermissions(r),
		Query:           query,
		IncludeInactive: includeInactive,
		Error:           message,
		Locations:       locations,
	}); err != nil {
		opts.Logger.Error("render locations failed", "error", err)
	}
}

func locationStatusLabel(location domain.Location) string {
	if location.IsActive {
		return "사용"
	}
	return "비활성"
}
