//go:build fyne

package main

import (
	"context"
	"errors"
	"fmt"
	"sync"
	"time"

	"fyne.io/fyne/v2"

	"github.com/Homeria/baeum-maru/internal/app"
	"github.com/Homeria/baeum-maru/internal/server"
	"github.com/Homeria/baeum-maru/internal/web"
)

type launcherServerStatus string

const (
	launcherServerStopped  launcherServerStatus = "stopped"
	launcherServerStarting launcherServerStatus = "starting"
	launcherServerRunning  launcherServerStatus = "running"
	launcherServerStopping launcherServerStatus = "stopping"
	launcherServerError    launcherServerStatus = "error"
)

type launcherServerController struct {
	mu      sync.Mutex
	runtime *app.Runtime
	server  *server.Server
	status  launcherServerStatus
	detail  string
	onState func(launcherServerStatus, string)
}

func newLauncherServerController(runtime *app.Runtime, onState func(launcherServerStatus, string)) *launcherServerController {
	return &launcherServerController{
		runtime: runtime,
		status:  launcherServerStopped,
		detail:  "서버가 정지되어 있습니다.",
		onState: onState,
	}
}

func (c *launcherServerController) Status() (launcherServerStatus, string) {
	c.mu.Lock()
	defer c.mu.Unlock()
	return c.status, c.detail
}

func (c *launcherServerController) Start() {
	c.mu.Lock()
	if c.status == launcherServerStarting || c.status == launcherServerRunning {
		c.mu.Unlock()
		return
	}
	srv := newLauncherHTTPServer(c.runtime)
	c.server = srv
	c.status = launcherServerStarting
	c.detail = "로컬 서버를 시작하고 있습니다."
	c.mu.Unlock()
	c.notify()

	go func() {
		c.setState(launcherServerRunning, "서버가 실행 중입니다.")
		if err := srv.Start(); err != nil {
			c.mu.Lock()
			if c.server == srv {
				c.server = nil
				c.status = launcherServerError
				c.detail = "서버 오류: " + err.Error()
			}
			c.mu.Unlock()
			c.notify()
			return
		}

		c.mu.Lock()
		if c.server == srv {
			c.server = nil
			c.status = launcherServerStopped
			c.detail = "서버가 정지되었습니다."
		}
		c.mu.Unlock()
		c.notify()
	}()
}

func (c *launcherServerController) Stop() {
	c.stop(false)
}

func (c *launcherServerController) Restart() {
	go func() {
		if err := c.shutdownCurrent(context.Background(), true); err != nil {
			c.setState(launcherServerError, "서버 재시작 실패: "+err.Error())
			return
		}
		c.Start()
	}()
}

func (c *launcherServerController) Shutdown(ctx context.Context) error {
	return c.shutdownCurrent(ctx, false)
}

func (c *launcherServerController) stop(sync bool) {
	if sync {
		_ = c.shutdownCurrent(context.Background(), false)
		return
	}
	go func() {
		if err := c.shutdownCurrent(context.Background(), false); err != nil {
			c.setState(launcherServerError, "서버 종료 실패: "+err.Error())
		}
	}()
}

func (c *launcherServerController) shutdownCurrent(ctx context.Context, restarting bool) error {
	c.mu.Lock()
	srv := c.server
	if srv == nil || c.status == launcherServerStopped {
		c.status = launcherServerStopped
		c.detail = "서버가 정지되어 있습니다."
		c.mu.Unlock()
		c.notify()
		return nil
	}
	c.status = launcherServerStopping
	if restarting {
		c.detail = "서버를 재시작하고 있습니다."
	} else {
		c.detail = "서버를 종료하고 있습니다."
	}
	c.mu.Unlock()
	c.notify()

	if ctx == nil {
		var cancel context.CancelFunc
		ctx, cancel = context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
	}
	if err := srv.Shutdown(ctx); err != nil {
		return err
	}

	c.mu.Lock()
	if c.server == srv {
		c.server = nil
		c.status = launcherServerStopped
		if restarting {
			c.detail = "서버를 재시작합니다."
		} else {
			c.detail = "서버가 정지되었습니다."
		}
	}
	c.mu.Unlock()
	c.notify()
	return nil
}

func (c *launcherServerController) setState(status launcherServerStatus, detail string) {
	c.mu.Lock()
	c.status = status
	c.detail = detail
	c.mu.Unlock()
	c.notify()
}

func (c *launcherServerController) notify() {
	if c.onState == nil {
		return
	}
	status, detail := c.Status()
	fyne.Do(func() {
		c.onState(status, detail)
	})
}

func (s launcherServerStatus) Display() string {
	switch s {
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

func (s launcherServerStatus) CanStart() bool {
	return s == launcherServerStopped || s == launcherServerError
}

func (s launcherServerStatus) CanStop() bool {
	return s == launcherServerRunning
}

func (s launcherServerStatus) CanRestart() bool {
	return s == launcherServerRunning || s == launcherServerError
}

func newLauncherHTTPServer(runtime *app.Runtime) *server.Server {
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

func launcherServerActionError(action string, err error) string {
	if err == nil {
		return ""
	}
	if errors.Is(err, context.Canceled) {
		return action + " 취소됨"
	}
	return fmt.Sprintf("%s 실패: %v", action, err)
}
