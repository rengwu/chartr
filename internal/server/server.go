// Package server is the chartr backend: one HTTP server that serves the
// embedded cockpit SPA, answers operator actions over plain HTTP, and pushes
// the whole derived model to every browser over a JSON control socket (ADR
// 0010). The walking skeleton wires the transport end to end with a near-empty
// model; later tickets add spaces, maps, and sessions on top
// of exactly this delivery skeleton.
package server

import (
	"context"
	"errors"
	"io/fs"
	"net"
	"net/http"
	"os"
	"path/filepath"
	"sync"
	"time"

	"github.com/rengwu/chartr/internal/prompt"
	"github.com/rengwu/chartr/internal/registry"
	"github.com/rengwu/chartr/internal/terminal"
	"github.com/rengwu/chartr/web"
)

// Options configures a Server.
type Options struct {
	// DataDir is the chartr-owned runtime root — per-session payload archives and
	// scrollback under `sessions/`. It is per-workspace state, not config, and
	// defaults to the current directory when empty.
	DataDir string
	// ConfigDir is the operator's local config root — `~/.config/chartr` on every
	// platform (honouring `XDG_CONFIG_HOME`) — and the single home for every
	// user-scoped setting: the space
	// registry (`spaces.toml`), the agent library (`user.toml`), terminal
	// customization (`terminal.toml`), the operator's own skills (`skills/`), and
	// the materialized built-in library (`builtin-skills/`). Defaults to the OS
	// user config dir; tests point it at a temp dir so a developer's own config
	// never leaks into a run.
	ConfigDir string
}

// Server is a single operator's chartr backend. Construct with New, then run
// with Serve.
type Server struct {
	opts  Options
	hub   *hub
	mux   *http.ServeMux
	reg   *registry.Registry
	watch *watcher
	terms *terminal.Manager
	// pickLock serializes native folder choosers, so a double-click on New Space
	// raises one dialog rather than a stack the operator dismisses in order.
	pickLock sync.Mutex
}

// New builds a Server with the control-socket hub seeded to the empty model and
// all routes wired. It fails only if the embedded SPA cannot be opened, which
// cannot happen once the web package compiles.
func New(opts Options) (*Server, error) {
	if opts.DataDir == "" {
		opts.DataDir = "."
	}
	if opts.ConfigDir == "" {
		opts.ConfigDir = userConfigRoot(opts.DataDir)
	}
	dist, err := web.Dist()
	if err != nil {
		return nil, err
	}
	reg, err := registry.Load(opts.ConfigDir)
	if err != nil {
		return nil, err
	}
	// Materialize the skill library to disk so the operator can read and edit
	// exactly what a session is told (ticket 08, story 45). Existing files are
	// preserved, so edits survive a restart and compose on the next preview.
	if err := prompt.Materialize(opts.ConfigDir); err != nil {
		return nil, err
	}

	s := &Server{
		opts: opts,
		hub:  newHub(),
		mux:  http.NewServeMux(),
		reg:  reg,
	}
	// Discovery is by notice, not refresh (story 11): the watch fires a rebuild
	// whenever a space's `.plan/` changes, so a map created outside chartr
	// appears on its own. rebuild also reconciles the watch set, so this starts
	// covering whatever the persisted registry already holds.
	//
	// The config root is pinned into the same watch. It is nobody's space, but
	// every rebuild re-reads `user.toml` and `terminal.toml` out of it, so an
	// operator saving a config edit in their own editor — the only way those files
	// are ever edited, since the surface opens rather than edits them — is the same
	// kind of notice a map write is, and reaches live terminals without a refresh.
	s.watch = newWatcher(s.rebuild, opts.ConfigDir)
	// Ad-hoc shells are chartr-owned runtime state (ticket 05). The manager
	// pushes a fresh model whenever a terminal opens or ends, so a tab appears
	// and disappears on its own; the model is built before the first rebuild.
	s.terms = terminal.NewManager(s.rebuild)

	// The control socket: JSON, server-authoritative, whole-snapshot push.
	s.mux.HandleFunc("/ws/control", s.handleControl)
	// The terminal socket: binary, one per attached terminal, raw PTY bytes down
	// and keystrokes up, scrollback replayed on attach. A separate connection
	// from the control socket by design, so a flooding shell cannot block map
	// updates (ADR 0010).
	s.mux.HandleFunc("/ws/terminal/{termID}", s.handleTerminal)
	// Operator actions are plain HTTP request/response so a failure surfaces as
	// a response (ADR 0010). Health is the skeleton's; ticket 02 adds the
	// registry actions; spawn, halt, and the rest hang here later.
	s.mux.HandleFunc("/api/health", s.handleHealth)
	s.mux.HandleFunc("POST /api/spaces", s.handleRegister)
	// Naming the folder to register: the operator's own OS folder chooser, raised
	// server-side like the config surface's open action. It is a POST because it
	// raises a dialog, and it is separate from the register above so that action
	// stays the one place a space is created — the client posts back whatever path
	// comes out of here.
	s.mux.HandleFunc("POST /api/spaces/pick", s.handlePickFolder)
	s.mux.HandleFunc("DELETE /api/spaces/{id}", s.handleDeregister)
	s.mux.HandleFunc("POST /api/spaces/{id}/pin", s.handlePin)
	// The remembered agent, set directly rather than only as a side effect of a
	// spawn — the action footer's agent selector persists the operator's pick the
	// moment they make it, so it reads as a saved setting, not a pending choice.
	s.mux.HandleFunc("PUT /api/spaces/{id}/agent", s.handleSetSpaceAgent)
	// The effective config surface (ticket 05, ADR 0014). The read half rides the
	// per-space model push; these are the two writes it is allowed. Editing a role
	// Opening a layer file is a POST because it launches a process, and it resolves
	// a *named* layer server-side — never a path from the client.
	s.mux.HandleFunc("POST /api/spaces/{id}/config/open", s.handleOpenLayer)
	// The same open, for the layers that belong to no space — the operator's own
	// config file and the global skill library. The global scope is reachable with
	// nothing registered, so it cannot borrow a space id to open its own files.
	s.mux.HandleFunc("POST /api/config/open", s.handleOpenGlobalLayer)
	// Stamp a global config file from its defaults template — the companion to the
	// open action for a layer that does not exist yet. Named-layer resolution and a
	// bundled template server-side, so the client sends a name, never a path or
	// contents; only a layer with a template can be created, and an existing file is
	// never clobbered.
	s.mux.HandleFunc("POST /api/config/create", s.handleCreateGlobalLayer)
	// The agent library: named launch specs the operator registers once and picks
	// from at the gate. Global — the library lives in the operator's own file and is
	// never committed — so these routes take no space id and work with nothing
	// registered at all.
	s.mux.HandleFunc("PUT /api/config/agents/{name}", s.handleSetAgent)
	s.mux.HandleFunc("DELETE /api/config/agents/{name}", s.handleDeleteAgent)
	// Payload preview (ticket 08): for a chosen ticket and role, exactly what a
	// session would be told, with per-part layer provenance. Read-only, so a GET;
	// the composition reads the library and the map fresh off disk each time.
	s.mux.HandleFunc("GET /api/spaces/{id}/maps/{slug}/tickets/{num}/payload", s.handlePayloadPreview)
	// Spawn a session (ticket 09): the tracer bullet. From a frontier ticket, the
	// chartr writes the claim commit, composes and archives the payload, settles the
	// chosen agent, and launches the agent's own TUI with the opener typed in — or
	// hard-blocks the one spawn when the chosen agent is unregistered or absent. A
	// plain HTTP action so a refusal (missing agent, a ticket off the frontier)
	// surfaces as a response.
	s.mux.HandleFunc("POST /api/spaces/{id}/maps/{slug}/tickets/{num}/spawn", s.handleSpawn)
	// The death halt (ticket 10): a session that died stays pinned to its ticket,
	// and the operator resolves it one of exactly three ways — resume it (same-
	// ticket crash recovery), respawn a fresh session on the same ticket, or release
	// the claim back to the frontier. chartr itself takes none of these; each is
	// an explicit operator action, so nothing changes without an HTTP call.
	s.mux.HandleFunc("POST /api/spaces/{id}/sessions/{sid}/resume", s.handleResume)
	s.mux.HandleFunc("POST /api/spaces/{id}/sessions/{sid}/respawn", s.handleRespawn)
	s.mux.HandleFunc("POST /api/spaces/{id}/sessions/{sid}/release", s.handleRelease)
	// Ad-hoc shells: open one in the space's working tree, end one by the human's
	// command. Opening is a plain HTTP action so a spawn failure surfaces as a
	// response (ADR 0010); the shell itself lives on the terminal socket.
	s.mux.HandleFunc("POST /api/spaces/{id}/terminals", s.handleOpenTerminal)
	s.mux.HandleFunc("DELETE /api/spaces/{id}/terminals/{termID}", s.handleCloseTerminal)
	// The ideate on-ramp (ticket 15): the one opinionated nudge toward charting.
	// A live, ticketless agent tab opened with the on-disk starter prompt typed
	// in — no map or ticket lookup, no claim, no lifecycle, ended only by the
	// human, exactly like an ad-hoc shell.
	s.mux.HandleFunc("POST /api/spaces/{id}/ideate", s.handleIdeate)
	// Everything else is the embedded SPA, with a client-routing fallback.
	s.mux.Handle("/", spaHandler(dist))

	// Reflect any registry persisted from a prior run in the first snapshot, so
	// a restart restores the operator's spaces without a re-register.
	s.rebuild()

	return s, nil
}

// userConfigRoot resolves chartr's config root to `~/.config/chartr`, honouring
// `XDG_CONFIG_HOME` when it is set. This is deliberately *not* os.UserConfigDir:
// that returns `~/Library/Application Support` on macOS, and we want one path an
// operator can reason about on every platform. It falls back to the runtime root
// only when there is no home directory to anchor to (a stripped-down environment).
func userConfigRoot(fallback string) string {
	if xdg := os.Getenv("XDG_CONFIG_HOME"); xdg != "" {
		return filepath.Join(xdg, "chartr")
	}
	if home, err := os.UserHomeDir(); err == nil && home != "" {
		return filepath.Join(home, ".config", "chartr")
	}
	return fallback
}

// Serve runs the HTTP server on ln until ctx is cancelled, then drains
// in-flight requests within a short grace period. Serve owns ln and closes it.
func (s *Server) Serve(ctx context.Context, ln net.Listener) error {
	defer s.watch.close()
	defer s.terms.Shutdown()

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
