//go:build !fyne

package main

import (
	"context"
	"fmt"
	"os"
	"os/exec"
	"os/signal"
	"syscall"
	"time"

	"github.com/Homeria/baeum-maru/internal/app"
	launchercore "github.com/Homeria/baeum-maru/internal/launcher"
)

func main() {
	runtime, err := app.Bootstrap("")
	if err != nil {
		fmt.Fprintf(os.Stderr, "launcher startup failed: %v\n", err)
		os.Exit(1)
	}
	defer runtime.Close()

	httpServer := launchercore.NewRuntimeServer(runtime)

	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()

	errCh := make(chan error, 1)
	go func() {
		errCh <- httpServer.Start()
	}()

	url := launchercore.BrowserURL(runtime.Config.Server.Host, runtime.Config.Server.Port)
	fmt.Printf("%s %s\n", runtime.Config.App.DisplayName, app.Version)
	fmt.Printf("server running at %s\n", url)
	if runtime.Config.UI.OpenBrowserOnStart {
		if err := openBrowser(url); err != nil {
			runtime.Logger.Warn("open browser failed", "error", err)
		}
	}

	select {
	case <-ctx.Done():
		shutdownCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		if err := httpServer.Shutdown(shutdownCtx); err != nil {
			fmt.Fprintf(os.Stderr, "launcher shutdown failed: %v\n", err)
			os.Exit(1)
		}
	case err := <-errCh:
		if err != nil {
			fmt.Fprintf(os.Stderr, "launcher server failed: %v\n", err)
			os.Exit(1)
		}
	}
}

func openBrowser(url string) error {
	return exec.Command("rundll32", "url.dll,FileProtocolHandler", url).Start()
}
