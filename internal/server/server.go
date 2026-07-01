// Package server owns HTTP server lifecycle and client tracking.
package server

import (
	"context"
	"errors"
	"fmt"
	"log/slog"
	"net"
	"net/http"
	"strconv"
	"time"
)

type Options struct {
	Host              string
	Port              int
	Handler           http.Handler
	Logger            *slog.Logger
	ReadHeaderTimeout time.Duration
	ShutdownTimeout   time.Duration
}

type Server struct {
	httpServer      *http.Server
	logger          *slog.Logger
	shutdownTimeout time.Duration
}

func New(opts Options) *Server {
	if opts.Host == "" {
		opts.Host = "0.0.0.0"
	}
	if opts.Port == 0 {
		opts.Port = 18080
	}
	if opts.Handler == nil {
		opts.Handler = http.NotFoundHandler()
	}
	if opts.Logger == nil {
		opts.Logger = slog.Default()
	}
	if opts.ReadHeaderTimeout == 0 {
		opts.ReadHeaderTimeout = 5 * time.Second
	}
	if opts.ShutdownTimeout == 0 {
		opts.ShutdownTimeout = 5 * time.Second
	}

	addr := net.JoinHostPort(opts.Host, strconv.Itoa(opts.Port))
	return &Server{
		httpServer: &http.Server{
			Addr:              addr,
			Handler:           opts.Handler,
			ReadHeaderTimeout: opts.ReadHeaderTimeout,
		},
		logger:          opts.Logger,
		shutdownTimeout: opts.ShutdownTimeout,
	}
}

func (s *Server) Addr() string {
	if s == nil || s.httpServer == nil {
		return ""
	}
	return s.httpServer.Addr
}

func (s *Server) Start() error {
	if s == nil || s.httpServer == nil {
		return errors.New("server is not initialized")
	}

	s.logger.Info("http server starting", "addr", s.httpServer.Addr)
	err := s.httpServer.ListenAndServe()
	if errors.Is(err, http.ErrServerClosed) {
		s.logger.Info("http server stopped")
		return nil
	}
	if err != nil {
		return fmt.Errorf("listen and serve %s: %w", s.httpServer.Addr, err)
	}
	return nil
}

func (s *Server) Shutdown(ctx context.Context) error {
	if s == nil || s.httpServer == nil {
		return nil
	}

	if ctx == nil {
		var cancel context.CancelFunc
		ctx, cancel = context.WithTimeout(context.Background(), s.shutdownTimeout)
		defer cancel()
	}

	s.logger.Info("http server shutting down", "addr", s.httpServer.Addr)
	if err := s.httpServer.Shutdown(ctx); err != nil {
		return fmt.Errorf("shutdown http server: %w", err)
	}
	return nil
}
