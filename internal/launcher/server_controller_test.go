package launcher

import (
	"context"
	"errors"
	"sync"
	"testing"
	"time"
)

func TestServerControllerStartAndStop(t *testing.T) {
	states := make(chan ServerState, 10)
	server := newFakeManagedServer(nil)
	controller := NewServerController(func() ManagedServer { return server }, func(state ServerState) {
		states <- state
	})

	if state := controller.State(); state.Status != ServerStopped {
		t.Fatalf("initial status = %q, want stopped", state.Status)
	}
	if !controller.Start() {
		t.Fatal("Start() = false, want accepted")
	}
	waitForServerStatus(t, states, ServerStarting)
	waitForServerStatus(t, states, ServerRunning)

	if controller.Start() {
		t.Fatal("second Start() = true, want rejected while running")
	}
	if !controller.Stop() {
		t.Fatal("Stop() = false, want accepted")
	}
	waitForServerStatus(t, states, ServerStopping)
	stopped := waitForServerStatus(t, states, ServerStopped)
	if stopped.Operation != ServerOperationStop {
		t.Fatalf("stop operation = %q, want stop", stopped.Operation)
	}

	select {
	case <-server.shutdownCalled:
	case <-time.After(time.Second):
		t.Fatal("server Shutdown() was not called")
	}
}

func TestServerControllerRestartsWithNewServer(t *testing.T) {
	states := make(chan ServerState, 20)
	var mu sync.Mutex
	var servers []*fakeManagedServer
	controller := NewServerController(func() ManagedServer {
		mu.Lock()
		defer mu.Unlock()
		server := newFakeManagedServer(nil)
		servers = append(servers, server)
		return server
	}, func(state ServerState) {
		states <- state
	})

	controller.Start()
	waitForServerStatus(t, states, ServerStarting)
	waitForServerStatus(t, states, ServerRunning)
	if !controller.Restart() {
		t.Fatal("Restart() = false, want accepted")
	}
	waitForServerStatus(t, states, ServerStopping)
	waitForServerStatus(t, states, ServerStopped)
	waitForServerStatus(t, states, ServerStarting)
	restarted := waitForServerStatus(t, states, ServerRunning)
	if restarted.Operation != ServerOperationRestart {
		t.Fatalf("restart operation = %q, want restart", restarted.Operation)
	}

	mu.Lock()
	serverCount := len(servers)
	mu.Unlock()
	if serverCount != 2 {
		t.Fatalf("server factory calls = %d, want 2", serverCount)
	}

	if err := controller.Shutdown(context.Background()); err != nil {
		t.Fatalf("Shutdown() error = %v", err)
	}
}

func TestServerControllerReportsStartFailure(t *testing.T) {
	states := make(chan ServerState, 10)
	wantErr := errors.New("listen failed")
	controller := NewServerController(func() ManagedServer {
		return newFakeManagedServer(wantErr)
	}, func(state ServerState) {
		states <- state
	})

	controller.Start()
	waitForServerStatus(t, states, ServerStarting)
	waitForServerStatus(t, states, ServerRunning)
	failed := waitForServerStatus(t, states, ServerError)
	if !errors.Is(failed.Err, wantErr) {
		t.Fatalf("server error = %v, want %v", failed.Err, wantErr)
	}
	if !controller.State().Status.CanStart() {
		t.Fatal("failed controller cannot be started again")
	}
}

type fakeManagedServer struct {
	startErr       error
	stop           chan struct{}
	shutdownCalled chan struct{}
	stopOnce       sync.Once
	shutdownOnce   sync.Once
}

func newFakeManagedServer(startErr error) *fakeManagedServer {
	return &fakeManagedServer{
		startErr:       startErr,
		stop:           make(chan struct{}),
		shutdownCalled: make(chan struct{}),
	}
}

func (s *fakeManagedServer) Start() error {
	if s.startErr != nil {
		return s.startErr
	}
	<-s.stop
	return nil
}

func (s *fakeManagedServer) Shutdown(context.Context) error {
	s.shutdownOnce.Do(func() { close(s.shutdownCalled) })
	s.stopOnce.Do(func() { close(s.stop) })
	return nil
}

func waitForServerStatus(t *testing.T, states <-chan ServerState, want ServerStatus) ServerState {
	t.Helper()
	deadline := time.After(2 * time.Second)
	for {
		select {
		case state := <-states:
			if state.Status == want {
				return state
			}
		case <-deadline:
			t.Fatalf("timed out waiting for server status %q", want)
		}
	}
}
