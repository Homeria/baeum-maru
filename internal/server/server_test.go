package server

import (
	"context"
	"errors"
	"net/http"
	"testing"
)

func TestNewBuildsAddress(t *testing.T) {
	srv := New(Options{
		Host: "127.0.0.1",
		Port: 18081,
	})

	if srv.Addr() != "127.0.0.1:18081" {
		t.Fatalf("Addr() = %q, want %q", srv.Addr(), "127.0.0.1:18081")
	}
}

func TestStartRejectsNilServer(t *testing.T) {
	var srv *Server

	if err := srv.Start(); err == nil {
		t.Fatal("Start() error = nil, want error")
	}
}

func TestShutdownAcceptsNilServer(t *testing.T) {
	var srv *Server

	if err := srv.Shutdown(context.Background()); err != nil {
		t.Fatalf("Shutdown() error = %v, want nil", err)
	}
}

func TestStartReturnsListenError(t *testing.T) {
	srv := New(Options{
		Host:    "127.0.0.1",
		Port:    -1,
		Handler: http.NewServeMux(),
	})

	err := srv.Start()
	if err == nil {
		t.Fatal("Start() error = nil, want listen error")
	}
	if errors.Is(err, http.ErrServerClosed) {
		t.Fatalf("Start() error = %v, did not expect ErrServerClosed", err)
	}
}
