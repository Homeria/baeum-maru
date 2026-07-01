// Package web contains HTTP handlers, middleware, and route wiring.
package web

import (
	"context"
	"github.com/Homeria/baeum-maru/internal/config"
	"html/template"
	"io"
	"log/slog"
	"net/http"
	"strconv"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/service"
)

type RouterOptions struct {
	DisplayName   string
	Version       string
	Logger        *slog.Logger
	Members       MemberService
	Courses       CourseService
	Registrations RegistrationService
	Lotteries     LotteryService
	Exports       ExportService
	Imports       ImportService
	Backups       BackupService
	Attendance    AttendanceService
	Settings      SettingsService
}

type MemberService interface {
	Create(context.Context, service.MemberInput) (domain.Member, error)
	Search(context.Context, string, int) ([]domain.Member, error)
}

type CourseService interface {
	CreateOffering(context.Context, service.CourseOfferingInput) (domain.CourseOffering, error)
	ListOfferings(context.Context, int) ([]domain.CourseOffering, error)
}

type RegistrationService interface {
	Create(context.Context, service.RegistrationInput) (domain.Registration, error)
	Cancel(context.Context, int64) (domain.Registration, error)
	Confirm(context.Context, int64) (domain.Registration, error)
	CancelWithPromotion(context.Context, int64) (domain.RegistrationStatusChange, error)
	ListByMember(context.Context, int64) ([]domain.Registration, error)
	ListRecent(context.Context, int) ([]domain.Registration, error)
}

type ExportService interface {
	ExportMembers(context.Context) (service.ExportResult, error)
	ExportCourseOfferings(context.Context) (service.ExportResult, error)
	ExportRegistrations(context.Context) (service.ExportResult, error)
	ExportLotteryResults(context.Context, int64) (service.ExportResult, error)
	ExportAttendanceSession(context.Context, int64) (service.ExportResult, error)
	ExportAttendanceOffering(context.Context, int64) (service.ExportResult, error)
}

type ImportService interface {
	ImportMembers(context.Context, io.Reader) (service.ImportResult, error)
	ImportCourseOfferings(context.Context, io.Reader) (service.ImportResult, error)
	MemberTemplate() (service.ImportTemplate, error)
	CourseOfferingTemplate() (service.ImportTemplate, error)
}

type LotteryService interface {
	RunOfferingLottery(context.Context, int64, ...service.LotteryRunOptions) (domain.LotteryRunSummary, error)
	ListRuns(context.Context, int) ([]domain.LotteryRun, error)
}

type BackupService interface {
	CreateBackup(context.Context) (domain.BackupFile, error)
	ListBackups(context.Context) ([]domain.BackupFile, error)
	Status(context.Context) (domain.BackupStatus, error)
	PruneOldBackups(context.Context) (domain.BackupCleanup, error)
	ResolveBackupPath(string) (string, error)
	QueueRestore(context.Context, string) (domain.RestorePlan, error)
}

type AttendanceService interface {
	CreateSession(context.Context, service.AttendanceSessionInput) (domain.AttendanceSession, error)
	ListSessions(context.Context, int64, int) ([]domain.AttendanceSession, error)
	ListConfirmedByOffering(context.Context, int64) ([]domain.Registration, error)
	ListRecordsBySession(context.Context, int64) ([]domain.AttendanceRecord, error)
	SaveRecord(context.Context, service.AttendanceRecordInput) (domain.AttendanceRecord, error)
}

type SettingsService interface {
	Get(context.Context) (config.Config, error)
	Update(context.Context, service.SettingsInput) (config.Config, error)
}

type pageData struct {
	Title       string
	DisplayName string
	Version     string
	Heading     string
	Description string
}

var pageTemplate = template.Must(template.New("page").Funcs(uiTemplateFuncs(nil)).Parse(`<!doctype html>
<html lang="ko">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{{.Title}} - {{.DisplayName}}</title>
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
	<main class="page" aria-label="{{.Description}}">
    <section class="page-header">
      <div>
        <h1>{{.Heading}}</h1>
      </div>
    </section>
    <section class="panel">
      <div class="actions">
        <a class="button" href="/reception">접수 화면</a>
        <a class="button secondary" href="/admin/imports">엑셀 가져오기</a>
        <a class="button secondary" href="/admin/registrations">신청 현황</a>
        <a class="button secondary" href="/admin/lottery">추첨 관리</a>
        <a class="button secondary" href="/admin/backups">백업 관리</a>
      </div>
    </section>
    <footer class="footer">{{.DisplayName}} {{.Version}}</footer>
  </main>
</body>
</html>
`))

func NewRouter(opts RouterOptions) http.Handler {
	if opts.DisplayName == "" {
		opts.DisplayName = "배움마루"
	}
	if opts.Version == "" {
		opts.Version = "dev"
	}
	if opts.Logger == nil {
		opts.Logger = slog.Default()
	}

	mux := http.NewServeMux()
	mux.HandleFunc("/", exactPath("/", renderPage(opts, pageData{
		Title:       "홈",
		Heading:     "배움마루",
		Description: "로컬 호스팅 수강신청 업무 도구가 실행 중입니다.",
	})))
	mux.HandleFunc("/admin", exactPath("/admin", renderPage(opts, pageData{
		Title:       "관리",
		Heading:     "관리 화면",
		Description: "회원, 강좌, 신청 현황, 추첨, 출력을 관리하는 화면입니다.",
	})))
	mux.HandleFunc("/reception", receptionHandler(opts))
	mux.HandleFunc("/admin/members", membersHandler(opts))
	mux.HandleFunc("/admin/courses", coursesHandler(opts))
	mux.HandleFunc("/admin/registrations", registrationsHandler(opts))
	mux.HandleFunc("/admin/registrations/status", registrationStatusHandler(opts))
	mux.HandleFunc("/admin/lottery", lotteryHandler(opts))
	mux.HandleFunc("/admin/lottery/run", runLotteryHandler(opts))
	mux.HandleFunc("/admin/exports", exportsHandler(opts))
	mux.HandleFunc("/admin/exports/members", exportMembersHandler(opts))
	mux.HandleFunc("/admin/exports/courses", exportCoursesHandler(opts))
	mux.HandleFunc("/admin/exports/registrations", exportRegistrationsHandler(opts))
	mux.HandleFunc("/admin/exports/lottery-results", exportLotteryResultsHandler(opts))
	mux.HandleFunc("/admin/exports/attendance-session", exportAttendanceSessionHandler(opts))
	mux.HandleFunc("/admin/exports/attendance-offering", exportAttendanceOfferingHandler(opts))
	mux.HandleFunc("/admin/imports", importsHandler(opts))
	mux.HandleFunc("/admin/imports/members", importMembersHandler(opts))
	mux.HandleFunc("/admin/imports/courses", importCoursesHandler(opts))
	mux.HandleFunc("/admin/imports/members/template", memberImportTemplateHandler(opts))
	mux.HandleFunc("/admin/imports/courses/template", courseImportTemplateHandler(opts))
	mux.HandleFunc("/admin/backups", backupsHandler(opts))
	mux.HandleFunc("/admin/backups/create", createBackupHandler(opts))
	mux.HandleFunc("/admin/backups/download", downloadBackupHandler(opts))
	mux.HandleFunc("/admin/backups/restore", restoreBackupHandler(opts))
	mux.HandleFunc("/admin/attendance", attendanceHandler(opts))
	mux.HandleFunc("/admin/attendance/session", createAttendanceSessionHandler(opts))
	mux.HandleFunc("/admin/attendance/record", saveAttendanceRecordHandler(opts))
	mux.HandleFunc("/admin/settings", settingsHandler(opts))
	mux.HandleFunc("/reception/cancel", cancelRegistrationHandler(opts))
	mux.HandleFunc("/healthz", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodGet {
			w.Header().Set("Allow", http.MethodGet)
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		w.Header().Set("Content-Type", "text/plain; charset=utf-8")
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte("ok\n"))
	})

	return mux
}

func exactPath(path string, next http.HandlerFunc) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != path {
			http.NotFound(w, r)
			return
		}
		if r.Method != http.MethodGet {
			w.Header().Set("Allow", http.MethodGet)
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		next(w, r)
	}
}

type membersPageData struct {
	DisplayName string
	Version     string
	Query       string
	Error       string
	Members     []domain.Member
}

var membersTemplate = template.Must(template.New("members").Funcs(uiTemplateFuncs(nil)).Parse(`<!doctype html>
<html lang="ko">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>회원 관리 - {{.DisplayName}}</title>
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
        <h1>회원 관리</h1>
      </div>
      <a class="button secondary" href="/admin/exports/members">엑셀 다운로드</a>
    </section>
    {{if .Error}}<p class="alert error" role="alert">{{.Error}}</p>{{end}}

    <form class="panel form-grid" method="get" action="/admin/members">
      <label>검색 <input name="q" value="{{.Query}}" placeholder="이름, 회원번호, 연락처"></label>
      <button type="submit">검색</button>
    </form>

    <form class="panel" method="post" action="/admin/members">
      <h2>회원 등록</h2>
      <div class="form-grid">
        <label>회원번호 <input name="member_no"></label>
        <label>이름 <input name="name" required></label>
        <label>성별
          <select name="gender_code">
            <option value="">선택 안 함</option>
            <option value="male">남성</option>
            <option value="female">여성</option>
            <option value="unknown">미상</option>
          </select>
        </label>
        <label>생년월일 <input name="birth_date" placeholder="YYYY-MM-DD"></label>
        <label>연락처 <input name="phone"></label>
        <label>비고 <input name="note"></label>
        <button type="submit">등록</button>
      </div>
    </form>

    <section class="panel">
      <h2>회원 목록</h2>
      <div class="table-wrap">
        <table>
          <thead>
            <tr><th>ID</th><th>회원번호</th><th>이름</th><th>성별</th><th>생년월일</th><th>연락처</th><th>비고</th></tr>
          </thead>
          <tbody>
            {{range .Members}}
              <tr>
                <td>{{.ID}}</td>
                <td>{{.MemberNo}}</td>
                <td>{{.Name}}</td>
                <td>{{.GenderCode}}</td>
                <td>{{.BirthDate}}</td>
                <td>{{.Phone}}</td>
                <td>{{.Note}}</td>
              </tr>
            {{else}}
              <tr><td class="empty" colspan="7">회원이 없습니다.</td></tr>
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
			_, err := opts.Members.Create(r.Context(), service.MemberInput{
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
	if err := membersTemplate.Execute(w, membersPageData{
		DisplayName: opts.DisplayName,
		Version:     opts.Version,
		Query:       query,
		Error:       message,
		Members:     members,
	}); err != nil {
		opts.Logger.Error("render members failed", "error", err)
	}
}

type coursesPageData struct {
	DisplayName string
	Version     string
	Error       string
	Offerings   []domain.CourseOffering
}

var coursesTemplate = template.Must(template.New("courses").Funcs(uiTemplateFuncs(template.FuncMap{
	"weekdayLabel": weekdayLabel,
})).Parse(`<!doctype html>
<html lang="ko">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>강좌 관리 - {{.DisplayName}}</title>
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
        <h1>강좌 관리</h1>
      </div>
      <a class="button secondary" href="/admin/exports/courses">엑셀 다운로드</a>
    </section>
    {{if .Error}}<p class="alert error" role="alert">{{.Error}}</p>{{end}}

    <form class="panel" method="post" action="/admin/courses">
      <h2>강좌 개설 등록</h2>
      <div class="form-grid">
        <label>회차 <input name="term_name" placeholder="예: 2026년 여름학기"></label>
        <label>분류 <input name="category_name" placeholder="예: 건강"></label>
        <label>강좌명 <input name="course_title" required></label>
        <label>강사 <input name="instructor_name"></label>
        <label>강의실 <input name="classroom_name"></label>
        <label>정원 <input name="capacity" type="number" min="0" value="0"></label>
        <label>요일
          <select name="weekday">
            <option value="1">월</option>
            <option value="2">화</option>
            <option value="3">수</option>
            <option value="4">목</option>
            <option value="5">금</option>
            <option value="6">토</option>
            <option value="0">일</option>
          </select>
        </label>
        <label>시작 <input name="start_time" placeholder="09:00" required></label>
        <label>종료 <input name="end_time" placeholder="10:00" required></label>
        <label>비고 <input name="note"></label>
        <button type="submit">등록</button>
      </div>
    </form>

    <section class="panel">
      <h2>강좌 개설 목록</h2>
      <div class="table-wrap">
        <table>
          <thead>
            <tr><th>ID</th><th>회차</th><th>분류</th><th>강좌명</th><th>강사</th><th>강의실</th><th>정원</th><th>시간</th><th>신청</th></tr>
          </thead>
          <tbody>
            {{range .Offerings}}
              <tr>
                <td>{{.ID}}</td>
                <td>{{.TermName}}</td>
                <td>{{.CategoryName}}</td>
                <td>{{.CourseTitle}}</td>
                <td>{{.InstructorName}}</td>
                <td>{{.ClassroomName}}</td>
                <td>{{.Capacity}}</td>
                <td>{{weekdayLabel .Weekday}} {{.StartTime}}-{{.EndTime}}</td>
                <td><span class="badge">{{.RegistrationCount}}</span></td>
              </tr>
            {{else}}
              <tr><td class="empty" colspan="9">강좌가 없습니다.</td></tr>
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

func coursesHandler(opts RouterOptions) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if opts.Courses == nil {
			http.Error(w, "course service is not configured", http.StatusServiceUnavailable)
			return
		}

		switch r.Method {
		case http.MethodGet:
			renderCourses(w, r, opts, "")
		case http.MethodPost:
			if err := r.ParseForm(); err != nil {
				http.Error(w, "invalid form", http.StatusBadRequest)
				return
			}
			capacity, err := strconv.Atoi(r.FormValue("capacity"))
			if err != nil {
				renderCourses(w, r, opts, "정원은 숫자로 입력해야 합니다.")
				return
			}
			weekday, err := strconv.Atoi(r.FormValue("weekday"))
			if err != nil {
				renderCourses(w, r, opts, "요일 값이 올바르지 않습니다.")
				return
			}
			_, err = opts.Courses.CreateOffering(r.Context(), service.CourseOfferingInput{
				TermName:       r.FormValue("term_name"),
				CategoryName:   r.FormValue("category_name"),
				CourseTitle:    r.FormValue("course_title"),
				InstructorName: r.FormValue("instructor_name"),
				ClassroomName:  r.FormValue("classroom_name"),
				Capacity:       capacity,
				Weekday:        weekday,
				StartTime:      r.FormValue("start_time"),
				EndTime:        r.FormValue("end_time"),
				Note:           r.FormValue("note"),
			})
			if err != nil {
				renderCourses(w, r, opts, err.Error())
				return
			}
			http.Redirect(w, r, "/admin/courses", http.StatusSeeOther)
		default:
			w.Header().Set("Allow", "GET, POST")
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		}
	}
}

func renderCourses(w http.ResponseWriter, r *http.Request, opts RouterOptions, message string) {
	offerings, err := opts.Courses.ListOfferings(r.Context(), 100)
	if err != nil {
		message = err.Error()
	}

	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	if err := coursesTemplate.Execute(w, coursesPageData{
		DisplayName: opts.DisplayName,
		Version:     opts.Version,
		Error:       message,
		Offerings:   offerings,
	}); err != nil {
		opts.Logger.Error("render courses failed", "error", err)
	}
}

func weekdayLabel(weekday int) string {
	switch weekday {
	case 0:
		return "일"
	case 1:
		return "월"
	case 2:
		return "화"
	case 3:
		return "수"
	case 4:
		return "목"
	case 5:
		return "금"
	case 6:
		return "토"
	default:
		return "?"
	}
}

func renderPage(opts RouterOptions, data pageData) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		data.DisplayName = opts.DisplayName
		data.Version = opts.Version

		w.Header().Set("Content-Type", "text/html; charset=utf-8")
		if err := pageTemplate.Execute(w, data); err != nil {
			opts.Logger.Error("render page failed", "path", r.URL.Path, "error", err)
		}
	}
}
