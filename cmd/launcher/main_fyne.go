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
	"github.com/Homeria/baeum-maru/internal/server"
	"github.com/Homeria/baeum-maru/internal/web"
)

func main() {
	runtime, err := app.Bootstrap("")
	if err != nil {
		fmt.Fprintf(os.Stderr, "launcher startup failed: %v\n", err)
		os.Exit(1)
	}

	httpServer := server.New(server.Options{
		Host: runtime.Config.Server.Host,
		Port: runtime.Config.Server.Port,
		Handler: web.NewRouter(web.RouterOptions{
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
		}),
		Logger: runtime.Logger.Logger,
	})

	guiApp := fyneapp.NewWithID("github.com.Homeria.baeum-maru.launcher")
	window := guiApp.NewWindow(runtime.Config.App.DisplayName + " 런처")
	window.Resize(fyne.NewSize(1120, 760))

	state := newLauncherState(
		runtime,
		guiApp,
		browserURL(runtime.Config.Server.Host, runtime.Config.Server.Port),
		networkAccessURLs(runtime.Config.Server.Host, runtime.Config.Server.Port),
	)
	tabs := buildLauncherTabs(state)
	header := buildLauncherHeader(runtime.Config.App.DisplayName)

	window.SetContent(container.New(layout.NewPaddedLayout(), container.NewBorder(header, nil, nil, nil, tabs)))
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

	state.setStatus("서버 시작 중", "로컬 서버를 시작하고 있습니다.")
	go func() {
		if err := httpServer.Start(); err != nil {
			fyne.Do(func() {
				state.setStatus("서버 오류", "서버 오류: "+err.Error())
			})
		}
	}()
	state.setStatus("서버 실행 중", state.serverURL+" 에서 접속할 수 있습니다.")
	state.refreshAccessCodes()

	window.ShowAndRun()
}
