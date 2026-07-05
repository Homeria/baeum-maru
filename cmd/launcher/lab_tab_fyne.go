//go:build fyne

package main

import (
	"fyne.io/fyne/v2"
	"fyne.io/fyne/v2/container"
	"fyne.io/fyne/v2/widget"
)

func buildLabTab(state *launcherState) fyne.CanvasObject {
	driveCheck := widget.NewCheck("Google Drive 백업 연동", nil)
	mailCheck := widget.NewCheck("메일 API 백업 전송", nil)
	reportCheck := widget.NewCheck("운영 현황 알림", nil)
	schedulerCheck := widget.NewCheck("예약 작업 스케줄러", nil)
	for _, item := range []*widget.Check{driveCheck, mailCheck, reportCheck, schedulerCheck} {
		item.Disable()
	}

	experimentCard := widget.NewCard("실험 기능", "", container.NewVBox(
		driveCheck,
		mailCheck,
		reportCheck,
		schedulerCheck,
	))

	statusCard := placeholderCard("연동 상태",
		infoLine("Google Drive", "미연결"),
		infoLine("메일 API", "미연결"),
		infoLine("현황 알림", "미사용"),
	)

	note := widget.NewLabel("실험 기능은 검증 후 백업/동기화 또는 서버 설정 탭으로 이동합니다.")
	note.Wrapping = fyne.TextWrapWord
	return container.NewBorder(nil, note, nil, nil, container.NewVBox(experimentCard, statusCard))
}
