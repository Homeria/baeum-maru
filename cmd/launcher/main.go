package main

import (
	"fmt"
	"os"

	"github.com/Homeria/baeum-maru/internal/app"
)

func main() {
	runtime, err := app.Bootstrap("")
	if err != nil {
		fmt.Fprintf(os.Stderr, "launcher startup failed: %v\n", err)
		os.Exit(1)
	}
	defer runtime.Close()

	runtime.Logger.Info("launcher skeleton started", "version", app.Version)
	fmt.Printf("%s launcher skeleton (%s)\n", runtime.Config.App.DisplayName, app.Version)
}
