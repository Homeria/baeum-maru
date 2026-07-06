//go:build fyne

package main

import (
	"context"
	"fmt"
	"os"
	"time"

	"fyne.io/fyne/v2"
	fyneapp "fyne.io/fyne/v2/app"
	"fyne.io/fyne/v2/container"
	"fyne.io/fyne/v2/layout"

	"github.com/Homeria/baeum-maru/internal/app"
)

func main() {
	runtime, err := app.Bootstrap("")
	if err != nil {
		fmt.Fprintf(os.Stderr, "launcher startup failed: %v\n", err)
		os.Exit(1)
	}

	guiApp := fyneapp.NewWithID("github.com.Homeria.baeum-maru.launcher")
	window := guiApp.NewWindow(runtime.Config.App.DisplayName + " 런처")
	window.Resize(fyne.NewSize(1120, 760))

	state := newLauncherState(
		runtime,
		guiApp,
		browserURL(runtime.Config.Server.Host, runtime.Config.Server.Port),
		networkAccessURLs(runtime.Config.Server.Host, runtime.Config.Server.Port),
	)
	state.serverController = newLauncherServerController(runtime, state.setServerStatus)
	tabs := buildLauncherTabs(state)
	header := buildLauncherHeader(runtime.Config.App.DisplayName)

	window.SetContent(container.New(layout.NewPaddedLayout(), container.NewBorder(header, nil, nil, nil, tabs)))
	window.SetCloseIntercept(func() {
		shutdownCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		if err := state.serverController.Shutdown(shutdownCtx); err != nil {
			runtime.Logger.Warn("launcher shutdown failed", "error", err)
		}
		if err := runtime.Close(); err != nil {
			fmt.Fprintf(os.Stderr, "runtime close failed: %v\n", err)
		}
		window.Close()
		guiApp.Quit()
	})

	state.setServerStatus(launcherServerStopped, "서버가 정지되어 있습니다. 시작 버튼을 눌러 서버를 실행하세요.")
	state.refreshAccessCodes()

	window.ShowAndRun()
}
