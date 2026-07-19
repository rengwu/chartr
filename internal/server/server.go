// Package server is the harness backend: one HTTP server that serves the
// embedded cockpit SPA, answers operator actions over plain HTTP, and pushes
// the whole derived model to every browser over a JSON control socket (ADR
// 0010). The walking skeleton wires the transport end to end with a near-empty
// model; later tickets add spaces, maps, sessions, and the review gate on top
// of exactly this delivery skeleton.
package server

import (
	"context"
	"errors"
	"io/fs"
	"net"
	"net/http"
	"time"

	"github.com/rengwu/wayfinder-harness/internal/registry"
	"github.com/rengwu/wayfinder-harness/web"
)

// Options configures a Server.
type Options struct {
	// DataDir is the harness-owned state root (registry, per-session payload
	// archives, scrollback). The skeleton only holds it; ticket 02 onward reads
	// and writes beneath it. Defaults to the current directory when empty.
	DataDir string
}

// Server is a single operator's harness backend. Construct with New, then run
// with Serve.
type Server struct {
	opts Options
	hub  *hub
	mux  *http.ServeMux
	reg  *registry.Registry
}

// New builds a Server with the control-socket hub seeded to the empty model and
// all routes wired. It fails only if the embedded SPA cannot be opened, which
// cannot happen once the web package compiles.
func New(opts Options) (*Server, error) {
	if opts.DataDir == "" {
		opts.DataDir = "."
	}
	dist, err := web.Dist()
	if err != nil {
		return nil, err
	}
	reg, err := registry.Load(opts.DataDir)
	if err != nil {
		return nil, err
	}

	s := &Server{
		opts: opts,
		hub:  newHub(),
		mux:  http.NewServeMux(),
		reg:  reg,
	}

	// The control socket: JSON, server-authoritative, whole-snapshot push.
	s.mux.HandleFunc("/ws/control", s.handleControl)
	// Operator actions are plain HTTP request/response so a failure surfaces as
	// a response (ADR 0010). Health is the skeleton's; ticket 02 adds the
	// registry actions; classify, spawn, approve, and the rest hang here later.
	s.mux.HandleFunc("/api/health", s.handleHealth)
	s.mux.HandleFunc("POST /api/spaces", s.handleRegister)
	s.mux.HandleFunc("DELETE /api/spaces/{id}", s.handleDeregister)
	s.mux.HandleFunc("POST /api/spaces/{id}/pin", s.handlePin)
	// Everything else is the embedded SPA, with a client-routing fallback.
	s.mux.Handle("/", spaHandler(dist))

	// Reflect any registry persisted from a prior run in the first snapshot, so
	// a restart restores the operator's spaces without a re-register.
	s.rebuild()

	return s, nil
}

// Serve runs the HTTP server on ln until ctx is cancelled, then drains
// in-flight requests within a short grace period. Serve owns ln and closes it.
func (s *Server) Serve(ctx context.Context, ln net.Listener) error {
	httpSrv := &http.Server{
		Handler:           s.mux,
		ReadHeaderTimeout: 10 * time.Second,
		// No WriteTimeout: the control and terminal sockets are long-lived.
		BaseContext: func(net.Listener) context.Context { return ctx },
	}

	serveErr := make(chan error, 1)
	go func() {
		err := httpSrv.Serve(ln)
		if errors.Is(err, http.ErrServerClosed) {
			err = nil
		}
		serveErr <- err
	}()

	select {
	case <-ctx.Done():
		shutdownCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		_ = httpSrv.Shutdown(shutdownCtx)
		return <-serveErr
	case err := <-serveErr:
		return err
	}
}

func (s *Server) handleHealth(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	_, _ = w.Write([]byte(`{"status":"ok"}`))
}

// spaHandler serves static assets from the embedded build and falls back to
// index.html for any path that is not a real file, so client-side deep links
// (ticket 07's star names) resolve. On an unbuilt checkout dist holds only
// .gitkeep and non-asset paths 404 — the browser demo needs `make web` first;
// the tests exercise the control socket and HTTP actions, which do not.
func spaHandler(dist fs.FS) http.Handler {
	files := http.FileServer(http.FS(dist))
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path == "/" || fileExists(dist, r.URL.Path) {
			files.ServeHTTP(w, r)
			return
		}
		if index, err := fs.ReadFile(dist, "index.html"); err == nil {
			w.Header().Set("Content-Type", "text/html; charset=utf-8")
			_, _ = w.Write(index)
			return
		}
		http.NotFound(w, r)
	})
}

func fileExists(fsys fs.FS, urlPath string) bool {
	name := urlPath
	for len(name) > 0 && name[0] == '/' {
		name = name[1:]
	}
	if name == "" {
		return false
	}
	info, err := fs.Stat(fsys, name)
	return err == nil && !info.IsDir()
}
