//go:build fyne

package main

import (
	"fyne.io/fyne/v2"
	"fyne.io/fyne/v2/container"
	"fyne.io/fyne/v2/theme"
)

func buildLauncherTabs(state *launcherState) fyne.CanvasObject {
	tabs := container.NewAppTabs(
		container.NewTabItemWithIcon("대시보드", theme.HomeIcon(), buildDashboardTab(state)),
		container.NewTabItemWithIcon("접속 관리", theme.VisibilityIcon(), buildAccessTab(state)),
		container.NewTabItemWithIcon("로그", theme.HistoryIcon(), buildLogsTab(state)),
		container.NewTabItemWithIcon("서버 설정", theme.SettingsIcon(), buildSettingsTab(state)),
		container.NewTabItemWithIcon("백업/동기화", theme.FolderOpenIcon(), buildBackupSyncTab(state)),
		container.NewTabItemWithIcon("실험실", theme.WarningIcon(), buildLabTab(state)),
	)
	tabs.SetTabLocation(container.TabLocationTop)
	return tabs
}
