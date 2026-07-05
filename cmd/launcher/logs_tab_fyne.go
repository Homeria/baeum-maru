//go:build fyne

package main

import (
	"fyne.io/fyne/v2"
	"fyne.io/fyne/v2/container"
	"fyne.io/fyne/v2/theme"
	"fyne.io/fyne/v2/widget"
)

func buildLogsTab(state *launcherState) fyne.CanvasObject {
	activityLabel := widget.NewLabel("최근 작업 없음")
	activityLabel.Wrapping = fyne.TextWrapWord
	state.addActivityLabel(activityLabel)

	launcherLog := widget.NewLabel("")
	launcherLog.Wrapping = fyne.TextWrapWord
	state.addLogLabel(launcherLog)
	launcherScroll := container.NewScroll(launcherLog)
	launcherScroll.SetMinSize(fyne.NewSize(420, 420))

	launcherPanel := container.NewBorder(
		container.NewVBox(widget.NewLabelWithStyle("런처 작업 로그", fyne.TextAlignLeading, fyne.TextStyle{Bold: true}), activityLabel, widget.NewSeparator()),
		nil,
		nil,
		nil,
		launcherScroll,
	)

	authPanel := placeholderCard("인증/접속 로그",
		infoLine("상태", "준비 중"),
		infoLine("대상", "접속 코드 로그인, 마지막 사용 시각, 만료/폐기 이벤트"),
	)
	auditPanel := placeholderCard("업무 감사 로그",
		infoLine("상태", "준비 중"),
		infoLine("대상", "회원, 강좌, 신청, 출석, 백업 작업 기록"),
	)
	errorPanel := placeholderCard("오류/경고",
		infoLine("상태", "준비 중"),
		infoLine("대상", "서버 시작 실패, 백업 실패, 외부 연동 실패"),
	)

	tabs := container.NewAppTabs(
		container.NewTabItemWithIcon("런처", theme.HistoryIcon(), launcherPanel),
		container.NewTabItemWithIcon("인증/접속", theme.VisibilityIcon(), authPanel),
		container.NewTabItemWithIcon("감사", theme.DocumentIcon(), auditPanel),
		container.NewTabItemWithIcon("오류", theme.WarningIcon(), errorPanel),
	)
	tabs.SetTabLocation(container.TabLocationTop)
	return tabs
}
