//go:build fyne

package main

import (
	"context"
	"net"
	"strconv"
	"strings"

	"fyne.io/fyne/v2"
	"fyne.io/fyne/v2/container"
	"fyne.io/fyne/v2/theme"
	"fyne.io/fyne/v2/widget"

	"github.com/Homeria/baeum-maru/internal/service"
)

const (
	networkModeLocal  = "이 컴퓨터만"
	networkModeLAN    = "내부망 허용"
	networkModeCustom = "직접 입력"
)

func buildSettingsTab(state *launcherState) fyne.CanvasObject {
	host := strings.TrimSpace(state.runtime.Config.Server.Host)
	mode := networkModeForHost(host)

	modeRadio := widget.NewRadioGroup([]string{networkModeLAN, networkModeLocal, networkModeCustom}, nil)
	modeRadio.Horizontal = false
	modeRadio.SetSelected(mode)

	customHostEntry := widget.NewEntry()
	customHostEntry.SetPlaceHolder("예: 192.168.0.10")
	if mode == networkModeCustom {
		customHostEntry.SetText(host)
	} else {
		customHostEntry.SetText("")
		customHostEntry.Disable()
	}

	portEntry := widget.NewEntry()
	portEntry.SetText(strconv.Itoa(state.runtime.Config.Server.Port))
	portEntry.SetPlaceHolder("예: 18080")

	currentURL := widget.NewLabel(state.serverURL)
	currentURL.TextStyle = fyne.TextStyle{Monospace: true}
	currentURL.Wrapping = fyne.TextWrapOff
	state.addServerActionHook(func(launcherServerStatus) {
		currentURL.SetText(state.serverURL)
	})

	modeRadio.OnChanged = func(selected string) {
		if selected == networkModeCustom {
			customHostEntry.Enable()
		} else {
			customHostEntry.Disable()
		}
	}

	notice := widget.NewLabel("서버가 정지된 상태에서만 네트워크 설정을 저장할 수 있습니다. 저장 후 서버를 다시 시작하면 새 접속 주소가 적용됩니다.")
	notice.Wrapping = fyne.TextWrapWord

	saveButton := widget.NewButtonWithIcon("네트워크 설정 저장", theme.DocumentSaveIcon(), func() {
		if state.serverStatus != launcherServerStopped && state.serverStatus != launcherServerError {
			state.setStatus("설정 저장 제한", "서버를 중지한 뒤 네트워크 설정을 변경하세요.")
			return
		}

		port, err := strconv.Atoi(strings.TrimSpace(portEntry.Text))
		if err != nil || port < 1 || port > 65535 {
			state.setStatus("설정 오류", "포트는 1부터 65535 사이의 숫자로 입력하세요.")
			return
		}

		host, err := hostForNetworkMode(modeRadio.Selected, customHostEntry.Text)
		if err != nil {
			state.setStatus("설정 오류", err.Error())
			return
		}

		updated, err := state.runtime.Settings.Update(context.Background(), service.SettingsInput{
			ServerHost:         host,
			ServerPort:         port,
			BackupKeepDays:     state.runtime.Config.Backup.KeepDays,
			OpenBrowserOnStart: state.runtime.Config.UI.OpenBrowserOnStart,
		})
		if err != nil {
			state.setStatus("설정 저장 실패", err.Error())
			return
		}

		state.runtime.Config = updated
		state.updateNetworkConfig(updated.Server.Host, updated.Server.Port)
		currentURL.SetText(state.serverURL)
		state.setStatus("네트워크 설정 저장", "네트워크 설정을 저장했습니다. 서버 시작 시 새 설정이 적용됩니다.")
	})

	setControlsEnabled := func(enabled bool) {
		if enabled {
			modeRadio.Enable()
			portEntry.Enable()
			saveButton.Enable()
			if modeRadio.Selected == networkModeCustom {
				customHostEntry.Enable()
			} else {
				customHostEntry.Disable()
			}
			return
		}
		modeRadio.Disable()
		customHostEntry.Disable()
		portEntry.Disable()
		saveButton.Disable()
	}
	state.addServerActionHook(func(status launcherServerStatus) {
		setControlsEnabled(status == launcherServerStopped || status == launcherServerError)
	})

	networkCard := widget.NewCard("접속 방식", "", container.NewVBox(
		widget.NewLabel("바인딩 모드"),
		modeRadio,
		widget.NewSeparator(),
		widget.NewForm(
			widget.NewFormItem("직접 입력 IP", customHostEntry),
			widget.NewFormItem("포트", portEntry),
		),
		notice,
		container.NewHBox(saveButton),
	))

	settingsURLList := buildAccessURLList(state.shareURLs)
	state.addNetworkURLList(settingsURLList)

	addressCard := widget.NewCard("현재 접속 주소", "", container.NewVBox(
		widget.NewLabelWithStyle("이 컴퓨터", fyne.TextAlignLeading, fyne.TextStyle{Bold: true}),
		currentURL,
		widget.NewSeparator(),
		widget.NewLabelWithStyle("다른 PC 접속 주소", fyne.TextAlignLeading, fyne.TextStyle{Bold: true}),
		settingsURLList,
	))

	infoCard := widget.NewCard("모드 기준", "", container.NewVBox(
		infoLine("내부망 허용", "같은 와이파이/핫스팟의 다른 PC에서 접속할 수 있도록 0.0.0.0으로 엽니다."),
		infoLine("이 컴퓨터만", "런처를 실행한 PC에서만 접속할 수 있도록 127.0.0.1로 엽니다."),
		infoLine("직접 입력", "특정 네트워크 어댑터 IP로만 서버를 엽니다."),
	))

	return container.NewBorder(nil, nil, nil, nil, container.NewVBox(networkCard, addressCard, infoCard))
}

func networkModeForHost(host string) string {
	switch strings.TrimSpace(host) {
	case "", "0.0.0.0", "::":
		return networkModeLAN
	case "127.0.0.1", "localhost", "::1":
		return networkModeLocal
	default:
		return networkModeCustom
	}
}

func hostForNetworkMode(mode string, customHost string) (string, error) {
	switch mode {
	case networkModeLocal:
		return "127.0.0.1", nil
	case networkModeLAN, "":
		return "0.0.0.0", nil
	case networkModeCustom:
		host := strings.TrimSpace(customHost)
		if host == "" {
			return "", errString("직접 입력 IP를 입력하세요.")
		}
		if net.ParseIP(host) == nil {
			return "", errString("직접 입력은 IP 주소만 사용할 수 있습니다.")
		}
		return host, nil
	default:
		return "", errString("알 수 없는 바인딩 모드입니다.")
	}
}

type errString string

func (e errString) Error() string {
	return string(e)
}
