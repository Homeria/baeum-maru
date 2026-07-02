//go:build fyne

package main

import (
	"context"
	"fmt"
	"os"
	"os/exec"
	"strings"
	"time"

	"fyne.io/fyne/v2"
	fyneapp "fyne.io/fyne/v2/app"
	"fyne.io/fyne/v2/container"
	"fyne.io/fyne/v2/layout"
	"fyne.io/fyne/v2/widget"

	"github.com/Homeria/baeum-maru/internal/app"
	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/server"
	"github.com/Homeria/baeum-maru/internal/service"
	"github.com/Homeria/baeum-maru/internal/web"
)

func main() {
	runtime, err := app.Bootstrap("")
	if err != nil {
		fmt.Fprintf(os.Stderr, "launcher startup failed: %v\n", err)
		os.Exit(1)
	}

	router := web.NewRouter(web.RouterOptions{
		DisplayName:   runtime.Config.App.DisplayName,
		Version:       app.Version,
		Logger:        runtime.Logger.Logger,
		Auth:          runtime.Config.Auth,
		Authenticator: runtime.AccessAuth,
		Members:       runtime.Members,
		Courses:       runtime.Courses,
		Registrations: runtime.Registrations,
		Lotteries:     runtime.Lotteries,
		Exports:       runtime.Exports,
		Imports:       runtime.Imports,
		Backups:       runtime.Backups,
		Attendance:    runtime.Attendance,
		Settings:      runtime.Settings,
		Audits:        runtime.Audits,
	})
	httpServer := server.New(server.Options{
		Host:    runtime.Config.Server.Host,
		Port:    runtime.Config.Server.Port,
		Handler: router,
		Logger:  runtime.Logger.Logger,
	})

	guiApp := fyneapp.NewWithID("github.com.Homeria.baeum-maru.launcher")
	window := guiApp.NewWindow(runtime.Config.App.DisplayName + " 런처")
	window.Resize(fyne.NewSize(900, 620))

	url := browserURL(runtime.Config.Server.Host, runtime.Config.Server.Port)
	statusLabel := widget.NewLabel("서버 시작 중")
	addressEntry := widget.NewEntry()
	addressEntry.SetText(url)
	addressEntry.Disable()

	var summaries []service.AccessCodeSummary
	var selectedAccessCodeID int64
	codeList := widget.NewList(
		func() int { return len(summaries) },
		func() fyne.CanvasObject {
			title := widget.NewLabel("")
			meta := widget.NewLabel("")
			meta.Wrapping = fyne.TextWrapWord
			return container.NewVBox(title, meta)
		},
		func(id widget.ListItemID, item fyne.CanvasObject) {
			box := item.(*fyne.Container)
			title := box.Objects[0].(*widget.Label)
			meta := box.Objects[1].(*widget.Label)
			summary := summaries[id]
			title.SetText(fmt.Sprintf("%s / %s / %s", summary.DisplayName, roleLabel(summary.Role), statusLabelText(summary.Status)))
			meta.SetText(fmt.Sprintf("ID #%d  만료 %s  마지막 사용 %s", summary.ID, shortTime(summary.ExpiresAt), emptyDash(shortTime(summary.LastUsedAt))))
		},
	)
	codeList.OnSelected = func(id widget.ListItemID) {
		if id >= 0 && id < len(summaries) {
			selectedAccessCodeID = summaries[id].ID
		}
	}

	refreshCodes := func() {
		items, err := runtime.AccessAuth.ListRecentAccessCodes(context.Background(), 100)
		if err != nil {
			statusLabel.SetText("접속 코드 목록 조회 실패: " + err.Error())
			return
		}
		summaries = items
		selectedAccessCodeID = 0
		codeList.UnselectAll()
		codeList.Refresh()
	}

	go func() {
		if err := httpServer.Start(); err != nil {
			fyne.Do(func() {
				statusLabel.SetText("서버 오류: " + err.Error())
			})
		}
	}()
	statusLabel.SetText("서버 실행 중")

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

	issuedCode := widget.NewEntry()
	issuedCode.Disable()

	issueButton := widget.NewButton("접속 코드 발급", func() {
		expiresAt := resolveExpiration(durationSelect.Selected)
		issued, err := runtime.AccessAuth.IssueAccessCode(context.Background(), service.AccessCodeIssueInput{
			DisplayName: displayName.Text,
			Affiliation: affiliation.Text,
			ContactNote: contactNote.Text,
			Role:        selectedRole(roleSelect.Selected),
			ExpiresAt:   expiresAt,
			Label:       labelEntry.Text,
		})
		if err != nil {
			statusLabel.SetText("발급 실패: " + err.Error())
			return
		}
		issuedCode.Enable()
		issuedCode.SetText(issued.Code)
		issuedCode.Disable()
		guiApp.Clipboard().SetContent(issued.Code)
		statusLabel.SetText("접속 코드를 발급하고 클립보드에 복사했습니다.")
		refreshCodes()
	})

	revokeButton := widget.NewButton("선택 코드 폐기", func() {
		if selectedAccessCodeID <= 0 {
			statusLabel.SetText("폐기할 접속 코드를 선택하세요.")
			return
		}
		if err := runtime.AccessAuth.RevokeAccessCode(context.Background(), selectedAccessCodeID); err != nil {
			statusLabel.SetText("폐기 실패: " + err.Error())
			return
		}
		statusLabel.SetText("접속 코드를 폐기했습니다.")
		refreshCodes()
	})

	openButton := widget.NewButton("브라우저 열기", func() {
		if err := openBrowser(url); err != nil {
			statusLabel.SetText("브라우저 열기 실패: " + err.Error())
			return
		}
	})
	copyAddressButton := widget.NewButton("주소 복사", func() {
		guiApp.Clipboard().SetContent(url)
		statusLabel.SetText("접속 주소를 클립보드에 복사했습니다.")
	})
	copyCodeButton := widget.NewButton("코드 복사", func() {
		if strings.TrimSpace(issuedCode.Text) == "" {
			statusLabel.SetText("복사할 발급 코드가 없습니다.")
			return
		}
		guiApp.Clipboard().SetContent(issuedCode.Text)
		statusLabel.SetText("접속 코드를 클립보드에 복사했습니다.")
	})

	serverPanel := widget.NewCard("서버", "", container.NewVBox(
		statusLabel,
		container.NewBorder(nil, nil, widget.NewLabel("접속 주소"), nil, addressEntry),
		container.NewHBox(openButton, copyAddressButton),
	))

	issueForm := widget.NewForm(
		widget.NewFormItem("이름", displayName),
		widget.NewFormItem("소속/위치", affiliation),
		widget.NewFormItem("식별 메모", contactNote),
		widget.NewFormItem("역할", roleSelect),
		widget.NewFormItem("유효 시간", durationSelect),
		widget.NewFormItem("발급명", labelEntry),
	)
	issuePanel := widget.NewCard("접속 코드 발급", "", container.NewVBox(
		issueForm,
		issueButton,
		widget.NewSeparator(),
		widget.NewLabel("최근 발급 코드"),
		container.NewBorder(nil, nil, nil, copyCodeButton, issuedCode),
	))

	listPanel := widget.NewCard("최근 접속 코드", "", container.NewBorder(
		nil,
		container.NewHBox(widget.NewButton("새로고침", refreshCodes), revokeButton),
		nil,
		nil,
		codeList,
	))

	content := container.NewBorder(
		container.NewVBox(widget.NewLabelWithStyle(runtime.Config.App.DisplayName, fyne.TextAlignLeading, fyne.TextStyle{Bold: true}), serverPanel),
		nil,
		nil,
		nil,
		container.NewHSplit(issuePanel, listPanel),
	)
	window.SetContent(container.New(layout.NewPaddedLayout(), content))
	window.SetCloseIntercept(func() {
		shutdownCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		if err := httpServer.Shutdown(shutdownCtx); err != nil {
			runtime.Logger.Warn("launcher shutdown failed", "error", err)
		}
		if err := runtime.Close(); err != nil {
			fmt.Fprintf(os.Stderr, "runtime close failed: %v\n", err)
		}
		window.Close()
		guiApp.Quit()
	})

	refreshCodes()
	if runtime.Config.UI.OpenBrowserOnStart {
		_ = openBrowser(url)
	}
	window.ShowAndRun()
}

func selectedRole(label string) string {
	switch label {
	case "직원":
		return domain.UserRoleStaff
	case "조회 전용":
		return domain.UserRoleViewer
	default:
		return domain.UserRoleTemporaryStaff
	}
}

func roleLabel(role string) string {
	switch role {
	case domain.UserRoleStaff:
		return "직원"
	case domain.UserRoleViewer:
		return "조회 전용"
	default:
		return "임시 직원"
	}
}

func statusLabelText(status string) string {
	switch status {
	case domain.AccessCodeStatusRevoked:
		return "폐기"
	case domain.AccessCodeStatusExpired:
		return "만료"
	default:
		return "사용 가능"
	}
}

func resolveExpiration(selected string) time.Time {
	now := time.Now()
	switch selected {
	case "4시간":
		return now.Add(4 * time.Hour)
	case "8시간":
		return now.Add(8 * time.Hour)
	case "3일":
		return now.Add(72 * time.Hour)
	default:
		return time.Date(now.Year(), now.Month(), now.Day()+1, 0, 0, 0, 0, now.Location())
	}
}

func shortTime(value string) string {
	if value == "" {
		return ""
	}
	parsed, err := time.Parse(time.RFC3339, value)
	if err != nil {
		return value
	}
	return parsed.Local().Format("2006-01-02 15:04")
}

func emptyDash(value string) string {
	if strings.TrimSpace(value) == "" {
		return "-"
	}
	return value
}

func openBrowser(url string) error {
	return exec.Command("rundll32", "url.dll,FileProtocolHandler", url).Start()
}

func browserURL(host string, port int) string {
	if host == "" || host == "0.0.0.0" || host == "::" {
		host = "127.0.0.1"
	}
	return fmt.Sprintf("http://%s:%d", host, port)
}
