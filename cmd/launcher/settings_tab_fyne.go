//go:build fyne

package main

import (
	"strconv"

	"fyne.io/fyne/v2"
	"fyne.io/fyne/v2/container"
	"fyne.io/fyne/v2/theme"
	"fyne.io/fyne/v2/widget"
)

func buildSettingsTab(state *launcherState) fyne.CanvasObject {
	hostEntry := widget.NewEntry()
	hostEntry.SetText(state.runtime.Config.Server.Host)
	portEntry := widget.NewEntry()
	portEntry.SetText(strconv.Itoa(state.runtime.Config.Server.Port))
	displayNameEntry := widget.NewEntry()
	displayNameEntry.SetText(state.runtime.Config.App.DisplayName)
	logLevelEntry := widget.NewEntry()
	logLevelEntry.SetText(state.runtime.Config.Logging.Level)

	saveButton := widget.NewButtonWithIcon("저장", theme.DocumentSaveIcon(), func() {
		state.setStatus("설정 저장 준비 중", "서버 설정 저장은 후속 브랜치에서 연결합니다.")
	})
	restartNotice := widget.NewLabel("IP/포트 변경은 저장 후 앱 재시작이 필요합니다.")
	restartNotice.Wrapping = fyne.TextWrapWord

	serverForm := widget.NewForm(
		widget.NewFormItem("바인딩 IP", hostEntry),
		widget.NewFormItem("포트", portEntry),
		widget.NewFormItem("기관명", displayNameEntry),
		widget.NewFormItem("로그 레벨", logLevelEntry),
	)

	settingsCard := widget.NewCard("서버 설정", "", container.NewVBox(
		serverForm,
		restartNotice,
		container.NewHBox(saveButton),
	))

	brandingCard := placeholderCard("브랜딩",
		infoLine("기관명", state.runtime.Config.App.DisplayName),
		infoLine("영문명", state.runtime.Config.App.EnglishName),
		infoLine("로고", "준비 중"),
	)

	return container.NewBorder(nil, nil, nil, nil, container.NewVBox(settingsCard, brandingCard))
}
