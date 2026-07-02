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
	Permissions permissionSet
	Message     string
	Error       string
	Offerings   []lotteryOfferingRow
	Runs        []domain.LotteryRun
}

type lotteryOfferingRow struct {
	Offering  domain.CourseOffering
	LatestRun *domain.LotteryRun
}

var lotteryTemplate = mustPageTemplate("lottery", "lottery.html", template.FuncMap{"weekdayLabel": weekdayLabel})

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
	if err := lotteryTemplate.ExecuteTemplate(w, "lottery", lotteryPageData{
		DisplayName: opts.DisplayName,
		Version:     opts.Version,
		Permissions: pagePermissions(r),
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
