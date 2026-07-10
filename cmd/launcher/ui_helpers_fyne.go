//go:build fyne

package main

import (
	"fmt"
	"os/exec"
	"strings"
	"time"

	"fyne.io/fyne/v2"
	"fyne.io/fyne/v2/container"
	"fyne.io/fyne/v2/theme"
	"fyne.io/fyne/v2/widget"

	"github.com/Homeria/baeum-maru/internal/app"
	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/service"
)

func buildLauncherHeader(displayName string) fyne.CanvasObject {
	title := widget.NewLabelWithStyle(displayName, fyne.TextAlignLeading, fyne.TextStyle{Bold: true})
	subtitle := widget.NewLabel("내부망 수강신청 운영 콘솔")
	subtitle.Wrapping = fyne.TextWrapWord
	versionLabel := widget.NewLabel("v" + app.Version)
	return container.NewBorder(nil, nil, nil, versionLabel, container.NewVBox(title, subtitle))
}

func infoLine(label string, value string) fyne.CanvasObject {
	name := widget.NewLabel(label)
	name.TextStyle = fyne.TextStyle{Bold: true}
	valueLabel := widget.NewLabel(value)
	valueLabel.Wrapping = fyne.TextWrapWord
	return container.NewBorder(nil, nil, name, nil, valueLabel)
}

func placeholderCard(title string, rows ...fyne.CanvasObject) fyne.CanvasObject {
	content := []fyne.CanvasObject{}
	content = append(content, rows...)
	if len(content) == 0 {
		content = append(content, widget.NewLabel("준비 중"))
	}
	return widget.NewCard(title, "", container.NewVBox(content...))
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

func roleDescription(role string) string {
	switch role {
	case domain.UserRoleStaff:
		return "직원: 회원, 강좌, 접수, 추첨, 엑셀, 출석 업무를 수행합니다."
	case domain.UserRoleViewer:
		return "조회 전용: 주요 목록과 출석 현황을 조회할 수 있고 저장 작업은 제한됩니다."
	default:
		return "임시 직원: 접수와 출석 입력 중심으로 제한된 업무를 수행합니다."
	}
}

func expirationDescription(selected string) string {
	switch selected {
	case "4시간":
		return "발급 후 4시간 동안 사용할 수 있습니다."
	case "8시간":
		return "발급 후 8시간 동안 사용할 수 있습니다."
	case "3일":
		return "발급 후 3일 동안 사용할 수 있습니다."
	default:
		return "오늘 자정까지 사용할 수 있습니다."
	}
}

func accessCodeTitle(summary service.AccessCodeSummary) string {
	label := strings.TrimSpace(summary.Label)
	if label == "" {
		label = "발급명 없음"
	}
	return fmt.Sprintf("#%d  %s  /  %s  /  %s", summary.ID, summary.DisplayName, roleLabel(summary.Role), statusLabelText(summary.Status)) + "  ·  " + label
}

func compactAccessCodeTitle(summary service.AccessCodeSummary) string {
	return fmt.Sprintf("%s  ·  %s  ·  %s", summary.DisplayName, roleLabel(summary.Role), statusLabelText(summary.Status))
}

func compactAccessCodeMeta(summary service.AccessCodeSummary) string {
	affiliation := strings.TrimSpace(summary.Affiliation)
	if affiliation == "" {
		affiliation = "소속/위치 없음"
	}
	lastUsed := emptyDash(shortTime(summary.LastUsedAt))
	return affiliation + "  ·  만료 " + shortTime(summary.ExpiresAt) + "  ·  최근 사용 " + lastUsed
}

func accessCodeMeta(summary service.AccessCodeSummary) string {
	affiliation := strings.TrimSpace(summary.Affiliation)
	if affiliation == "" {
		affiliation = "소속/위치 없음"
	}
	return affiliation + "  ·  만료 " + shortTime(summary.ExpiresAt)
}

func accessCodeTimeline(summary service.AccessCodeSummary) string {
	return "발급 " + shortTime(summary.IssuedAt) + "  ·  마지막 사용 " + emptyDash(shortTime(summary.LastUsedAt))
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

func statusIconLine(icon fyne.Resource, label *widget.Label) fyne.CanvasObject {
	return container.NewHBox(widget.NewIcon(icon), label)
}

func copyButton(label string, content func() string, state *launcherState) *widget.Button {
	return widget.NewButtonWithIcon(label, theme.ContentCopyIcon(), func() {
		state.copyToClipboard(label, content())
	})
}
