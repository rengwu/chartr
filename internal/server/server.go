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
	"log"
	"net"
	"net/http"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"time"

	"github.com/rengwu/chartr/internal/notify"
	"github.com/rengwu/chartr/internal/registry"
	"github.com/rengwu/chartr/internal/sources"
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
	// customization (`terminal.toml`), notification timing (`notify.toml`), the
	// source list (`sources.toml`) and the skill sources under `sources/`, and the
	// operator's own `preferences.md`. The generated write contract used to live
	// here too (`conventions.md`); it is now materialized per-space instead (see
	// prompt.ReconcileSpaceConventions), so a session sandboxed to its own space
	// can read it.
	// Defaults to the OS user config dir; tests point it at a
	// temp dir so a developer's own config never leaks into a run.
	ConfigDir string
	// Notifier fires the OS notification that reports an ended run. It defaults to
	// the platform notifier for the machine chartr is running on, chosen here so
	// nothing downstream knows which OS it is on; a test substitutes a stub, which
	// is what keeps the suite from ever shelling out to a real notifier.
	Notifier notify.Notifier
	// DefaultSourceURL is the git skill source a genuinely fresh install (one with
	// no `sources.toml` yet) pre-registers, so the first run already has something
	// to spawn from. The real entrypoints set it to DefaultSkillSourceURL; tests
	// leave it empty, which skips the pre-registration entirely so the suite never
	// clones over the network. Registration is best-effort — a fresh install with
	// no network or no `git` still starts, just with an empty list.
	DefaultSourceURL string
	// NoWatch starts with the filesystem watch dead — the same state newWatcher
	// falls back to when the OS watcher cannot be created (fd limits, a sandbox,
	// an unusual filesystem). fsnotify's init cannot be made to fail portably, so
	// this is how a test reaches the degraded path that connect-time discovery
	// exists to cover (ticket 19).
	NoWatch bool
}

// Server is a single operator's chartr backend. Construct with New, then run
// with Serve.
type Server struct {
	opts Options
	hub  *hub
	mux  *http.ServeMux
	reg  *registry.Registry
	// srcs is the operator's skill-source list. It is loaded once here and walked
	// fresh on every resolve, so a skill folder created a second ago is usable in
	// the very next spawn without a restart.
	srcs  *sources.Registry
	watch *watcher
	terms *terminal.Manager
	// origins is the browser-origin allowlist both websocket handshakes are
	// checked against (originPatterns). It is set once, in Serve, from the address
	// the listener actually bound: that is the last moment before a handler can
	// run and the first at which the address is known — the shell binds
	// `127.0.0.1:0` and learns its port only after net.Listen, so New cannot have
	// it.
	origins []string
	// pickLock serializes native folder choosers, so a double-click on New Space
	// raises one dialog rather than a stack the operator dismisses in order.
	pickLock sync.Mutex

	// notifier announces an ended run to the operating system, and notifyFailed
	// keeps that announcement quiet when it cannot: a machine with no notification
	// daemon logs its first failure and nothing after it, so a feature that can
	// never work on this box does not fill the log of one that is working fine.
	notifier     notify.Notifier
	notifyFailed sync.Once
}

// New builds a Server with the control-socket hub seeded to the empty model and
// all routes wired. It fails only if the embedded SPA cannot be opened, which
// cannot happen once the web package compiles.
func New(opts Options) (*Server, error) {
	if opts.DataDir == "" {
		opts.DataDir = "."
	}
	if opts.ConfigDir == "" {
		opts.ConfigDir = ConfigRoot(opts.DataDir)
	}
	if opts.Notifier == nil {
		opts.Notifier = notify.Platform()
	}
	// Create the config root before anything writes into it, so it is owner-only
	// from the start: the registry and the agent library live here, and a root
	// several writers can create is a root whose mode is whichever of them ran
	// first (websocket-origin-fix, ticket 05). An existing root is left as it is —
	// MkdirAll never chmods, and this path can fall back to the runtime root, which
	// is not chartr's to re-permission.
	if err := os.MkdirAll(opts.ConfigDir, ownerDirMode); err != nil {
		return nil, err
	}
	dist, err := web.Dist()
	if err != nil {
		return nil, err
	}
	reg, err := registry.Load(opts.ConfigDir)
	if err != nil {
		return nil, err
	}
	// Everything chartr writes into its own config root, in one ordered sequence
	// (see firstrun.go): the migration off the three-layer skill model, the source
	// list, the default source a fresh install pre-registers, and the operator's own
	// preferences file.
	srcs, err := firstRun(opts.ConfigDir, opts.DefaultSourceURL)
	if err != nil {
		return nil, err
	}

	s := &Server{
		opts:     opts,
		hub:      newHub(),
		mux:      http.NewServeMux(),
		reg:      reg,
		srcs:     srcs,
		notifier: opts.Notifier,
	}
	// Discovery is by notice, not refresh (story 11): the watch fires a rebuild
	// whenever a space's `.plan/` changes, so a map created outside chartr
	// appears on its own. rebuild also reconciles the watch set, so this starts
	// covering whatever the persisted registry already holds.
	//
	// The config root is pinned into the same watch. It is nobody's space, but
	// every rebuild re-reads `user.toml`, `terminal.toml` and `notify.toml` out of it, so an
	// operator saving a config edit in their own editor — the only way those files
	// are ever edited, since the surface opens rather than edits them — is the same
	// kind of notice a map write is, and reaches live terminals without a refresh.
	if opts.NoWatch {
		s.watch = deadWatcher(opts.ConfigDir)
	} else {
		s.watch = newWatcher(s.rebuild, opts.ConfigDir)
	}
	// Ad-hoc shells are chartr-owned runtime state (ticket 05). The manager
	// pushes a fresh model whenever a terminal opens or ends, so a tab appears
	// and disappears on its own; the model is built before the first rebuild.
	s.terms = terminal.NewManager(s.rebuild, s.onRunFinished)

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
	// The sidebar arrangement, as one write of the whole ordered list. It is not
	// addressed by space because it is not a per-row move: the drag and the
	// keyboard both send the complete list, which is idempotent and matches the
	// dense 0..n-1 order every save writes.
	s.mux.HandleFunc("POST /api/spaces/reorder", s.handleReorder)
	s.mux.HandleFunc("DELETE /api/spaces/{id}", s.handleDeregister)
	// Open the registered repository in the operator's folder browser. The id is
	// resolved server-side; no path supplied by the page reaches the OS opener.
	s.mux.HandleFunc("POST /api/spaces/{id}/open", s.handleOpenSpace)
	// The remembered agent, set directly rather than only as a side effect of a
	// spawn — the action footer's agent selector persists the operator's pick the
	// moment they make it, so it reads as a saved setting, not a pending choice.
	s.mux.HandleFunc("PUT /api/spaces/{id}/agent", s.handleSetSpaceAgent)
	// The operator's own display name for a space, persisted per-path in the
	// registry. Presentation only — it renames nothing on disk — so it is a plain
	// setter beside the agent one above; an empty name clears the override back to
	// the folder basename.
	s.mux.HandleFunc("PUT /api/spaces/{id}/name", s.handleRenameSpace)
	// The settings surface (ticket 05). The read half rides the model push, so
	// opening a config file is the only write it gets. It is a POST because it
	// launches a process, and it resolves a *named* layer server-side — never a
	// path from the client. Every layer is the operator's own config file,
	// reachable with nothing registered, so it never borrows a space id.
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
	// The skill-source list (ticket 10): the six actions the sources section
	// offers, all global for the same reason the agent library is. Everything
	// else about `sources.toml` is an open-the-file action on the named layer
	// above — these routes are the whole of what the surface edits.
	s.mux.HandleFunc("POST /api/config/sources", s.handleRegisterSource)
	s.mux.HandleFunc("POST /api/config/sources/reorder", s.handleReorderSources)
	s.mux.HandleFunc("DELETE /api/config/sources/{name}", s.handleRemoveSource)
	s.mux.HandleFunc("PUT /api/config/sources/{name}/enabled", s.handleSetSourceEnabled)
	s.mux.HandleFunc("POST /api/config/sources/{name}/refresh", s.handleRefreshSource)
	// The role bindings row's link button: open a resolved `Source/skill`. A
	// POST for the same reason /api/config/open is — a dir source's answer
	// launches a process — and it resolves the ref server-side through the
	// registry, never a path the client names.
	s.mux.HandleFunc("POST /api/config/sources/open", s.handleOpenSkill)
	// The role picker's one write: bind a role to a chosen skill or to `auto`.
	s.mux.HandleFunc("PUT /api/config/roles/{role}", s.handleSetRoleBinding)
	// Payload preview (ticket 08): for a chosen ticket and role, exactly what a
	// session would be told, with per-part origin provenance. Read-only, so a GET;
	// the composition reads the sources and the map fresh off disk each time.
	s.mux.HandleFunc("GET /api/spaces/{id}/maps/{slug}/tickets/{num}/payload", s.handlePayloadPreview)
	// Spawn a session (ticket 09): the tracer bullet. From a frontier ticket, the
	// chartr writes the claim, composes and archives the payload, settles the
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
	// Release addressed by ticket rather than by session — the way back for a claim
	// whose session tab is gone (a chartr restart, a dismissed dead tab, a claim
	// committed on another machine). Same write and same commit as the halt's
	// release; refused while a live session in this space still holds the ticket.
	s.mux.HandleFunc("POST /api/spaces/{id}/maps/{slug}/tickets/{num}/release", s.handleReleaseTicket)
	// Ad-hoc shells: open one in the space's working tree, end one by the human's
	// command. Opening is a plain HTTP action so a spawn failure surfaces as a
	// response (ADR 0010); the shell itself lives on the terminal socket.
	s.mux.HandleFunc("POST /api/spaces/{id}/terminals", s.handleOpenTerminal)
	// The operator's own arrangement of a space's session tabs (scoped to the one
	// card): the whole ordered list of that space's terminal ids, the same
	// whole-list shape the sidebar reorder takes.
	s.mux.HandleFunc("POST /api/spaces/{id}/terminals/reorder", s.handleReorderTerminals)
	s.mux.HandleFunc("DELETE /api/spaces/{id}/terminals/{termID}", s.handleCloseTerminal)
	// The acknowledgement of a run that finished while the operator was elsewhere
	// (session-notifications, the dot): focusing a tab clears its dot in the
	// snapshot. It is a POST because it writes state, and it is the *only* way the
	// dot clears — there is no manual dismiss and no clear-all, which is what keeps
	// a stale dot unrepresentable.
	s.mux.HandleFunc("POST /api/spaces/{id}/terminals/{termID}/seen", s.handleTerminalSeen)
	// The new-shell control's agent rows: start a free session on a chosen agent —
	// a live, ticketless tab told the free payload, with no map or ticket lookup, no
	// claim, no lifecycle, ended only by the human, exactly like an ad-hoc shell.
	s.mux.HandleFunc("POST /api/spaces/{id}/launch", s.handleFree)
	// The operator's manual mirror refresh (the "Sync skills" control): reconcile
	// this space's `.chartr/skills` against the enabled sources now, rather than
	// waiting on the next spawn barrier — the prewarm for an ad-hoc session chartr
	// did not spawn. Correctness still rides on the barrier; this only brings the
	// refresh forward.
	s.mux.HandleFunc("POST /api/spaces/{id}/skills/sync", s.handleSyncSkills)
	// Everything else is the embedded SPA, with a client-routing fallback.
	s.mux.Handle("/", spaHandler(dist))

	// Reflect any registry persisted from a prior run in the first snapshot, so
	// a restart restores the operator's spaces without a re-register.
	s.rebuild()

	// The standing `CHARTR.md` at each registered space's root (chartr-md, ticket
	// 01). It goes here rather than in firstRun because firstRun takes a config
	// root and knows nothing about the space registry; both it and the source list
	// are loaded above, and this runs before the server serves.
	s.reconcileChartrDoc()

	return s, nil
}

// ConfigRoot resolves chartr's config root to `~/.config/chartr`, honouring
// `XDG_CONFIG_HOME` when it is set. This is deliberately *not* os.UserConfigDir:
// that returns `~/Library/Application Support` on macOS, and we want one path an
// operator can reason about on every platform. It falls back to the runtime root
// only when there is no home directory to anchor to (a stripped-down environment).
//
// It is exported because the macOS shell needs the same answer *before* it
// constructs a Server: a bundled launch is handed `/` as its working directory,
// so it anchors its runtime root here instead. One function is what keeps the
// bundled app and the command-line binary from forking an operator's state.
func ConfigRoot(fallback string) string {
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

	// Scope both websocket handshakes to the address we are actually listening on,
	// so a page on another origin can neither read the model snapshot nor type
	// into a terminal. This is written before the listener is served, and every
	// handler that reads it runs on a goroutine started after, so the write is
	// ordered ahead of every read without a lock.
	s.origins = originPatterns(ln.Addr())

	// A bind that is not loopback puts an unauthenticated API — shells, agent
	// spawns, the operator's files — in front of everyone who can reach the port,
	// and neither the origin patterns above nor the host gate below stand in the
	// way of a client that is not a browser. Nothing is refused here: an operator
	// who meant it keeps working, and they are told what they exposed.
	if exposedBind(ln.Addr()) {
		log.Print(exposureWarning(ln.Addr()))
	}

	httpSrv := &http.Server{
		// Every route goes through the gate first — the checks that hold whatever
		// the route is (gate): the Host must name the address we bound, and a write
		// must declare a JSON body. Scoping the sockets to the bound origin above is
		// not enough on its own: a DNS-rebinding name makes the browser believe the
		// attacker's origin *is* this address, and the only thing that still gives
		// it away is the name it asks for.
		Handler:           gate(ln.Addr(), s.mux),
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
//
// `/api/` is excluded from the fallback: an unrouted API path is a missing
// endpoint, not a client route, and answering it with the app's index page turns
// a deleted route (`/api/spaces/{id}/pin`) into a silent 200 for whoever still
// calls it. It 404s instead.
func spaHandler(dist fs.FS) http.Handler {
	files := http.FileServer(http.FS(dist))
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path == "/" || fileExists(dist, r.URL.Path) {
			files.ServeHTTP(w, r)
			return
		}
		if strings.HasPrefix(r.URL.Path, "/api/") {
			http.NotFound(w, r)
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
