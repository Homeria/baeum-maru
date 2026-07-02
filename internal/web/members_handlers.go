package web

import (
	"net/http"
	"strconv"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/service"
)

type membersPageData struct {
	DisplayName string
	Version     string
	Permissions permissionSet
	Query       string
	Error       string
	Members     []domain.Member
}

var membersTemplate = mustPageTemplate("members", "members.html", nil)

func membersHandler(opts RouterOptions) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if opts.Members == nil {
			http.Error(w, "member service is not configured", http.StatusServiceUnavailable)
			return
		}

		switch r.Method {
		case http.MethodGet:
			renderMembers(w, r, opts, "")
		case http.MethodPost:
			if err := r.ParseForm(); err != nil {
				http.Error(w, "invalid form", http.StatusBadRequest)
				return
			}
			created, err := opts.Members.Create(r.Context(), service.MemberInput{
				MemberNo:   r.FormValue("member_no"),
				Name:       r.FormValue("name"),
				GenderCode: r.FormValue("gender_code"),
				BirthDate:  r.FormValue("birth_date"),
				Phone:      r.FormValue("phone"),
				Note:       r.FormValue("note"),
			})
			if err != nil {
				renderMembers(w, r, opts, err.Error())
				return
			}
			recordAudit(r, opts, "member.create", "member", created.ID, "회원 등록 #"+strconv.FormatInt(created.ID, 10))
			http.Redirect(w, r, "/admin/members", http.StatusSeeOther)
		default:
			w.Header().Set("Allow", "GET, POST")
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		}
	}
}

func renderMembers(w http.ResponseWriter, r *http.Request, opts RouterOptions, message string) {
	query := r.URL.Query().Get("q")
	members, err := opts.Members.Search(r.Context(), query, 50)
	if err != nil {
		message = err.Error()
	}

	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	if err := membersTemplate.ExecuteTemplate(w, "members", membersPageData{
		DisplayName: opts.DisplayName,
		Version:     opts.Version,
		Permissions: pagePermissions(r),
		Query:       query,
		Error:       message,
		Members:     members,
	}); err != nil {
		opts.Logger.Error("render members failed", "error", err)
	}
}
