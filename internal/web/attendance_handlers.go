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

var attendanceTemplate = mustPageTemplate("attendance", "attendance.html", template.FuncMap{"weekdayLabel": weekdayLabel})

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
		recordAudit(r, opts, "attendance.session.create", "attendance_session", session.ID, "출석 회차 생성 #"+strconv.FormatInt(session.ID, 10))
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
		record, err := opts.Attendance.SaveRecord(r.Context(), service.AttendanceRecordInput{
			SessionID:      sessionID,
			RegistrationID: registrationID,
			Status:         r.FormValue("status"),
			Note:           r.FormValue("note"),
		})
		if err != nil {
			renderAttendance(w, r, opts, "", err.Error())
			return
		}
		recordAudit(r, opts, "attendance.record.save", "attendance_record", record.ID, "출석 저장 / 회차 #"+strconv.FormatInt(sessionID, 10)+" / 신청 #"+strconv.FormatInt(registrationID, 10))
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
	if err := attendanceTemplate.ExecuteTemplate(w, "attendance", attendancePageData{
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
