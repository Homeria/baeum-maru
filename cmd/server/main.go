package main

import (
	"context"
	"fmt"
	"os"
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
		fmt.Fprintf(os.Stderr, "server startup failed: %v\n", err)
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

	fmt.Printf("%s server running at http://%s\n", runtime.Config.App.DisplayName, httpServer.Addr())

	select {
	case <-ctx.Done():
		shutdownCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		if err := httpServer.Shutdown(shutdownCtx); err != nil {
			fmt.Fprintf(os.Stderr, "server shutdown failed: %v\n", err)
			os.Exit(1)
		}
	case err := <-errCh:
		if err != nil {
			fmt.Fprintf(os.Stderr, "server failed: %v\n", err)
			os.Exit(1)
		}
	}
}
