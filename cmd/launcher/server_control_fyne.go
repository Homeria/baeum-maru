//go:build fyne

package main

import (
	"fyne.io/fyne/v2"

	"github.com/Homeria/baeum-maru/internal/app"
	launchercore "github.com/Homeria/baeum-maru/internal/launcher"
)

type launcherServerStatus = launchercore.ServerStatus
type launcherServerController = launchercore.ServerController

const (
	launcherServerStopped  = launchercore.ServerStopped
	launcherServerStarting = launchercore.ServerStarting
	launcherServerRunning  = launchercore.ServerRunning
	launcherServerStopping = launchercore.ServerStopping
	launcherServerError    = launchercore.ServerError
)

func newLauncherServerController(runtime *app.Runtime, onState func(launcherServerStatus, string)) *launcherServerController {
	return launchercore.NewServerController(
		func() launchercore.ManagedServer {
			return launchercore.NewRuntimeServer(runtime)
		},
		func(state launchercore.ServerState) {
			fyne.Do(func() {
				onState(state.Status, serverStateDetail(state))
			})
		},
	)
}

func serverStatusDisplay(status launcherServerStatus) string {
	switch status {
	case launcherServerStarting:
		return "시작 중"
	case launcherServerRunning:
		return "실행 중"
	case launcherServerStopping:
		return "종료 중"
	case launcherServerError:
		return "오류"
	default:
		return "정지"
	}
}

func serverStateDetail(state launchercore.ServerState) string {
	if state.Err != nil {
		switch state.Operation {
		case launchercore.ServerOperationRestart:
			return "서버 재시작 실패: " + state.Err.Error()
		case launchercore.ServerOperationStop:
			return "서버 종료 실패: " + state.Err.Error()
		default:
			return "서버 오류: " + state.Err.Error()
		}
	}

	switch state.Status {
	case launcherServerStarting:
		if state.Operation == launchercore.ServerOperationRestart {
			return "서버를 다시 시작하고 있습니다."
		}
		return "로컬 서버를 시작하고 있습니다."
	case launcherServerRunning:
		return "서버가 실행 중입니다."
	case launcherServerStopping:
		if state.Operation == launchercore.ServerOperationRestart {
			return "서버를 다시 시작하고 있습니다."
		}
		return "서버를 종료하고 있습니다."
	default:
		return "서버가 정지되어 있습니다."
	}
}
