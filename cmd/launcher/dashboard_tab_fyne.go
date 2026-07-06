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
	serverStateLabel := widget.NewLabelWithStyle("정지", fyne.TextAlignLeading, fyne.TextStyle{Bold: true})
	serverDetailLabel := widget.NewLabel("서버가 정지되어 있습니다.")
	serverDetailLabel.Wrapping = fyne.TextWrapWord
	state.addServerStatusLabels(serverStateLabel, serverDetailLabel)

	addressEntry := widget.NewEntry()
	addressEntry.Disable()
	state.addServerURLEntry(addressEntry)

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
	startButton := widget.NewButtonWithIcon("서버 시작", theme.MediaPlayIcon(), func() {
		state.serverController.Start()
	})
	stopButton := widget.NewButtonWithIcon("서버 중지", theme.MediaStopIcon(), func() {
		state.serverController.Stop()
	})
	restartButton := widget.NewButtonWithIcon("서버 재시작", theme.ViewRefreshIcon(), func() {
		state.serverController.Restart()
	})
	refreshButton := widget.NewButtonWithIcon("상태 새로고침", theme.ViewRefreshIcon(), func() {
		state.refreshAccessCodes()
		state.setStatus("상태 새로고침", "대시보드 상태를 새로고침했습니다.")
	})
	state.addServerActionHook(func(status launcherServerStatus) {
		if status.CanStart() {
			startButton.Enable()
		} else {
			startButton.Disable()
		}
		if status.CanStop() {
			stopButton.Enable()
		} else {
			stopButton.Disable()
		}
		if status.CanRestart() {
			restartButton.Enable()
		} else {
			restartButton.Disable()
		}
		if status == launcherServerRunning {
			openButton.Enable()
		} else {
			openButton.Disable()
		}
	})

	serverCard := widget.NewCard("서버 상태", "", container.NewVBox(
		statusIconLine(theme.InfoIcon(), serverStateLabel),
		serverDetailLabel,
		widget.NewSeparator(),
		container.NewHBox(startButton, stopButton, restartButton),
		widget.NewSeparator(),
		widget.NewLabel("접속 주소"),
		addressEntry,
		container.NewHBox(openButton, copyAddressButton, refreshButton),
	))

	hostLabel := widget.NewLabel(state.runtime.Config.Server.Host)
	hostLabel.Wrapping = fyne.TextWrapWord
	portLabel := widget.NewLabel(strconv.Itoa(state.runtime.Config.Server.Port))
	state.addNetworkLabels(hostLabel, portLabel)
	accessURLList := buildAccessURLList(state.shareURLs)
	state.addNetworkURLList(accessURLList)

	networkCard := widget.NewCard("네트워크", "", container.NewVBox(
		widget.NewLabelWithStyle("바인딩 IP", fyne.TextAlignLeading, fyne.TextStyle{Bold: true}),
		hostLabel,
		widget.NewLabelWithStyle("포트", fyne.TextAlignLeading, fyne.TextStyle{Bold: true}),
		portLabel,
		widget.NewSeparator(),
		widget.NewLabel("다른 PC 접속 주소"),
		accessURLList,
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

func buildAccessURLList(urls []string) *fyne.Container {
	return container.NewVBox(accessURLRows(urls)...)
}

func accessURLRows(urls []string) []fyne.CanvasObject {
	if len(urls) == 0 {
		label := widget.NewLabel(accessURLText(urls))
		label.Wrapping = fyne.TextWrapWord
		return []fyne.CanvasObject{label}
	}

	rows := make([]fyne.CanvasObject, 0, len(urls))
	for _, url := range urls {
		label := widget.NewLabel(url)
		label.TextStyle = fyne.TextStyle{Monospace: true}
		label.Wrapping = fyne.TextWrapOff
		rows = append(rows, container.NewHBox(widget.NewIcon(theme.ComputerIcon()), label))
	}
	return rows
}
