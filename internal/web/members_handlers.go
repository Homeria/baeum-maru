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
	Gender      string
	Sort        string
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
			input := service.MemberInput{
				MemberNo:   r.FormValue("member_no"),
				Name:       r.FormValue("name"),
				GenderCode: r.FormValue("gender_code"),
				BirthDate:  r.FormValue("birth_date"),
				Phone:      r.FormValue("phone"),
				Note:       r.FormValue("note"),
			}
			if r.FormValue("action") == "update" {
				id, err := strconv.ParseInt(r.FormValue("member_id"), 10, 64)
				if err != nil {
					renderMembers(w, r, opts, "회원 선택이 올바르지 않습니다.")
					return
				}
				updated, err := opts.Members.Update(r.Context(), id, input)
				if err != nil {
					renderMembers(w, r, opts, err.Error())
					return
				}
				recordAudit(r, opts, "member.update", "member", updated.ID, "회원 수정 #"+strconv.FormatInt(updated.ID, 10))
				http.Redirect(w, r, "/admin/members", http.StatusSeeOther)
				return
			}
			created, err := opts.Members.Create(r.Context(), service.MemberInput{
				MemberNo:   input.MemberNo,
				Name:       input.Name,
				GenderCode: input.GenderCode,
				BirthDate:  input.BirthDate,
				Phone:      input.Phone,
				Note:       input.Note,
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
	gender := r.URL.Query().Get("gender")
	sortKey := r.URL.Query().Get("sort")
	members, err := opts.Members.Search(r.Context(), query, 500)
	if err != nil {
		message = err.Error()
	}
	members = filterMembers(members, gender)
	sortMembers(members, sortKey)

	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	if err := membersTemplate.ExecuteTemplate(w, "members", membersPageData{
		DisplayName: opts.DisplayName,
		Version:     opts.Version,
		Permissions: pagePermissions(r),
		Query:       query,
		Gender:      gender,
		Sort:        sortKey,
		Error:       message,
		Members:     members,
	}); err != nil {
		opts.Logger.Error("render members failed", "error", err)
	}
}
