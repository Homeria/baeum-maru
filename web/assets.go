package web

import "embed"

// Files contains the HTML, CSS, and JavaScript assets embedded into the Go binary.
//
//go:embed templates/**/*.html static/**
var Files embed.FS
