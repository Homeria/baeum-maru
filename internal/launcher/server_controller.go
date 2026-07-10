package launcher

import (
	"context"
	"errors"
	"sync"
	"time"
)

type ServerStatus string

const (
	ServerStopped  ServerStatus = "stopped"
	ServerStarting ServerStatus = "starting"
	ServerRunning  ServerStatus = "running"
	ServerStopping ServerStatus = "stopping"
	ServerError    ServerStatus = "error"
)

type ServerOperation string

const (
	ServerOperationNone    ServerOperation = ""
	ServerOperationStart   ServerOperation = "start"
	ServerOperationStop    ServerOperation = "stop"
	ServerOperationRestart ServerOperation = "restart"
)

type ServerState struct {
	Status    ServerStatus
	Operation ServerOperation
	Err       error
}

func (s ServerStatus) CanStart() bool {
	return s == ServerStopped || s == ServerError
}

func (s ServerStatus) CanStop() bool {
	return s == ServerRunning
}

func (s ServerStatus) CanRestart() bool {
	return s == ServerRunning || s == ServerError
}

type ManagedServer interface {
	Start() error
	Shutdown(context.Context) error
}

type ServerFactory func() ManagedServer
type ServerStateListener func(ServerState)

type ServerController struct {
	mu              sync.Mutex
	factory         ServerFactory
	listener        ServerStateListener
	shutdownTimeout time.Duration
	server          ManagedServer
	generation      uint64
	state           ServerState
}

func NewServerController(factory ServerFactory, listener ServerStateListener) *ServerController {
	return &ServerController{
		factory:         factory,
		listener:        listener,
		shutdownTimeout: 5 * time.Second,
		state:           ServerState{Status: ServerStopped},
	}
}

func (c *ServerController) State() ServerState {
	if c == nil {
		return ServerState{Status: ServerStopped}
	}
	c.mu.Lock()
	defer c.mu.Unlock()
	return c.state
}

func (c *ServerController) Start() bool {
	return c.start(ServerOperationStart)
}

func (c *ServerController) start(operation ServerOperation) bool {
	if c == nil {
		return false
	}

	c.mu.Lock()
	if !c.state.Status.CanStart() || c.server != nil {
		c.mu.Unlock()
		return false
	}
	if c.factory == nil {
		state := ServerState{Status: ServerError, Operation: operation, Err: errors.New("server factory is not configured")}
		c.state = state
		c.mu.Unlock()
		c.notify(state)
		return false
	}

	srv := c.factory()
	if srv == nil {
		state := ServerState{Status: ServerError, Operation: operation, Err: errors.New("server factory returned nil")}
		c.state = state
		c.mu.Unlock()
		c.notify(state)
		return false
	}

	c.generation++
	generation := c.generation
	c.server = srv
	state := ServerState{Status: ServerStarting, Operation: operation}
	c.state = state
	c.mu.Unlock()
	c.notify(state)

	go c.run(srv, generation, operation)
	return true
}

func (c *ServerController) run(srv ManagedServer, generation uint64, operation ServerOperation) {
	if !c.setStateForServer(srv, generation, ServerState{Status: ServerRunning, Operation: operation}) {
		return
	}

	err := srv.Start()
	state := ServerState{Status: ServerStopped, Operation: operation}
	if err != nil {
		state.Status = ServerError
		state.Err = err
	}

	c.mu.Lock()
	if c.server != srv || c.generation != generation {
		c.mu.Unlock()
		return
	}
	if c.state.Status == ServerStopping {
		c.mu.Unlock()
		return
	}
	c.server = nil
	c.state = state
	c.mu.Unlock()
	c.notify(state)
}

func (c *ServerController) Stop() bool {
	if c == nil {
		return false
	}

	c.mu.Lock()
	if !c.state.Status.CanStop() || c.server == nil {
		c.mu.Unlock()
		return false
	}
	srv := c.server
	generation := c.generation
	state := ServerState{Status: ServerStopping, Operation: ServerOperationStop}
	c.state = state
	c.mu.Unlock()
	c.notify(state)

	go func() {
		ctx, cancel := context.WithTimeout(context.Background(), c.shutdownTimeout)
		defer cancel()
		_ = c.shutdownServer(ctx, srv, generation, ServerOperationStop)
	}()
	return true
}

func (c *ServerController) Restart() bool {
	if c == nil {
		return false
	}

	c.mu.Lock()
	if !c.state.Status.CanRestart() {
		c.mu.Unlock()
		return false
	}
	srv := c.server
	generation := c.generation
	if srv == nil {
		c.state = ServerState{Status: ServerStopped, Operation: ServerOperationRestart}
		c.mu.Unlock()
		return c.start(ServerOperationRestart)
	}
	state := ServerState{Status: ServerStopping, Operation: ServerOperationRestart}
	c.state = state
	c.mu.Unlock()
	c.notify(state)

	go func() {
		ctx, cancel := context.WithTimeout(context.Background(), c.shutdownTimeout)
		defer cancel()
		if err := c.shutdownServer(ctx, srv, generation, ServerOperationRestart); err == nil {
			c.start(ServerOperationRestart)
		}
	}()
	return true
}

func (c *ServerController) Shutdown(ctx context.Context) error {
	if c == nil {
		return nil
	}

	c.mu.Lock()
	srv := c.server
	generation := c.generation
	if srv == nil || c.state.Status == ServerStopped {
		c.state = ServerState{Status: ServerStopped, Operation: ServerOperationStop}
		state := c.state
		c.mu.Unlock()
		c.notify(state)
		return nil
	}
	state := ServerState{Status: ServerStopping, Operation: ServerOperationStop}
	c.state = state
	c.mu.Unlock()
	c.notify(state)

	if ctx == nil {
		var cancel context.CancelFunc
		ctx, cancel = context.WithTimeout(context.Background(), c.shutdownTimeout)
		defer cancel()
	}
	return c.shutdownServer(ctx, srv, generation, ServerOperationStop)
}

func (c *ServerController) shutdownServer(ctx context.Context, srv ManagedServer, generation uint64, operation ServerOperation) error {
	err := srv.Shutdown(ctx)
	if err != nil {
		c.setStateForServer(srv, generation, ServerState{Status: ServerError, Operation: operation, Err: err})
		return err
	}
	c.setStateForServer(srv, generation, ServerState{Status: ServerStopped, Operation: operation})
	return nil
}

func (c *ServerController) setStateForServer(srv ManagedServer, generation uint64, state ServerState) bool {
	c.mu.Lock()
	if c.server != srv || c.generation != generation {
		c.mu.Unlock()
		return false
	}
	if state.Status == ServerStopped {
		c.server = nil
	}
	c.state = state
	c.mu.Unlock()
	c.notify(state)
	return true
}

func (c *ServerController) notify(state ServerState) {
	if c.listener != nil {
		c.listener(state)
	}
}
