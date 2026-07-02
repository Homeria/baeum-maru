package web

import (
	"html/template"
	"net/http"
	"net/url"
	"strconv"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/service"
)

type receptionPageData struct {
	DisplayName   string
	Version       string
	Query         string
	SelectedID    int64
	Error         string
	Members       []domain.Member
	Offerings     []domain.CourseOffering
	Registrations []domain.Registration
}

var receptionTemplate = mustPageTemplate("reception", "reception.html", template.FuncMap{"weekdayLabel": weekdayLabel})

func receptionHandler(opts RouterOptions) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if opts.Members == nil || opts.Courses == nil || opts.Registrations == nil {
			if r.Method == http.MethodGet {
				exactPath("/reception", renderPage(opts, pageData{
					Title:       "접수",
					Heading:     "접수 화면",
					Description: "회원 검색과 수강신청 입력을 진행하는 화면입니다.",
				}))(w, r)
				return
			}
			http.Error(w, "registration services are not configured", http.StatusServiceUnavailable)
			return
		}

		switch r.Method {
		case http.MethodGet:
			renderReception(w, r, opts, "")
		case http.MethodPost:
			if err := r.ParseForm(); err != nil {
				http.Error(w, "invalid form", http.StatusBadRequest)
				return
			}
			memberID, err := strconv.ParseInt(r.FormValue("member_id"), 10, 64)
			if err != nil {
				renderReception(w, r, opts, "회원 선택이 올바르지 않습니다.")
				return
			}
			offeringID, err := strconv.ParseInt(r.FormValue("offering_id"), 10, 64)
			if err != nil {
				renderReception(w, r, opts, "강좌 선택이 올바르지 않습니다.")
				return
			}
			created, err := opts.Registrations.Create(r.Context(), service.RegistrationInput{
				MemberID:   memberID,
				OfferingID: offeringID,
			})
			if err != nil {
				renderReception(w, r, opts, err.Error())
				return
			}
			recordAudit(r, opts, "registration.create", "registration", created.ID, "수강신청 등록 #"+strconv.FormatInt(created.ID, 10))
			http.Redirect(w, r, "/reception?member_id="+strconv.FormatInt(memberID, 10), http.StatusSeeOther)
		default:
			w.Header().Set("Allow", "GET, POST")
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		}
	}
}

func renderReception(w http.ResponseWriter, r *http.Request, opts RouterOptions, message string) {
	query := r.URL.Query().Get("q")
	selectedID, _ := strconv.ParseInt(r.URL.Query().Get("member_id"), 10, 64)
	members, err := opts.Members.Search(r.Context(), query, 30)
	if err != nil {
		message = err.Error()
	}
	offerings, err := opts.Courses.ListOfferings(r.Context(), 200)
	if err != nil {
		message = err.Error()
	}
	var registrations []domain.Registration
	if selectedID > 0 {
		registrations, err = opts.Registrations.ListByMember(r.Context(), selectedID)
		if err != nil {
			message = err.Error()
		}
	}

	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	if err := receptionTemplate.ExecuteTemplate(w, "reception", receptionPageData{
		DisplayName:   opts.DisplayName,
		Version:       opts.Version,
		Query:         query,
		SelectedID:    selectedID,
		Error:         message,
		Members:       members,
		Offerings:     offerings,
		Registrations: registrations,
	}); err != nil {
		opts.Logger.Error("render reception failed", "error", err)
	}
}

func cancelRegistrationHandler(opts RouterOptions) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if opts.Registrations == nil {
			http.Error(w, "registration service is not configured", http.StatusServiceUnavailable)
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
		registrationID, err := strconv.ParseInt(r.FormValue("registration_id"), 10, 64)
		if err != nil {
			http.Error(w, "invalid registration id", http.StatusBadRequest)
			return
		}
		cancelled, err := opts.Registrations.Cancel(r.Context(), registrationID)
		if err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
		recordAudit(r, opts, "registration.cancel", "registration", cancelled.ID, "수강신청 취소 #"+strconv.FormatInt(cancelled.ID, 10))
		redirect := "/reception?member_id=" + r.FormValue("member_id")
		if q := r.FormValue("q"); q != "" {
			redirect += "&q=" + url.QueryEscape(q)
		}
		http.Redirect(w, r, redirect, http.StatusSeeOther)
	}
}

type registrationsPageData struct {
	DisplayName   string
	Version       string
	Message       string
	Error         string
	Registrations []domain.Registration
}

var registrationsTemplate = mustPageTemplate("registrations", "registrations.html", nil)

func registrationsHandler(opts RouterOptions) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if opts.Registrations == nil {
			http.Error(w, "registration service is not configured", http.StatusServiceUnavailable)
			return
		}
		if r.Method != http.MethodGet {
			w.Header().Set("Allow", http.MethodGet)
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}

		registrations, err := opts.Registrations.ListRecent(r.Context(), 200)
		message := ""
		if err != nil {
			message = err.Error()
		}

		w.Header().Set("Content-Type", "text/html; charset=utf-8")
		if err := registrationsTemplate.ExecuteTemplate(w, "registrations", registrationsPageData{
			DisplayName:   opts.DisplayName,
			Version:       opts.Version,
			Message:       r.URL.Query().Get("message"),
			Error:         message,
			Registrations: registrations,
		}); err != nil {
			opts.Logger.Error("render registrations failed", "error", err)
		}
	}
}

func registrationStatusHandler(opts RouterOptions) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if opts.Registrations == nil {
			http.Error(w, "registration service is not configured", http.StatusServiceUnavailable)
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
		registrationID, err := strconv.ParseInt(r.FormValue("registration_id"), 10, 64)
		if err != nil {
			http.Error(w, "invalid registration id", http.StatusBadRequest)
			return
		}

		switch r.FormValue("action") {
		case "confirm":
			confirmed, err := opts.Registrations.Confirm(r.Context(), registrationID)
			if err != nil {
				http.Error(w, err.Error(), http.StatusBadRequest)
				return
			}
			recordAudit(r, opts, "registration.confirm", "registration", confirmed.ID, "수강신청 확정 #"+strconv.FormatInt(confirmed.ID, 10))
			http.Redirect(w, r, "/admin/registrations?message="+url.QueryEscape("신청을 확정했습니다."), http.StatusSeeOther)
		case "cancel":
			change, err := opts.Registrations.CancelWithPromotion(r.Context(), registrationID)
			if err != nil {
				http.Error(w, err.Error(), http.StatusBadRequest)
				return
			}
			message := "신청을 취소했습니다."
			if change.Promoted != nil {
				message = "신청을 취소하고 대기자를 선정으로 승격했습니다."
			}
			summary := "수강신청 취소 #" + strconv.FormatInt(change.Registration.ID, 10)
			if change.Promoted != nil {
				summary += ", 대기자 승격 #" + strconv.FormatInt(change.Promoted.ID, 10)
			}
			recordAudit(r, opts, "registration.cancel", "registration", change.Registration.ID, summary)
			http.Redirect(w, r, "/admin/registrations?message="+url.QueryEscape(message), http.StatusSeeOther)
		default:
			http.Error(w, "unsupported registration action", http.StatusBadRequest)
		}
	}
}
