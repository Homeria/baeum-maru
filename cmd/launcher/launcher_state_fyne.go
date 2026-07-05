//go:build fyne

package main

import (
	"context"
	"fmt"
	"strings"
	"time"

	"fyne.io/fyne/v2"
	"fyne.io/fyne/v2/widget"

	"github.com/Homeria/baeum-maru/internal/app"
	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/service"
)

type launcherState struct {
	runtime   *app.Runtime
	guiApp    fyne.App
	serverURL string
	shareURLs []string

	summaries            []service.AccessCodeSummary
	accessFilter         string
	selectedAccessCodeID int64
	logLines             []string

	statusTitleLabels  []*widget.Label
	statusDetailLabels []*widget.Label
	activityLabels     []*widget.Label
	logLabels          []*widget.Label
	activeCountLabels  []*widget.Label
	selectedCodeLabels []*widget.Label
	codeLists          []*widget.List
}

func newLauncherState(runtime *app.Runtime, guiApp fyne.App, serverURL string, shareURLs []string) *launcherState {
	return &launcherState{
		runtime:      runtime,
		guiApp:       guiApp,
		serverURL:    serverURL,
		shareURLs:    shareURLs,
		accessFilter: "all",
	}
}

func (s *launcherState) setStatus(title string, detail string) {
	for _, label := range s.statusTitleLabels {
		label.SetText(title)
	}
	for _, label := range s.statusDetailLabels {
		label.SetText(detail)
	}
	for _, label := range s.activityLabels {
		label.SetText(detail)
	}
	s.appendLog(detail)
}

func (s *launcherState) appendLog(message string) {
	line := time.Now().Format("15:04:05") + "  " + message
	s.logLines = append([]string{line}, s.logLines...)
	if len(s.logLines) > 100 {
		s.logLines = s.logLines[:100]
	}
	text := strings.Join(s.logLines, "\n")
	for _, label := range s.logLabels {
		label.SetText(text)
	}
}

func (s *launcherState) refreshAccessCodes() {
	items, err := s.runtime.AccessAuth.ListRecentAccessCodes(context.Background(), 100)
	if err != nil {
		s.setStatus("코드 목록 오류", "접속 코드 목록 조회 실패: "+err.Error())
		return
	}

	s.summaries = items
	s.selectedAccessCodeID = 0
	s.updateAccessSummaryLabels()
	for _, label := range s.selectedCodeLabels {
		label.SetText("선택된 코드가 없습니다.")
	}
	for _, list := range s.codeLists {
		list.UnselectAll()
		list.Refresh()
	}
	s.appendLog(fmt.Sprintf("접속 코드 %d개를 불러왔습니다.", len(items)))
}

func (s *launcherState) setAccessFilter(filter string) {
	s.accessFilter = filter
	s.selectedAccessCodeID = 0
	for _, label := range s.selectedCodeLabels {
		label.SetText("선택된 코드가 없습니다.")
	}
	for _, list := range s.codeLists {
		list.UnselectAll()
		list.Refresh()
	}
}

func (s *launcherState) filteredAccessCodes() []service.AccessCodeSummary {
	if s.accessFilter == "" || s.accessFilter == "all" {
		return s.summaries
	}
	filtered := make([]service.AccessCodeSummary, 0, len(s.summaries))
	for _, summary := range s.summaries {
		if s.accessFilter == "active" && summary.Status == domain.AccessCodeStatusActive {
			filtered = append(filtered, summary)
		}
		if s.accessFilter == "expired" && summary.Status == domain.AccessCodeStatusExpired {
			filtered = append(filtered, summary)
		}
	}
	return filtered
}

func (s *launcherState) selectAccessCode(id widget.ListItemID) {
	items := s.filteredAccessCodes()
	if id < 0 || id >= len(items) {
		return
	}
	s.selectedAccessCodeID = items[id].ID
	text := "선택: " + compactAccessCodeTitle(items[id])
	for _, label := range s.selectedCodeLabels {
		label.SetText(text)
	}
}

func (s *launcherState) copyToClipboard(title string, content string) {
	if strings.TrimSpace(content) == "" {
		s.setStatus(title, "복사할 내용이 없습니다.")
		return
	}
	s.guiApp.Clipboard().SetContent(content)
	s.setStatus(title, "클립보드에 복사했습니다.")
}

func (s *launcherState) updateAccessSummaryLabels() {
	text := fmt.Sprintf("사용 가능 %d / 전체 %d", activeAccessCodeCount(s.summaries), len(s.summaries))
	for _, label := range s.activeCountLabels {
		label.SetText(text)
	}
}

func (s *launcherState) addStatusLabels(title *widget.Label, detail *widget.Label) {
	s.statusTitleLabels = append(s.statusTitleLabels, title)
	s.statusDetailLabels = append(s.statusDetailLabels, detail)
}

func (s *launcherState) addActivityLabel(label *widget.Label) {
	s.activityLabels = append(s.activityLabels, label)
}

func (s *launcherState) addLogLabel(label *widget.Label) {
	s.logLabels = append(s.logLabels, label)
}

func (s *launcherState) addAccessSummaryLabel(label *widget.Label) {
	s.activeCountLabels = append(s.activeCountLabels, label)
	s.updateAccessSummaryLabels()
}

func (s *launcherState) addSelectedCodeLabel(label *widget.Label) {
	s.selectedCodeLabels = append(s.selectedCodeLabels, label)
}

func (s *launcherState) addCodeList(list *widget.List) {
	s.codeLists = append(s.codeLists, list)
}

func activeAccessCodeCount(items []service.AccessCodeSummary) int {
	count := 0
	for _, item := range items {
		if item.Status == domain.AccessCodeStatusActive {
			count++
		}
	}
	return count
}
