package launcher

import (
	"github.com/Homeria/baeum-maru/internal/app"
	"github.com/Homeria/baeum-maru/internal/server"
	"github.com/Homeria/baeum-maru/internal/web"
)

func NewRuntimeServer(runtime *app.Runtime) ManagedServer {
	if runtime == nil {
		return nil
	}
	return server.New(server.Options{
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
			Locations:     runtime.Locations,
		}),
		Logger: runtime.Logger.Logger,
	})
}
