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

var receptionTemplate = template.Must(template.New("reception").Funcs(uiTemplateFuncs(template.FuncMap{
	"weekdayLabel": weekdayLabel,
})).Parse(`<!doctype html>
<html lang="ko">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>접수 - {{.DisplayName}}</title>
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
        <h1>접수 화면</h1>
      </div>
      <a class="button secondary" href="/admin/registrations">신청 현황</a>
    </section>
    {{if .Error}}<p class="alert error" role="alert">{{.Error}}</p>{{end}}

    <section class="panel">
      <h2>회원 검색</h2>
      <form class="form-grid" method="get" action="/reception">
        <label>검색 <input name="q" value="{{.Query}}" placeholder="이름, 회원번호, 연락처"></label>
        <button type="submit">검색</button>
      </form>
      <div class="table-wrap" style="margin-top: 14px;">
        <table>
          <thead><tr><th>ID</th><th>회원번호</th><th>이름</th><th>연락처</th><th>선택</th></tr></thead>
          <tbody>
            {{range .Members}}
              <tr>
                <td>{{.ID}}</td>
                <td>{{.MemberNo}}</td>
                <td>{{.Name}}</td>
                <td>{{.Phone}}</td>
                <td><a class="button secondary" href="/reception?q={{$.Query}}&member_id={{.ID}}">선택</a></td>
              </tr>
            {{else}}
              <tr><td class="empty" colspan="5">검색 결과가 없습니다.</td></tr>
            {{end}}
          </tbody>
        </table>
      </div>
    </section>

    <section class="panel">
      <h2>신청 입력</h2>
      {{if .SelectedID}}
        <form class="form-grid" method="post" action="/reception">
          <input type="hidden" name="member_id" value="{{.SelectedID}}">
          <label>강좌
            <select name="offering_id" required>
              {{range .Offerings}}
                <option value="{{.ID}}">{{.TermName}} / {{.CourseTitle}} / {{weekdayLabel .Weekday}} {{.StartTime}}-{{.EndTime}} / 정원 {{.Capacity}}</option>
              {{end}}
            </select>
          </label>
          <button type="submit">신청 저장</button>
        </form>
      {{else}}
        <p class="subtle">회원을 먼저 선택하세요.</p>
      {{end}}
    </section>

    <section class="panel">
      <h2>선택 회원 신청 목록</h2>
      <div class="table-wrap">
        <table>
          <thead><tr><th>ID</th><th>회차</th><th>강좌</th><th>상태</th><th>신청일</th><th>작업</th></tr></thead>
          <tbody>
            {{range .Registrations}}
              <tr>
                <td>{{.ID}}</td>
                <td>{{.TermName}}</td>
                <td>{{.CourseTitle}}</td>
                <td><span class="badge {{statusClass .Status}}">{{statusLabel .Status}}</span></td>
                <td>{{.CreatedAt}}</td>
                <td>
                  {{if ne .Status "cancelled"}}
                    <form class="inline-form" method="post" action="/reception/cancel">
                      <input type="hidden" name="registration_id" value="{{.ID}}">
                      <input type="hidden" name="member_id" value="{{$.SelectedID}}">
                      <input type="hidden" name="q" value="{{$.Query}}">
                      <button class="danger" type="submit">취소</button>
                    </form>
                  {{end}}
                </td>
              </tr>
            {{else}}
              <tr><td class="empty" colspan="6">신청 내역이 없습니다.</td></tr>
            {{end}}
          </tbody>
        </table>
      </div>
    </section>
    <footer class="footer">{{.DisplayName}} {{.Version}}</footer>
  </main>
</body>
</html>
`))

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
			_, err = opts.Registrations.Create(r.Context(), service.RegistrationInput{
				MemberID:   memberID,
				OfferingID: offeringID,
			})
			if err != nil {
				renderReception(w, r, opts, err.Error())
				return
			}
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
	if err := receptionTemplate.Execute(w, receptionPageData{
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
		if _, err := opts.Registrations.Cancel(r.Context(), registrationID); err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
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

var registrationsTemplate = template.Must(template.New("registrations").Funcs(uiTemplateFuncs(nil)).Parse(`<!doctype html>
<html lang="ko">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>신청 현황 - {{.DisplayName}}</title>
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
        <h1>신청 현황</h1>
      </div>
      <a class="button secondary" href="/admin/exports/registrations">엑셀 다운로드</a>
    </section>
    {{if .Message}}<p class="alert success" role="status">{{.Message}}</p>{{end}}
    {{if .Error}}<p class="alert error" role="alert">{{.Error}}</p>{{end}}
    <section class="panel">
      <div class="table-wrap">
        <table>
          <thead><tr><th>ID</th><th>회원</th><th>회원번호</th><th>회차</th><th>강좌</th><th>상태</th><th>신청일</th><th>작업</th></tr></thead>
          <tbody>
            {{range .Registrations}}
              <tr>
                <td>{{.ID}}</td>
                <td>{{.MemberName}}</td>
                <td>{{.MemberNo}}</td>
                <td>{{.TermName}}</td>
                <td>{{.CourseTitle}}</td>
                <td><span class="badge {{statusClass .Status}}">{{statusLabel .Status}}</span></td>
                <td>{{.CreatedAt}}</td>
                <td>
                  {{if eq .Status "selected"}}
                    <form class="inline-form" method="post" action="/admin/registrations/status">
                      <input type="hidden" name="registration_id" value="{{.ID}}">
                      <input type="hidden" name="action" value="confirm">
                      <button type="submit">확정</button>
                    </form>
                  {{end}}
                  {{if ne .Status "cancelled"}}
                    <form class="inline-form" method="post" action="/admin/registrations/status">
                      <input type="hidden" name="registration_id" value="{{.ID}}">
                      <input type="hidden" name="action" value="cancel">
                      <button class="danger" type="submit">취소</button>
                    </form>
                  {{end}}
                </td>
              </tr>
            {{else}}
              <tr><td class="empty" colspan="8">신청 내역이 없습니다.</td></tr>
            {{end}}
          </tbody>
        </table>
      </div>
    </section>
    <footer class="footer">{{.DisplayName}} {{.Version}}</footer>
  </main>
</body>
</html>
`))

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
		if err := registrationsTemplate.Execute(w, registrationsPageData{
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
			if _, err := opts.Registrations.Confirm(r.Context(), registrationID); err != nil {
				http.Error(w, err.Error(), http.StatusBadRequest)
				return
			}
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
			http.Redirect(w, r, "/admin/registrations?message="+url.QueryEscape(message), http.StatusSeeOther)
		default:
			http.Error(w, "unsupported registration action", http.StatusBadRequest)
		}
	}
}
