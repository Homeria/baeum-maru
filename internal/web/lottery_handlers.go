package web

import (
	"html/template"
	"net/http"
	"net/url"
	"strconv"

	"github.com/Homeria/baeum-maru/internal/domain"
)

type lotteryPageData struct {
	DisplayName string
	Version     string
	Message     string
	Error       string
	Offerings   []domain.CourseOffering
}

var lotteryTemplate = template.Must(template.New("lottery").Funcs(template.FuncMap{
	"weekdayLabel": weekdayLabel,
}).Parse(`<!doctype html>
<html lang="ko">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>추첨 - {{.DisplayName}}</title>
</head>
<body>
  <main>
    <nav><a href="/admin">관리</a> <a href="/admin/courses">강좌 관리</a> <a href="/admin/registrations">신청 현황</a></nav>
    <h1>추첨</h1>
    {{if .Message}}<p role="status">{{.Message}}</p>{{end}}
    {{if .Error}}<p role="alert">{{.Error}}</p>{{end}}
    <table>
      <thead>
        <tr><th>ID</th><th>회차</th><th>강좌명</th><th>정원</th><th>신청</th><th>시간</th><th>작업</th></tr>
      </thead>
      <tbody>
        {{range .Offerings}}
          <tr>
            <td>{{.ID}}</td>
            <td>{{.TermName}}</td>
            <td>{{.CourseTitle}}</td>
            <td>{{.Capacity}}</td>
            <td>{{.RegistrationCount}}</td>
            <td>{{weekdayLabel .Weekday}} {{.StartTime}}-{{.EndTime}}</td>
            <td>
              <form method="post" action="/admin/lottery/run">
                <input type="hidden" name="offering_id" value="{{.ID}}">
                <button type="submit">추첨 실행</button>
              </form>
            </td>
          </tr>
        {{else}}
          <tr><td colspan="7">강좌가 없습니다.</td></tr>
        {{end}}
      </tbody>
    </table>
    <small>{{.DisplayName}} {{.Version}}</small>
  </main>
</body>
</html>
`))

func lotteryHandler(opts RouterOptions) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/admin/lottery" {
			http.NotFound(w, r)
			return
		}
		if r.Method != http.MethodGet {
			w.Header().Set("Allow", http.MethodGet)
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		renderLottery(w, r, opts, r.URL.Query().Get("message"), "")
	}
}

func runLotteryHandler(opts RouterOptions) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if opts.Lotteries == nil {
			http.Error(w, "lottery service is not configured", http.StatusServiceUnavailable)
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
			renderLottery(w, r, opts, "", "강좌 선택이 올바르지 않습니다.")
			return
		}
		summary, err := opts.Lotteries.RunOfferingLottery(r.Context(), offeringID)
		if err != nil {
			renderLottery(w, r, opts, "", err.Error())
			return
		}
		message := "추첨 완료: " + summary.CourseTitle + " / 선정 " + strconv.Itoa(summary.SelectedCount) + "명 / 대기 " + strconv.Itoa(summary.WaitlistedCount) + "명"
		http.Redirect(w, r, "/admin/lottery?message="+url.QueryEscape(message), http.StatusSeeOther)
	}
}

func renderLottery(w http.ResponseWriter, r *http.Request, opts RouterOptions, message string, errorMessage string) {
	if opts.Courses == nil {
		http.Error(w, "course service is not configured", http.StatusServiceUnavailable)
		return
	}
	offerings, err := opts.Courses.ListOfferings(r.Context(), 200)
	if err != nil {
		errorMessage = err.Error()
	}

	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	if err := lotteryTemplate.Execute(w, lotteryPageData{
		DisplayName: opts.DisplayName,
		Version:     opts.Version,
		Message:     message,
		Error:       errorMessage,
		Offerings:   offerings,
	}); err != nil {
		opts.Logger.Error("render lottery failed", "error", err)
	}
}
