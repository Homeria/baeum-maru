package web

import (
	"html/template"
	"net/http"
	"net/url"
	"strconv"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/service"
)

type attendancePageData struct {
	DisplayName string
	Version     string
	Message     string
	Error       string
	OfferingID  int64
	SessionID   int64
	Offerings   []domain.CourseOffering
	Sessions    []domain.AttendanceSession
	Confirmed   []domain.Registration
	Records     []domain.AttendanceRecord
}

var attendanceTemplate = template.Must(template.New("attendance").Funcs(uiTemplateFuncs(template.FuncMap{
	"weekdayLabel": weekdayLabel,
})).Parse(`<!doctype html>
<html lang="ko">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>출석 - {{.DisplayName}}</title>
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
        <h1>출석</h1>
      </div>
    </section>
    {{if .Message}}<p class="alert success" role="status">{{.Message}}</p>{{end}}
    {{if .Error}}<p class="alert error" role="alert">{{.Error}}</p>{{end}}

    <form class="panel form-grid" method="get" action="/admin/attendance">
      <label>강좌
        <select name="offering_id">
          <option value="">선택</option>
          {{range .Offerings}}
            <option value="{{.ID}}" {{if eq $.OfferingID .ID}}selected{{end}}>{{.TermName}} / {{.CourseTitle}} / {{weekdayLabel .Weekday}} {{.StartTime}}-{{.EndTime}}</option>
          {{end}}
        </select>
      </label>
      <button type="submit">조회</button>
    </form>

    {{if .OfferingID}}
      <section class="panel">
        <div class="page-header">
          <div>
            <h2>출석 회차</h2>
          </div>
          <a class="button secondary" href="/admin/exports/attendance-offering?offering_id={{.OfferingID}}">강좌 전체 출석 다운로드</a>
        </div>
        <form class="form-grid" method="post" action="/admin/attendance/session">
          <input type="hidden" name="offering_id" value="{{.OfferingID}}">
          <label>수업일 <input name="session_date" placeholder="YYYY-MM-DD" required></label>
          <label>비고 <input name="note"></label>
          <button type="submit">회차 생성</button>
        </form>

        <div class="table-wrap" style="margin-top: 14px;">
          <table>
            <thead><tr><th>ID</th><th>수업일</th><th>강좌</th><th>비고</th><th>작업</th></tr></thead>
            <tbody>
              {{range .Sessions}}
                <tr>
                  <td>{{.ID}}</td>
                  <td>{{.SessionDate}}</td>
                  <td>{{.CourseTitle}}</td>
                  <td>{{.Note}}</td>
                  <td>
                    <div class="actions">
                      <a class="button secondary" href="/admin/attendance?offering_id={{.OfferingID}}&session_id={{.ID}}">출석 입력</a>
                      <a class="button secondary" href="/admin/exports/attendance-session?session_id={{.ID}}">엑셀 다운로드</a>
                    </div>
                  </td>
                </tr>
              {{else}}
                <tr><td class="empty" colspan="5">출석 회차가 없습니다.</td></tr>
              {{end}}
            </tbody>
          </table>
        </div>
      </section>

      <section class="panel">
        <h2>확정자</h2>
        <div class="table-wrap">
          <table>
            <thead><tr><th>신청 ID</th><th>회원번호</th><th>회원명</th><th>상태</th></tr></thead>
            <tbody>
              {{range .Confirmed}}
                <tr><td>{{.ID}}</td><td>{{.MemberNo}}</td><td>{{.MemberName}}</td><td><span class="badge {{statusClass .Status}}">{{statusLabel .Status}}</span></td></tr>
              {{else}}
                <tr><td class="empty" colspan="4">확정자가 없습니다.</td></tr>
              {{end}}
            </tbody>
          </table>
        </div>
      </section>
    {{end}}

    {{if .SessionID}}
      <section class="panel">
        <div class="page-header">
          <div>
            <h2>출석 입력</h2>
          </div>
          <a class="button secondary" href="/admin/exports/attendance-session?session_id={{.SessionID}}">현재 회차 출석 다운로드</a>
        </div>
        <div class="table-wrap">
          <table>
            <thead><tr><th>회원번호</th><th>회원명</th><th>상태</th><th>비고</th><th>저장</th></tr></thead>
            <tbody>
              {{range .Records}}
                <tr>
                  <td>{{.MemberNo}}</td>
                  <td>{{.MemberName}}</td>
                  <td>
                    <form class="inline-form" method="post" action="/admin/attendance/record">
                      <input type="hidden" name="offering_id" value="{{$.OfferingID}}">
                      <input type="hidden" name="session_id" value="{{$.SessionID}}">
                      <input type="hidden" name="registration_id" value="{{.RegistrationID}}">
                      <select name="status" required>
                        <option value="">선택</option>
                        <option value="present" {{if eq .Status "present"}}selected{{end}}>출석</option>
                        <option value="absent" {{if eq .Status "absent"}}selected{{end}}>결석</option>
                        <option value="late" {{if eq .Status "late"}}selected{{end}}>지각</option>
                        <option value="excused" {{if eq .Status "excused"}}selected{{end}}>공결</option>
                      </select>
                  </td>
                  <td><input name="note" value="{{.Note}}"></td>
                  <td><button type="submit">저장</button></form></td>
                </tr>
              {{else}}
                <tr><td class="empty" colspan="5">출석 대상자가 없습니다.</td></tr>
              {{end}}
            </tbody>
          </table>
        </div>
      </section>
    {{end}}
    <footer class="footer">{{.DisplayName}} {{.Version}}</footer>
  </main>
</body>
</html>
`))

func attendanceHandler(opts RouterOptions) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/admin/attendance" {
			http.NotFound(w, r)
			return
		}
		if r.Method != http.MethodGet {
			w.Header().Set("Allow", http.MethodGet)
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		renderAttendance(w, r, opts, r.URL.Query().Get("message"), "")
	}
}

func createAttendanceSessionHandler(opts RouterOptions) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if opts.Attendance == nil {
			http.Error(w, "attendance service is not configured", http.StatusServiceUnavailable)
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
		offeringID, err := strconv.ParseInt(r.FormValue("offering_id"), 10, 64)
		if err != nil {
			http.Error(w, "invalid offering id", http.StatusBadRequest)
			return
		}
		session, err := opts.Attendance.CreateSession(r.Context(), service.AttendanceSessionInput{
			OfferingID:  offeringID,
			SessionDate: r.FormValue("session_date"),
			Note:        r.FormValue("note"),
		})
		if err != nil {
			renderAttendance(w, r, opts, "", err.Error())
			return
		}
		redirect := "/admin/attendance?offering_id=" + strconv.FormatInt(offeringID, 10) + "&session_id=" + strconv.FormatInt(session.ID, 10) + "&message=" + url.QueryEscape("출석 회차를 생성했습니다.")
		http.Redirect(w, r, redirect, http.StatusSeeOther)
	}
}

func saveAttendanceRecordHandler(opts RouterOptions) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if opts.Attendance == nil {
			http.Error(w, "attendance service is not configured", http.StatusServiceUnavailable)
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
		offeringID, _ := strconv.ParseInt(r.FormValue("offering_id"), 10, 64)
		sessionID, err := strconv.ParseInt(r.FormValue("session_id"), 10, 64)
		if err != nil {
			http.Error(w, "invalid attendance session id", http.StatusBadRequest)
			return
		}
		registrationID, err := strconv.ParseInt(r.FormValue("registration_id"), 10, 64)
		if err != nil {
			http.Error(w, "invalid registration id", http.StatusBadRequest)
			return
		}
		if _, err := opts.Attendance.SaveRecord(r.Context(), service.AttendanceRecordInput{
			SessionID:      sessionID,
			RegistrationID: registrationID,
			Status:         r.FormValue("status"),
			Note:           r.FormValue("note"),
		}); err != nil {
			renderAttendance(w, r, opts, "", err.Error())
			return
		}
		redirect := "/admin/attendance?offering_id=" + strconv.FormatInt(offeringID, 10) + "&session_id=" + strconv.FormatInt(sessionID, 10) + "&message=" + url.QueryEscape("출석을 저장했습니다.")
		http.Redirect(w, r, redirect, http.StatusSeeOther)
	}
}

func renderAttendance(w http.ResponseWriter, r *http.Request, opts RouterOptions, message string, errorMessage string) {
	if opts.Attendance == nil || opts.Courses == nil {
		http.Error(w, "attendance services are not configured", http.StatusServiceUnavailable)
		return
	}

	offeringID, _ := strconv.ParseInt(r.URL.Query().Get("offering_id"), 10, 64)
	sessionID, _ := strconv.ParseInt(r.URL.Query().Get("session_id"), 10, 64)

	offerings, err := opts.Courses.ListOfferings(r.Context(), 200)
	if err != nil {
		errorMessage = err.Error()
	}
	var sessions []domain.AttendanceSession
	var confirmed []domain.Registration
	if offeringID > 0 {
		sessions, err = opts.Attendance.ListSessions(r.Context(), offeringID, 100)
		if err != nil {
			errorMessage = err.Error()
		}
		confirmed, err = opts.Attendance.ListConfirmedByOffering(r.Context(), offeringID)
		if err != nil {
			errorMessage = err.Error()
		}
	}
	var records []domain.AttendanceRecord
	if sessionID > 0 {
		records, err = opts.Attendance.ListRecordsBySession(r.Context(), sessionID)
		if err != nil {
			errorMessage = err.Error()
		}
	}

	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	if err := attendanceTemplate.Execute(w, attendancePageData{
		DisplayName: opts.DisplayName,
		Version:     opts.Version,
		Message:     message,
		Error:       errorMessage,
		OfferingID:  offeringID,
		SessionID:   sessionID,
		Offerings:   offerings,
		Sessions:    sessions,
		Confirmed:   confirmed,
		Records:     records,
	}); err != nil {
		opts.Logger.Error("render attendance failed", "error", err)
	}
}
