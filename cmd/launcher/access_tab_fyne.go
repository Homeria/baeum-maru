//go:build fyne

package main

import (
	"context"
	"strings"

	"fyne.io/fyne/v2"
	"fyne.io/fyne/v2/container"
	"fyne.io/fyne/v2/theme"
	"fyne.io/fyne/v2/widget"

	"github.com/Homeria/baeum-maru/internal/service"
)

func buildAccessTab(state *launcherState) fyne.CanvasObject {
	displayName := widget.NewEntry()
	displayName.SetPlaceHolder("예: 김접수")
	affiliation := widget.NewEntry()
	affiliation.SetPlaceHolder("예: 1층 접수대")
	contactNote := widget.NewEntry()
	contactNote.SetPlaceHolder("예: 휴대폰 끝 4자리 또는 메모")
	roleSelect := widget.NewSelect([]string{"직원", "임시 직원", "조회 전용"}, nil)
	roleSelect.SetSelected("임시 직원")
	durationSelect := widget.NewSelect([]string{"4시간", "8시간", "오늘 자정까지", "3일"}, nil)
	durationSelect.SetSelected("오늘 자정까지")
	labelEntry := widget.NewEntry()
	labelEntry.SetPlaceHolder("예: 오전 접수")

	roleHint := widget.NewLabel(roleDescription(selectedRole(roleSelect.Selected)))
	roleHint.Wrapping = fyne.TextWrapWord
	durationHint := widget.NewLabel(expirationDescription(durationSelect.Selected))
	durationHint.Wrapping = fyne.TextWrapWord
	roleSelect.OnChanged = func(selected string) {
		roleHint.SetText(roleDescription(selectedRole(selected)))
	}
	durationSelect.OnChanged = func(selected string) {
		durationHint.SetText(expirationDescription(selected))
	}

	issuedCode := widget.NewEntry()
	issuedCode.Disable()

	issueButton := widget.NewButtonWithIcon("접속 코드 발급", theme.ContentAddIcon(), func() {
		issued, err := state.runtime.AccessAuth.IssueAccessCode(context.Background(), service.AccessCodeIssueInput{
			DisplayName: displayName.Text,
			Affiliation: affiliation.Text,
			ContactNote: contactNote.Text,
			Role:        selectedRole(roleSelect.Selected),
			ExpiresAt:   resolveExpiration(durationSelect.Selected),
			Label:       labelEntry.Text,
		})
		if err != nil {
			state.setStatus("발급 실패", "접속 코드 발급 실패: "+err.Error())
			return
		}
		issuedCode.Enable()
		issuedCode.SetText(issued.Code)
		issuedCode.Disable()
		state.guiApp.Clipboard().SetContent(issued.Code)
		state.setStatus("코드 발급 완료", "접속 코드를 발급하고 클립보드에 복사했습니다. "+issued.User.DisplayName+" / "+roleLabel(issued.User.Role))
		state.refreshAccessCodes()
	})

	copyCodeButton := widget.NewButtonWithIcon("코드 복사", theme.ContentCopyIcon(), func() {
		if strings.TrimSpace(issuedCode.Text) == "" {
			state.setStatus("코드 없음", "복사할 발급 코드가 없습니다.")
			return
		}
		state.copyToClipboard("코드 복사 완료", issuedCode.Text)
	})

	issueForm := widget.NewForm(
		widget.NewFormItem("이름", displayName),
		widget.NewFormItem("소속/위치", affiliation),
		widget.NewFormItem("연락 메모", contactNote),
		widget.NewFormItem("역할", roleSelect),
		widget.NewFormItem("유효 시간", durationSelect),
		widget.NewFormItem("발급명", labelEntry),
	)
	issuePanel := widget.NewCard("접속 코드 발급", "", container.NewVBox(
		issueForm,
		roleHint,
		durationHint,
		issueButton,
		widget.NewSeparator(),
		widget.NewLabel("최근 발급 코드"),
		container.NewBorder(nil, nil, nil, copyCodeButton, issuedCode),
	))

	activeCountLabel := widget.NewLabel("사용 가능 0 / 전체 0")
	state.addAccessSummaryLabel(activeCountLabel)
	selectedCodeLabel := widget.NewLabel("선택된 코드가 없습니다.")
	selectedCodeLabel.Wrapping = fyne.TextWrapWord
	state.addSelectedCodeLabel(selectedCodeLabel)

	detailCode := widget.NewLabel("선택된 코드가 없습니다.")
	detailCode.TextStyle = fyne.TextStyle{Bold: true, Monospace: true}
	detailCode.Wrapping = fyne.TextWrapWord
	detailName := widget.NewLabel("-")
	detailRole := widget.NewLabel("-")
	detailStatus := widget.NewLabel("-")
	detailAffiliation := widget.NewLabel("-")
	detailLabel := widget.NewLabel("-")
	detailIssued := widget.NewLabel("-")
	detailExpires := widget.NewLabel("-")
	detailLastUsed := widget.NewLabel("-")

	resetDetails := func() {
		detailCode.SetText("선택된 코드가 없습니다.")
		detailName.SetText("-")
		detailRole.SetText("-")
		detailStatus.SetText("-")
		detailAffiliation.SetText("-")
		detailLabel.SetText("-")
		detailIssued.SetText("-")
		detailExpires.SetText("-")
		detailLastUsed.SetText("-")
	}
	showDetails := func(summary service.AccessCodeSummary) {
		detailCode.SetText(displayAccessCode(summary))
		detailName.SetText(summary.DisplayName)
		detailRole.SetText(roleLabel(summary.Role))
		detailStatus.SetText(statusLabelText(summary.Status))
		detailAffiliation.SetText(emptyDash(summary.Affiliation))
		detailLabel.SetText(emptyDash(summary.Label))
		detailIssued.SetText(shortTime(summary.IssuedAt))
		detailExpires.SetText(shortTime(summary.ExpiresAt))
		detailLastUsed.SetText(emptyDash(shortTime(summary.LastUsedAt)))
	}

	codeList := buildAccessCodeList(state)
	codeList.OnSelected = func(id widget.ListItemID) {
		state.selectAccessCode(id)
		items := state.filteredAccessCodes()
		if id < 0 || id >= len(items) {
			resetDetails()
			return
		}
		showDetails(items[id])
	}
	state.addCodeList(codeList)

	filterTabs := container.NewAppTabs(
		container.NewTabItem("전체", widget.NewLabel("")),
		container.NewTabItem("사용 가능", widget.NewLabel("")),
		container.NewTabItem("만료", widget.NewLabel("")),
	)
	filterTabs.SetTabLocation(container.TabLocationTop)
	filterTabs.OnSelected = func(tab *container.TabItem) {
		switch tab.Text {
		case "사용 가능":
			state.setAccessFilter("active")
		case "만료":
			state.setAccessFilter("expired")
		default:
			state.setAccessFilter("all")
		}
		resetDetails()
	}

	refreshButton := widget.NewButtonWithIcon("새로고침", theme.ViewRefreshIcon(), func() {
		state.refreshAccessCodes()
		resetDetails()
	})
	extendSelect := widget.NewSelect([]string{"4시간", "8시간", "오늘 자정까지", "3일"}, nil)
	extendSelect.SetSelected("4시간")
	extendButton := widget.NewButtonWithIcon("연장", theme.HistoryIcon(), func() {
		if state.selectedAccessCodeID <= 0 {
			state.setStatus("코드 선택 필요", "연장할 접속 코드를 선택하세요.")
			return
		}
		if err := state.runtime.AccessAuth.ExtendAccessCode(context.Background(), state.selectedAccessCodeID, resolveExpiration(extendSelect.Selected)); err != nil {
			state.setStatus("연장 실패", "접속 코드 연장 실패: "+err.Error())
			return
		}
		state.setStatus("코드 연장 완료", "선택한 접속 코드의 만료 시각을 연장했습니다.")
		state.refreshAccessCodes()
		resetDetails()
	})
	revokeButton := widget.NewButtonWithIcon("폐기", theme.DeleteIcon(), func() {
		if state.selectedAccessCodeID <= 0 {
			state.setStatus("코드 선택 필요", "폐기할 접속 코드를 선택하세요.")
			return
		}
		if err := state.runtime.AccessAuth.RevokeAccessCode(context.Background(), state.selectedAccessCodeID); err != nil {
			state.setStatus("폐기 실패", "접속 코드 폐기 실패: "+err.Error())
			return
		}
		state.setStatus("코드 폐기 완료", "접속 코드를 폐기했습니다.")
		state.refreshAccessCodes()
		resetDetails()
	})
	copySelectedButton := widget.NewButtonWithIcon("선택 코드 복사", theme.ContentCopyIcon(), func() {
		for _, summary := range state.summaries {
			if summary.ID == state.selectedAccessCodeID {
				if strings.TrimSpace(summary.Code) == "" {
					state.setStatus("코드 표시 불가", "이전 버전에서 발급된 코드는 원문을 다시 표시할 수 없습니다.")
					return
				}
				state.copyToClipboard("선택 코드 복사 완료", summary.Code)
				return
			}
		}
		state.setStatus("코드 선택 필요", "복사할 접속 코드를 선택하세요.")
	})

	listPane := container.NewBorder(
		container.NewVBox(activeCountLabel, filterTabs),
		nil,
		nil,
		nil,
		codeList,
	)
	detailPane := container.NewBorder(
		container.NewVBox(
			widget.NewLabelWithStyle("상세 정보", fyne.TextAlignLeading, fyne.TextStyle{Bold: true}),
			detailCode,
			selectedCodeLabel,
			widget.NewSeparator(),
		),
		container.NewVBox(
			container.NewHBox(refreshButton, extendSelect),
			container.NewHBox(copySelectedButton, extendButton, revokeButton),
		),
		nil,
		nil,
		container.NewVBox(
			detailRow("이름", detailName),
			detailRow("역할", detailRole),
			detailRow("상태", detailStatus),
			detailRow("소속/위치", detailAffiliation),
			detailRow("발급명", detailLabel),
			detailRow("발급", detailIssued),
			detailRow("만료", detailExpires),
			detailRow("최근 사용", detailLastUsed),
		),
	)

	manageSplit := container.NewHSplit(listPane, detailPane)
	manageSplit.SetOffset(0.58)
	listPanel := widget.NewCard("접속 코드 현황", "", manageSplit)

	split := container.NewHSplit(issuePanel, listPanel)
	split.SetOffset(0.36)
	return split
}

func buildAccessCodeList(state *launcherState) *widget.List {
	list := widget.NewList(
		func() int { return len(state.filteredAccessCodes()) },
		func() fyne.CanvasObject {
			code := widget.NewLabel("")
			code.TextStyle = fyne.TextStyle{Bold: true, Monospace: true}
			title := widget.NewLabel("")
			meta := widget.NewLabel("")
			meta.Wrapping = fyne.TextWrapWord
			return container.NewVBox(code, title, meta)
		},
		func(id widget.ListItemID, item fyne.CanvasObject) {
			box := item.(*fyne.Container)
			code := box.Objects[0].(*widget.Label)
			title := box.Objects[1].(*widget.Label)
			meta := box.Objects[2].(*widget.Label)
			summary := state.filteredAccessCodes()[id]
			code.SetText(displayAccessCode(summary))
			title.SetText(compactAccessCodeTitle(summary))
			meta.SetText(compactAccessCodeMeta(summary))
		},
	)
	list.OnSelected = state.selectAccessCode
	return list
}

func displayAccessCode(summary service.AccessCodeSummary) string {
	if strings.TrimSpace(summary.Code) == "" {
		return "이전 발급 코드"
	}
	return summary.Code
}

func detailRow(title string, value *widget.Label) fyne.CanvasObject {
	name := widget.NewLabel(title)
	name.TextStyle = fyne.TextStyle{Bold: true}
	value.Wrapping = fyne.TextWrapWord
	return container.NewBorder(nil, nil, name, nil, value)
}
