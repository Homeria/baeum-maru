package web

import (
	"errors"
	"html/template"
	"net/http"
	"net/url"
	"strconv"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/service"
)

type lotteryPageData struct {
	DisplayName string
	Version     string
	Message     string
	Error       string
	Offerings   []lotteryOfferingRow
	Runs        []domain.LotteryRun
}

type lotteryOfferingRow struct {
	Offering  domain.CourseOffering
	LatestRun *domain.LotteryRun
}

var lotteryTemplate = template.Must(template.New("lottery").Funcs(uiTemplateFuncs(template.FuncMap{
	"weekdayLabel": weekdayLabel,
})).Parse(`<!doctype html>
<html lang="ko">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>추첨 - {{.DisplayName}}</title>
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
      <a href="/admin/audit-logs">감사 로그</a>
      <a href="/reception">접수 화면</a>
    </nav>
  </header>
  <main class="page">
    <section class="page-header">
      <div>
        <h1>추첨</h1>
      </div>
      <a class="button secondary" href="/admin/backups">추첨 전 백업</a>
    </section>
    {{if .Message}}<p class="alert success" role="status">{{.Message}}</p>{{end}}
    {{if .Error}}<p class="alert error" role="alert">{{.Error}}</p>{{end}}
    <section class="panel">
      <h2>강좌별 추첨</h2>
      <div class="table-wrap">
        <table>
          <thead>
            <tr><th>ID</th><th>회차</th><th>강좌명</th><th>정원</th><th>신청</th><th>시간</th><th>최근 추첨</th><th>작업</th></tr>
          </thead>
          <tbody>
            {{range .Offerings}}
              <tr>
                <td>{{.Offering.ID}}</td>
                <td>{{.Offering.TermName}}</td>
                <td>{{.Offering.CourseTitle}}</td>
                <td>{{.Offering.Capacity}}</td>
                <td><span class="badge">{{.Offering.RegistrationCount}}</span></td>
                <td>{{weekdayLabel .Offering.Weekday}} {{.Offering.StartTime}}-{{.Offering.EndTime}}</td>
                <td>
                  {{if .LatestRun}}
                    <span class="badge completed">완료 #{{.LatestRun.ID}}</span>
                  {{else}}
                    <span class="badge pending">없음</span>
                  {{end}}
                </td>
                <td>
                  {{if .LatestRun}}
                    <form class="inline-form" method="post" action="/admin/lottery/run">
                      <input type="hidden" name="offering_id" value="{{.Offering.ID}}">
                      <input type="hidden" name="force_rerun" value="true">
                      <label style="display:inline-flex;grid-template-columns:auto 1fr;align-items:center;gap:6px;">
                        <input name="confirm_rerun" type="checkbox" value="true" required style="width:auto;min-height:auto;">
                        재추첨 확인
                      </label>
                      <button class="danger" type="submit">재추첨</button>
                    </form>
                  {{else}}
                    <form class="inline-form" method="post" action="/admin/lottery/run">
                      <input type="hidden" name="offering_id" value="{{.Offering.ID}}">
                      <button type="submit">추첨 실행</button>
                    </form>
                  {{end}}
                </td>
              </tr>
            {{else}}
              <tr><td class="empty" colspan="8">강좌가 없습니다.</td></tr>
            {{end}}
          </tbody>
        </table>
      </div>
    </section>
    <section class="panel">
      <h2>추첨 실행 이력</h2>
      <div class="table-wrap">
        <table>
          <thead>
            <tr><th>ID</th><th>회차</th><th>강좌명</th><th>상태</th><th>전체</th><th>선정</th><th>대기</th><th>완료일</th><th>파일</th></tr>
          </thead>
          <tbody>
            {{range .Runs}}
              <tr>
                <td>{{.ID}}</td>
                <td>{{.TermName}}</td>
                <td>{{.CourseTitle}}</td>
                <td><span class="badge {{statusClass .Status}}">{{statusLabel .Status}}</span></td>
                <td>{{.TotalCount}}</td>
                <td>{{.SelectedCount}}</td>
                <td>{{.WaitlistedCount}}</td>
                <td>{{.CompletedAt}}</td>
                <td><a class="button secondary" href="/admin/exports/lottery-results?run_id={{.ID}}">결과 다운로드</a></td>
              </tr>
            {{else}}
              <tr><td class="empty" colspan="9">추첨 실행 이력이 없습니다.</td></tr>
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
		forceRerun := r.FormValue("force_rerun") == "true" && r.FormValue("confirm_rerun") == "true"
		summary, err := opts.Lotteries.RunOfferingLottery(r.Context(), offeringID, service.LotteryRunOptions{ForceRerun: forceRerun})
		if err != nil {
			var rerunErr *service.LotteryRerunRequiredError
			if errors.As(err, &rerunErr) {
				message := "이미 추첨 이력이 있습니다. 강좌 행의 재추첨 확인을 체크한 뒤 다시 실행하세요."
				http.Redirect(w, r, "/admin/lottery?message="+url.QueryEscape(message), http.StatusSeeOther)
				return
			}
			renderLottery(w, r, opts, "", err.Error())
			return
		}
		message := "추첨 완료: " + summary.CourseTitle + " / 선정 " + strconv.Itoa(summary.SelectedCount) + "명 / 대기 " + strconv.Itoa(summary.WaitlistedCount) + "명"
		if summary.Rerun {
			message = "재추첨 완료: " + summary.CourseTitle + " / 이전 추첨 #" + strconv.FormatInt(summary.PreviousRunID, 10) + " 보존 / 선정 " + strconv.Itoa(summary.SelectedCount) + "명 / 대기 " + strconv.Itoa(summary.WaitlistedCount) + "명"
		}
		action := "lottery.run"
		summaryText := "추첨 실행 #" + strconv.FormatInt(summary.RunID, 10) + " / 강좌 #" + strconv.FormatInt(summary.OfferingID, 10)
		if summary.Rerun {
			action = "lottery.rerun"
			summaryText = "재추첨 실행 #" + strconv.FormatInt(summary.RunID, 10) + " / 이전 추첨 #" + strconv.FormatInt(summary.PreviousRunID, 10)
		}
		recordAudit(r, opts, action, "lottery_run", summary.RunID, summaryText)
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
	var runs []domain.LotteryRun
	if opts.Lotteries != nil {
		runs, err = opts.Lotteries.ListRuns(r.Context(), 50)
		if err != nil {
			errorMessage = err.Error()
		}
	}
	offeringRows := buildLotteryOfferingRows(offerings, runs)

	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	if err := lotteryTemplate.Execute(w, lotteryPageData{
		DisplayName: opts.DisplayName,
		Version:     opts.Version,
		Message:     message,
		Error:       errorMessage,
		Offerings:   offeringRows,
		Runs:        runs,
	}); err != nil {
		opts.Logger.Error("render lottery failed", "error", err)
	}
}

func buildLotteryOfferingRows(offerings []domain.CourseOffering, runs []domain.LotteryRun) []lotteryOfferingRow {
	latestByOffering := make(map[int64]domain.LotteryRun)
	for _, run := range runs {
		if run.OfferingID <= 0 {
			continue
		}
		if _, exists := latestByOffering[run.OfferingID]; !exists {
			latestByOffering[run.OfferingID] = run
		}
	}
	rows := make([]lotteryOfferingRow, 0, len(offerings))
	for _, offering := range offerings {
		row := lotteryOfferingRow{Offering: offering}
		if run, exists := latestByOffering[offering.ID]; exists {
			runCopy := run
			row.LatestRun = &runCopy
		}
		rows = append(rows, row)
	}
	return rows
}
