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
	"github.com/Homeria/baeum-maru/internal/server"
	"github.com/Homeria/baeum-maru/internal/web"
)

func main() {
	runtime, err := app.Bootstrap("")
	if err != nil {
		fmt.Fprintf(os.Stderr, "launcher startup failed: %v\n", err)
		os.Exit(1)
	}
	defer runtime.Close()

	router := web.NewRouter(web.RouterOptions{
		DisplayName:   runtime.Config.App.DisplayName,
		Version:       app.Version,
		Logger:        runtime.Logger.Logger,
		Members:       runtime.Members,
		Courses:       runtime.Courses,
		Registrations: runtime.Registrations,
		Lotteries:     runtime.Lotteries,
		Exports:       runtime.Exports,
		Imports:       runtime.Imports,
		Backups:       runtime.Backups,
		Attendance:    runtime.Attendance,
	})
	httpServer := server.New(server.Options{
		Host:    runtime.Config.Server.Host,
		Port:    runtime.Config.Server.Port,
		Handler: router,
		Logger:  runtime.Logger.Logger,
	})

	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()

	errCh := make(chan error, 1)
	go func() {
		errCh <- httpServer.Start()
	}()

	url := browserURL(runtime.Config.Server.Host, runtime.Config.Server.Port)
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

func browserURL(host string, port int) string {
	if host == "" || host == "0.0.0.0" || host == "::" {
		host = "127.0.0.1"
	}
	return fmt.Sprintf("http://%s:%d", host, port)
}
