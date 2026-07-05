//go:build fyne

package main

import (
	"context"
	"fmt"

	"fyne.io/fyne/v2"
	"fyne.io/fyne/v2/container"
	"fyne.io/fyne/v2/theme"
	"fyne.io/fyne/v2/widget"
)

func buildBackupSyncTab(state *launcherState) fyne.CanvasObject {
	backupStatus := widget.NewLabel("백업 상태를 불러오지 않았습니다.")
	backupStatus.Wrapping = fyne.TextWrapWord
	refreshBackupStatus := func() {
		status, err := state.runtime.Backups.Status(context.Background())
		if err != nil {
			backupStatus.SetText("백업 상태 조회 실패: " + err.Error())
			return
		}
		latest := "없음"
		if status.Latest != nil {
			latest = status.Latest.FileName
		}
		backupStatus.SetText(fmt.Sprintf("최근 백업 %s / 전체 %d개 / 보관 %d일", latest, status.TotalCount, status.KeepDays))
	}

	createBackupButton := widget.NewButtonWithIcon("수동 백업 생성", theme.DocumentSaveIcon(), func() {
		created, err := state.runtime.Backups.CreateBackup(context.Background())
		if err != nil {
			state.setStatus("백업 실패", "수동 백업 생성 실패: "+err.Error())
			return
		}
		if _, err := state.runtime.Backups.PruneOldBackups(context.Background()); err != nil {
			state.setStatus("백업 정리 실패", "백업 생성 후 오래된 백업 정리 실패: "+err.Error())
			return
		}
		refreshBackupStatus()
		state.setStatus("백업 완료", "수동 백업을 생성했습니다: "+created.FileName)
	})
	refreshButton := widget.NewButtonWithIcon("상태 새로고침", theme.ViewRefreshIcon(), refreshBackupStatus)

	backupCard := widget.NewCard("DB 백업", "", container.NewVBox(
		infoLine("DB 경로", state.runtime.Config.Database.Path),
		infoLine("백업 경로", state.runtime.Config.Backup.Path),
		backupStatus,
		container.NewHBox(createBackupButton, refreshButton),
	))

	syncCard := placeholderCard("외부 동기화",
		infoLine("Google Drive", "실험실에서 검증 후 승격"),
		infoLine("메일 백업", "실험실에서 검증 후 승격"),
		infoLine("자동 백업 주기", "후속 설정 항목"),
	)

	refreshBackupStatus()
	return container.NewBorder(nil, nil, nil, nil, container.NewVBox(backupCard, syncCard))
}
