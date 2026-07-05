//go:build fyne

package main

import (
	"strconv"

	"fyne.io/fyne/v2"
	"fyne.io/fyne/v2/container"
	"fyne.io/fyne/v2/theme"
	"fyne.io/fyne/v2/widget"
)

func buildDashboardTab(state *launcherState) fyne.CanvasObject {
	statusTitle := widget.NewLabelWithStyle("서버 시작 중", fyne.TextAlignLeading, fyne.TextStyle{Bold: true})
	statusDetail := widget.NewLabel("로컬 서버를 준비하고 있습니다.")
	statusDetail.Wrapping = fyne.TextWrapWord
	state.addStatusLabels(statusTitle, statusDetail)

	addressEntry := widget.NewEntry()
	addressEntry.SetText(state.serverURL)
	addressEntry.Disable()

	activeCountLabel := widget.NewLabel("사용 가능 0 / 전체 0")
	state.addAccessSummaryLabel(activeCountLabel)

	recentActivity := widget.NewLabel("최근 작업 없음")
	recentActivity.Wrapping = fyne.TextWrapWord
	state.addActivityLabel(recentActivity)

	openButton := widget.NewButtonWithIcon("브라우저 열기", theme.HomeIcon(), func() {
		if err := openBrowser(state.serverURL); err != nil {
			state.setStatus("브라우저 오류", "브라우저 열기 실패: "+err.Error())
			return
		}
		state.setStatus("브라우저 열기", "관리 화면을 브라우저로 열었습니다.")
	})
	copyAddressButton := copyButton("주소 복사", func() string { return state.serverURL }, state)
	copyShareButton := copyButton("내부망 주소 복사", func() string { return accessURLText(state.shareURLs) }, state)
	refreshButton := widget.NewButtonWithIcon("상태 새로고침", theme.ViewRefreshIcon(), func() {
		state.refreshAccessCodes()
		state.setStatus("상태 새로고침", "대시보드 상태를 새로고침했습니다.")
	})

	serverCard := widget.NewCard("서버 상태", "", container.NewVBox(
		statusIconLine(theme.InfoIcon(), statusTitle),
		statusDetail,
		widget.NewSeparator(),
		widget.NewLabel("접속 주소"),
		addressEntry,
		container.NewHBox(openButton, copyAddressButton, refreshButton),
	))

	networkCard := widget.NewCard("네트워크", "", container.NewVBox(
		infoLine("바인딩 IP", state.runtime.Config.Server.Host),
		infoLine("포트", strconv.Itoa(state.runtime.Config.Server.Port)),
		widget.NewSeparator(),
		widget.NewLabel("다른 PC 접속 주소"),
		buildAccessURLList(state.shareURLs),
		container.NewHBox(copyShareButton),
	))

	summaryCard := widget.NewCard("운영 요약", "", container.NewVBox(
		infoLine("접속 코드", activeCountLabel.Text),
		activeCountLabel,
		infoLine("최근 작업", recentActivity.Text),
		recentActivity,
	))

	grid := container.NewGridWithColumns(2, serverCard, networkCard)
	return container.NewBorder(nil, nil, nil, nil, container.NewVBox(grid, summaryCard))
}

func buildAccessURLList(urls []string) fyne.CanvasObject {
	if len(urls) == 0 {
		label := widget.NewLabel(accessURLText(urls))
		label.Wrapping = fyne.TextWrapWord
		return label
	}

	rows := make([]fyne.CanvasObject, 0, len(urls))
	for _, url := range urls {
		label := widget.NewLabel(url)
		label.TextStyle = fyne.TextStyle{Monospace: true}
		label.Wrapping = fyne.TextWrapOff
		rows = append(rows, container.NewHBox(widget.NewIcon(theme.ComputerIcon()), label))
	}
	return container.NewVBox(rows...)
}
