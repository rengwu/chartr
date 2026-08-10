// Package chartrtest is the process-boundary test rig (spec, Testing
// Decisions). It starts the real chartr against a temporary space and lets a
// test drive it exactly as an operator would — over HTTP and the control
// socket — asserting only on what the design makes public: snapshots, the files
// in .plan/, and git history. No test reaches into chartr internals; the one
// seam is the process boundary, and this package is how tests reach it.
//
// The rig runs the server in-process on a real TCP port through the same
// server.New path cmd/chartr uses, so the tested code path is the operator's.
// Later tickets extend this package (fixture maps, stub agents on PATH) rather
// than adding new seams.
package chartrtest

import (
	"bufio"
	"bytes"
	"context"
	"crypto/rand"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net"
	"net/http"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/coder/websocket"

	"github.com/rengwu/chartr/internal/model"
	"github.com/rengwu/chartr/internal/notify"
	"github.com/rengwu/chartr/internal/server"
)

// Chartr is a running chartr backend under test.
type Chartr struct {
	// BaseURL is the http origin the server is listening on, e.g.
	// http://127.0.0.1:54321.
	BaseURL string
	// DataDir is the chartr-owned runtime root the server was given (a temp dir
	// unless overridden) — `sessions/` and nothing else.
	DataDir string
	// ConfigDir is the operator's local config root the server was given (a temp
	// dir unless overridden): the registry (`spaces.toml`), agent library
	// (`user.toml`), terminal customization (`terminal.toml`), the source list
	// (`sources.toml`) and the skill sources under `sources/`. It is a temp dir by
	// default so a run never reads — or is coloured by — the developer's own
	// config.
	ConfigDir string

	t    testing.TB
	logs *safeBuffer
}

// Option configures Start.
type Option func(*startOptions)

// startOptions is everything Start takes: what the server is constructed with,
// and what it is told to listen on. The bind address is not a server.Option —
// the listener is main's to create, and the rig makes the same one — so the two
// travel together here rather than Start growing a second parameter.
type startOptions struct {
	server server.Options
	addr   string
	// configOverridden records that a test brought its own config root, which
	// suppresses the default skill-source seeding: such a test drives first-run,
	// migration, or empty-list behaviour and manages its own sources.
	configOverridden bool
	// noSkills suppresses the default seeding even for a temp config root, for the
	// rare test that wants a genuinely empty source list.
	noSkills bool
}

// WithConfigDir overrides the operator's config root (default: a fresh temp
// dir). Point it at a root pre-populated the old way to drive the migration. A
// test that brings its own root also opts out of the default skill seeding.
func WithConfigDir(dir string) Option {
	return func(o *startOptions) {
		o.server.ConfigDir = dir
		o.configOverridden = true
	}
}

// WithDefaultSource points the fresh-install pre-registration at a git URL, the
// way the real entrypoints point it at chartr's own skills repo. The rig leaves
// it empty otherwise, so the default suite never clones over the network; a test
// that wants to drive the pre-registration passes a local (e.g. `file://`) repo
// here. Pair it with WithConfigDir so the run is a genuine first run.
func WithDefaultSource(url string) Option {
	return func(o *startOptions) { o.server.DefaultSourceURL = url }
}

// WithoutSkills starts with a genuinely empty source list — no seeded skills and
// no bindings. chartr ships none of its own (ADR 0018); Start otherwise stands in
// for the operator's one-time setup by registering a `chartr-skills` dir source
// and binding the four roles, which is the environment most tests assume.
func WithoutSkills() Option {
	return func(o *startOptions) { o.noSkills = true }
}

// WithBindAddress starts chartr on a given address rather than the default
// ephemeral loopback port — the operator's `-addr` (websocket-origin-fix, ticket
// 04). `0.0.0.0:0` is the wildcard bind the README documents as `-addr :9000`,
// on a port the test does not have to pick.
//
// A test using this is asserting on what the *bind* changes, not on a route:
// the rig's BaseURL still dials loopback, because a wildcard-bound chartr is
// listening there too.
func WithBindAddress(addr string) Option {
	return func(o *startOptions) { o.addr = addr }
}

// WithNotifier substitutes the OS notifier a run's end is announced through
// (session-notifications, ticket 03). It is the one seam that has to be swapped
// rather than driven: the real notifier execs the machine's notification tool, and
// no test shells out to that on any platform. A test that does not care leaves the
// platform notifier in place — nothing in the suite runs long enough to reach the
// shipped 60-second threshold, so it is never called.
func WithNotifier(n notify.Notifier) Option {
	return func(o *startOptions) { o.server.Notifier = n }
}

// WithoutWatch starts the server with its filesystem watch dead — the degraded
// state chartr falls back to when the OS watcher cannot be created. Nothing
// written from outside fires a notice, so a test using this is asserting on the
// discovery that happens without one (ticket 19).
func WithoutWatch() Option {
	return func(o *startOptions) { o.server.NoWatch = true }
}

// Start launches chartr on a random loopback port and registers cleanup that
// shuts it down when the test ends. It fails the test on any startup error.
func Start(t testing.TB, opts ...Option) *Chartr {
	t.Helper()

	sopts := startOptions{
		server: server.Options{DataDir: t.TempDir(), ConfigDir: t.TempDir()},
		addr:   "127.0.0.1:0",
	}
	for _, opt := range opts {
		opt(&sopts)
	}

	srv, err := server.New(sopts.server)
	if err != nil {
		t.Fatalf("chartrtest: server.New: %v", err)
	}

	ln, err := net.Listen("tcp", sopts.addr)
	if err != nil {
		t.Fatalf("chartrtest: listen on %s: %v", sopts.addr, err)
	}

	// Everything the server logs while this rig is up, kept for Logged() and
	// still passed through to wherever it was going, so a failing test's output
	// is not swallowed by the rig capturing it. The restore is registered before
	// the shutdown cleanup so it runs after it (cleanups are LIFO) and a line
	// logged on the way down is captured too.
	logs := &safeBuffer{}
	prevLog := log.Writer()
	log.SetOutput(io.MultiWriter(prevLog, logs))
	t.Cleanup(func() { log.SetOutput(prevLog) })

	ctx, cancel := context.WithCancel(context.Background())
	done := make(chan error, 1)
	go func() { done <- srv.Serve(ctx, ln) }()

	t.Cleanup(func() {
		cancel()
		select {
		case err := <-done:
			if err != nil {
				t.Errorf("chartrtest: server exited with error: %v", err)
			}
		case <-time.After(5 * time.Second):
			t.Error("chartrtest: server did not shut down within 5s")
		}
	})

	h := &Chartr{
		BaseURL:   "http://" + dialableAuthority(ln.Addr()),
		DataDir:   sopts.server.DataDir,
		ConfigDir: sopts.server.ConfigDir,
		t:         t,
		logs:      logs,
	}

	// chartr ships no skills (ADR 0018), so a fresh install has an empty source
	// list and unbound roles. The old test environment always had chartr's seeded
	// source and four bindings, and most tests are written against that; Start
	// recreates it — a `chartr-skills` dir source and the four bindings — for a
	// default (temp) config root. A test with its own root, or WithoutSkills, gets
	// the genuine empty first-run state instead.
	if !sopts.configOverridden && !sopts.noSkills {
		h.SeedSkills(t)
	}

	return h
}

// SkillSource is the name Start (and SeedSkills) registers the stand-in skill
// source under. It is an ordinary `dir` source the operator could have registered
// themselves — chartr no longer ships one.
const SkillSource = "chartr-skills"

// SeedSkills stands in for the operator's one-time setup: it registers a `dir`
// source of the standard role skills and binds the four roles into it. It is what
// Start does by default, exposed so a WithConfigDir test can opt back into a
// working skill environment. Bindings are written straight to `user.toml`, which
// every composition re-reads.
func (h *Chartr) SeedSkills(t testing.TB) {
	t.Helper()
	dir := t.TempDir()
	// One body per skill, each carrying a marker a test can read; the implement
	// skill names the implementation map, which a payload test asserts reaches the
	// composed document.
	bodies := map[string]string{
		"grill":      "Grill the plan until only load-bearing decisions remain.",
		"prototype":  "Prototype the risky part first.",
		"research":   "Research the question and write what you found.",
		"implement":  "Implement the ticket against its implementation map.",
		"wayfinder":  "Chart a wayfinder map of tickets.",
		"to-spec":    "Turn the interview into a spec.",
		"to-tickets": "Decompose the spec into tickets.",
	}
	for name, body := range bodies {
		d := filepath.Join(dir, name)
		if err := os.MkdirAll(d, 0o755); err != nil {
			t.Fatalf("chartrtest: seeding skill %s: %v", name, err)
		}
		src := "---\nname: " + name + "\ndescription: the " + name + " skill\n---\n\n" + body + "\n"
		if err := os.WriteFile(filepath.Join(d, "SKILL.md"), []byte(src), 0o644); err != nil {
			t.Fatalf("chartrtest: seeding skill %s: %v", name, err)
		}
	}
	if code, body := h.Post("/api/config/sources", map[string]string{
		"name": SkillSource, "kind": "dir", "path": dir,
	}); code != 200 {
		t.Fatalf("chartrtest: registering the skills source: %d %s", code, body)
	}

	// Bind the four roles into it, preserving anything already in user.toml.
	path := filepath.Join(h.ConfigDir, "user.toml")
	existing, _ := os.ReadFile(path)
	roles := "\n[roles]\ngrill = \"" + SkillSource + "/grill\"\nprototype = \"" + SkillSource +
		"/prototype\"\nresearch = \"" + SkillSource + "/research\"\nimplement = \"" + SkillSource + "/implement\"\n"
	if err := os.WriteFile(path, append(existing, []byte(roles)...), 0o600); err != nil {
		t.Fatalf("chartrtest: writing role bindings: %v", err)
	}
}

// Logged is everything chartr has logged since this rig started — the channel a
// startup warning arrives on (websocket-origin-fix, ticket 04). It is the
// operator's own stderr, which makes it public surface rather than an internal
// seam: a test reads exactly what they would see.
//
// Serve runs on its own goroutine here, so a test asserting a line is *absent*
// must first do something that proves the server got past the point that logs it
// — a request answered is proof enough.
func (h *Chartr) Logged() string {
	return h.logs.String()
}

// dialableAuthority is the host:port a test dials to reach a listener. A
// wildcard bind reports itself as `0.0.0.0:PORT`, which is an address to listen
// on rather than one to connect to; it is listening on loopback as well, so that
// is what the rig talks to.
func dialableAuthority(addr net.Addr) string {
	host, port, err := net.SplitHostPort(addr.String())
	if err != nil {
		return addr.String()
	}
	if ip := net.ParseIP(host); ip != nil && ip.IsUnspecified() {
		host = "127.0.0.1"
	}
	return net.JoinHostPort(host, port)
}

// safeBuffer is a bytes.Buffer a test can read while the server's goroutines are
// still writing to it.
type safeBuffer struct {
	mu  sync.Mutex
	buf bytes.Buffer
}

func (b *safeBuffer) Write(p []byte) (int, error) {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.buf.Write(p)
}

func (b *safeBuffer) String() string {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.buf.String()
}

// Get performs a GET and returns the status code and body. It fails the test on
// a transport error.
func (h *Chartr) Get(path string) (int, string) {
	h.t.Helper()
	resp, err := http.Get(h.BaseURL + path)
	if err != nil {
		h.t.Fatalf("chartrtest: GET %s: %v", path, err)
	}
	defer resp.Body.Close()
	body, _ := io.ReadAll(resp.Body)
	return resp.StatusCode, string(body)
}

// GetWithHost performs a GET that asks for host, whatever address it dials — the
// request a browser makes once a DNS-rebinding name has been re-pointed at
// chartr's listening socket. It returns the status code and body.
//
// The browser is not lying when it does this: it really did connect to us, and
// it really does believe that name is this origin. The Host header is the only
// place the substitution shows (websocket-origin-fix, ticket 02).
func (h *Chartr) GetWithHost(path, host string) (int, string) {
	h.t.Helper()
	req, err := http.NewRequest(http.MethodGet, h.BaseURL+path, nil)
	if err != nil {
		h.t.Fatalf("chartrtest: build GET %s: %v", path, err)
	}
	// Request.Host, not a header: net/http writes the header from this field and
	// ignores Header["Host"] entirely.
	req.Host = host
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		h.t.Fatalf("chartrtest: GET %s as %s: %v", path, host, err)
	}
	defer resp.Body.Close()
	out, _ := io.ReadAll(resp.Body)
	return resp.StatusCode, string(out)
}

// Post performs a JSON POST — an operator action (ADR 0010) — and returns the
// status code and body. A nil body sends no request body.
func (h *Chartr) Post(path string, body any) (int, string) {
	h.t.Helper()
	var r io.Reader
	if body != nil {
		b, err := json.Marshal(body)
		if err != nil {
			h.t.Fatalf("chartrtest: marshal POST body: %v", err)
		}
		r = bytes.NewReader(b)
	}
	resp, err := http.Post(h.BaseURL+path, "application/json", r)
	if err != nil {
		h.t.Fatalf("chartrtest: POST %s: %v", path, err)
	}
	defer resp.Body.Close()
	out, _ := io.ReadAll(resp.Body)
	return resp.StatusCode, string(out)
}

// PostWithContentType posts body verbatim under a Content-Type of the caller's
// choosing — an empty contentType omits the header entirely, and an empty body
// sends none — and returns the status code and body.
//
// It is how a test makes the request a cross-origin page can send with *no
// preflight*: a POST whose content type is one of the three CORS-simple ones, or
// a bare `fetch(url, {method: 'POST'})` that carries neither body nor header.
// The browser delivers those without asking the server first, so the only thing
// that can refuse them is the server acting on what they declare
// (websocket-origin-fix, ticket 03).
func (h *Chartr) PostWithContentType(path, contentType, body string) (int, string) {
	h.t.Helper()
	var r io.Reader
	if body != "" {
		r = strings.NewReader(body)
	}
	req, err := http.NewRequest(http.MethodPost, h.BaseURL+path, r)
	if err != nil {
		h.t.Fatalf("chartrtest: build POST %s: %v", path, err)
	}
	if contentType != "" {
		req.Header.Set("Content-Type", contentType)
	}
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		h.t.Fatalf("chartrtest: POST %s as %q: %v", path, contentType, err)
	}
	defer resp.Body.Close()
	out, _ := io.ReadAll(resp.Body)
	return resp.StatusCode, string(out)
}

// Put performs a JSON PUT — the shape an action that sets one named field to one
// value takes (the transparency surface's binding edit, ticket 05) — and returns
// the status code and body.
func (h *Chartr) Put(path string, body any) (int, string) {
	h.t.Helper()
	b, err := json.Marshal(body)
	if err != nil {
		h.t.Fatalf("chartrtest: marshal PUT body: %v", err)
	}
	req, err := http.NewRequest(http.MethodPut, h.BaseURL+path, bytes.NewReader(b))
	if err != nil {
		h.t.Fatalf("chartrtest: build PUT %s: %v", path, err)
	}
	req.Header.Set("Content-Type", "application/json")
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		h.t.Fatalf("chartrtest: PUT %s: %v", path, err)
	}
	defer resp.Body.Close()
	out, _ := io.ReadAll(resp.Body)
	return resp.StatusCode, string(out)
}

// Delete performs a DELETE and returns the status code and body.
func (h *Chartr) Delete(path string) (int, string) {
	h.t.Helper()
	req, err := http.NewRequest(http.MethodDelete, h.BaseURL+path, nil)
	if err != nil {
		h.t.Fatalf("chartrtest: build DELETE %s: %v", path, err)
	}
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		h.t.Fatalf("chartrtest: DELETE %s: %v", path, err)
	}
	defer resp.Body.Close()
	out, _ := io.ReadAll(resp.Body)
	return resp.StatusCode, string(out)
}

// OpenTerminal opens an ad-hoc shell in the space and returns its terminal id.
// It fails the test on a non-200 response.
func (h *Chartr) OpenTerminal(spaceID string) string {
	h.t.Helper()
	code, body := h.Post("/api/spaces/"+spaceID+"/terminals", nil)
	if code != 200 {
		h.t.Fatalf("chartrtest: open terminal in %s = %d, body %s", spaceID, code, body)
	}
	var r struct {
		ID string `json:"id"`
	}
	if err := json.Unmarshal([]byte(body), &r); err != nil {
		h.t.Fatalf("chartrtest: open-terminal response not JSON: %v (%q)", err, body)
	}
	return r.ID
}

// Launch starts a free session in the space on a named registered agent and
// returns its tab id. A free session names its agent like every other spawn
// (agent-selection ticket 03), so — unlike Spawn — there is one helper and it
// always says which: there is no role behind it to fall back to. It fails the test
// on a non-200; LaunchRaw is the way to assert a refusal.
func (h *Chartr) Launch(spaceID, agent string) string {
	h.t.Helper()
	code, body := h.LaunchRaw(spaceID, agent)
	if code != 200 {
		h.t.Fatalf("chartrtest: free session in %s with %q = %d, body %s", spaceID, agent, code, body)
	}
	var r struct {
		ID string `json:"id"`
	}
	if err := json.Unmarshal([]byte(body), &r); err != nil {
		h.t.Fatalf("chartrtest: free-session response not JSON: %v (%q)", err, body)
	}
	return r.ID
}

// LaunchRaw posts the free-session action and returns the status code and body, so
// a test can assert a refusal — an unregistered agent, or one whose binary is gone.
func (h *Chartr) LaunchRaw(spaceID, agent string) (int, string) {
	h.t.Helper()
	return h.Post("/api/spaces/"+spaceID+"/launch", map[string]string{"agent": agent})
}

// Spawn posts a spawn action for a ticket and role and returns the status code
// and body — the operator's one-click "start work here" (ticket 09). A test drives
// it directly so it can assert the whole chain the response kicks off: the claim
// commit, the payload, and the live session tab.
func (h *Chartr) Spawn(spaceID, slug string, num int, role string) (int, string) {
	h.t.Helper()
	return h.Post(fmt.Sprintf("/api/spaces/%s/maps/%s/tickets/%d/spawn", spaceID, slug, num),
		map[string]string{"role": role})
}

// SpawnWithAgent posts a spawn that names the registered agent to run it — what
// the picker sends, as against Spawn's role-only request. The two are separate
// helpers rather than one variadic so a test says which shape of request it is
// making, and so the role-only path keeps a caller proving it still resolves
// through the binding.
func (h *Chartr) SpawnWithAgent(spaceID, slug string, num int, role, agent string) (int, string) {
	h.t.Helper()
	return h.Post(fmt.Sprintf("/api/spaces/%s/maps/%s/tickets/%d/spawn", spaceID, slug, num),
		map[string]string{"role": role, "agent": agent})
}

// SpawnForced posts the spawn the operator sends after confirming the concurrency
// warning: the same request as SpawnWithAgent, plus the force that overrules the
// one-live-session-per-space gate (ADR 0003 as amended). It is its own helper for
// the same reason SpawnWithAgent is — a test says which shape of request it makes.
func (h *Chartr) SpawnForced(spaceID, slug string, num int, role, agent string) (int, string) {
	h.t.Helper()
	return h.Post(fmt.Sprintf("/api/spaces/%s/maps/%s/tickets/%d/spawn", spaceID, slug, num),
		map[string]any{"role": role, "agent": agent, "force": true})
}

// Resume, Respawn, and Release drive the three death-halt choices for a pinned
// dead session (ticket 10). Each is a plain HTTP action so a test asserts that the
// halt takes none of them on its own and that each does exactly its one thing.
func (h *Chartr) Resume(spaceID, sessionID string) (int, string) {
	h.t.Helper()
	return h.Post(fmt.Sprintf("/api/spaces/%s/sessions/%s/resume", spaceID, sessionID), nil)
}

func (h *Chartr) Respawn(spaceID, sessionID string) (int, string) {
	h.t.Helper()
	return h.Post(fmt.Sprintf("/api/spaces/%s/sessions/%s/respawn", spaceID, sessionID), nil)
}

func (h *Chartr) Release(spaceID, sessionID string) (int, string) {
	h.t.Helper()
	return h.Post(fmt.Sprintf("/api/spaces/%s/sessions/%s/release", spaceID, sessionID), nil)
}

// ReleaseTicket drives the release addressed by ticket rather than by session —
// the one that reaches a claim whose session tab no longer exists. It is a
// separate helper from Release so a test says which door it came in by.
func (h *Chartr) ReleaseTicket(spaceID, slug string, num int) (int, string) {
	h.t.Helper()
	return h.Post(fmt.Sprintf("/api/spaces/%s/maps/%s/tickets/%d/release", spaceID, slug, num), nil)
}

// Snapshot connects a control socket, reads exactly one whole snapshot, and
// closes it. Because operator actions push the new model before their HTTP
// response returns, a snapshot taken after an action already reflects it.
func (h *Chartr) Snapshot(ctx context.Context) model.Model {
	h.t.Helper()
	conn := h.DialControl(ctx)
	defer conn.Close()
	return conn.ReadSnapshot(ctx)
}

// SnapshotUntil polls fresh snapshots until pred holds, returning the matching
// one. Each poll dials a new control socket, which the server answers with the
// current model immediately, so this suits an act-then-wait test that performs an
// action (or triggers an async one, like a stub agent dying) and then waits for
// the resulting state without having dialled beforehand. It fails the test if ctx
// expires first.
func (h *Chartr) SnapshotUntil(ctx context.Context, pred func(model.Model) bool) model.Model {
	h.t.Helper()
	for {
		m := h.Snapshot(ctx)
		if pred(m) {
			return m
		}
		select {
		case <-ctx.Done():
			h.t.Fatalf("chartrtest: snapshot never matched predicate: %v", ctx.Err())
		case <-time.After(30 * time.Millisecond):
		}
	}
}

// DialControl connects a control socket and returns it. The caller closes it.
func (h *Chartr) DialControl(ctx context.Context) *ControlConn {
	h.t.Helper()
	wsURL := "ws" + strings.TrimPrefix(h.BaseURL, "http") + "/ws/control"
	c, _, err := websocket.Dial(ctx, wsURL, nil)
	if err != nil {
		h.t.Fatalf("chartrtest: dial control socket: %v", err)
	}
	return &ControlConn{c: c, t: h.t}
}

// DialTerminal connects the binary terminal socket for a terminal id and returns
// it. The caller closes it. It fails the test if the socket cannot be dialled
// (e.g. the terminal does not exist).
func (h *Chartr) DialTerminal(ctx context.Context, termID string) *TerminalConn {
	h.t.Helper()
	wsURL := "ws" + strings.TrimPrefix(h.BaseURL, "http") + "/ws/terminal/" + termID
	c, _, err := websocket.Dial(ctx, wsURL, nil)
	if err != nil {
		h.t.Fatalf("chartrtest: dial terminal socket %s: %v", termID, err)
	}
	return &TerminalConn{c: c, t: h.t}
}

// DialControlAt dials the control socket by way of a given authority
// (`localhost:PORT`) rather than the rig's own BaseURL, and returns the
// connection together with the handshake status — the connection is nil exactly
// when it was refused. The authority has to resolve to the listening socket;
// this is the operator typing another spelling of the same address into their
// browser, not a forgery, so Host and Origin both carry it the way a real browse
// does.
func (h *Chartr) DialControlAt(ctx context.Context, authority string) (*ControlConn, int) {
	h.t.Helper()
	c, resp, err := websocket.Dial(ctx, "ws://"+authority+"/ws/control", nil)
	status := 0
	if resp != nil {
		status = resp.StatusCode
	}
	if err != nil {
		return nil, status
	}
	return &ControlConn{c: c, t: h.t}, status
}

// DialControlOrigin dials the control socket presenting origin as the browser's
// Origin header, and returns the connection together with the handshake's HTTP
// status. Unlike DialControl it does not fail the test on a refusal: a refusal is
// the result a test is usually after here — this is how the suite asserts that a
// page on another origin is turned away (websocket-origin-fix, ticket 01). The
// connection is nil exactly when the handshake was refused.
func (h *Chartr) DialControlOrigin(ctx context.Context, origin string) (*ControlConn, int) {
	h.t.Helper()
	c, status := h.dialOrigin(ctx, "/ws/control", origin)
	if c == nil {
		return nil, status
	}
	return &ControlConn{c: c, t: h.t}, status
}

// DialTerminalOrigin is DialControlOrigin for a terminal's binary socket: it
// presents origin and returns the connection (nil when refused) with the
// handshake status.
func (h *Chartr) DialTerminalOrigin(ctx context.Context, termID, origin string) (*TerminalConn, int) {
	h.t.Helper()
	c, status := h.dialOrigin(ctx, "/ws/terminal/"+termID, origin)
	if c == nil {
		return nil, status
	}
	return &TerminalConn{c: c, t: h.t}, status
}

func (h *Chartr) dialOrigin(ctx context.Context, path, origin string) (*websocket.Conn, int) {
	h.t.Helper()
	wsURL := "ws" + strings.TrimPrefix(h.BaseURL, "http") + path
	c, resp, err := websocket.Dial(ctx, wsURL, &websocket.DialOptions{
		HTTPHeader: http.Header{"Origin": []string{origin}},
	})
	status := 0
	if resp != nil {
		status = resp.StatusCode
	}
	if err != nil {
		return nil, status
	}
	return c, status
}

// AttemptCrossOriginTerminalWrite performs the terminal handshake by hand over a
// raw TCP connection — presenting origin as a browser would, and pipelining a
// masked binary frame carrying keys immediately after the request, without
// waiting for the response. It returns the handshake response's HTTP status.
//
// It is deliberately not a websocket.Dial. A client library will not write a
// frame onto a handshake it already knows was refused, so a test built on one
// could not tell an outright rejection from a server that accepts the socket and
// then closes it — and the second is the regression worth pinning
// (websocket-origin-fix, ticket 01). This always puts the bytes on the wire; the
// assertion is that they never reach the PTY.
func (h *Chartr) AttemptCrossOriginTerminalWrite(termID, origin, keys string) int {
	h.t.Helper()
	return h.rawHandshake("/ws/terminal/"+termID, h.authority(), origin, h.clientBinaryFrame([]byte(keys)))
}

// HandshakeWithHost performs the websocket handshake for path over a raw TCP
// connection to the address chartr bound, presenting host as the request's Host
// header and `http://host` as its Origin, and returns the handshake's HTTP
// status (101 when the socket was opened).
//
// That pairing is the point, and it is what a DNS-rebinding attacker's browser
// actually sends: it believes the attacker's name *is* this address, so it sets
// both headers to it and they agree. coder/websocket authorizes any handshake
// whose Origin equals its Host regardless of the origin patterns, so nothing in
// websocket-origin-fix ticket 01 stands here — only the Host check does
// (ticket 02).
//
// It is done by hand rather than with websocket.Dial because net/http drops a
// Host set through a header map: a client only sends a Host that disagrees with
// the address it dialled if the request was built with Request.Host, which
// DialOptions has no way to reach.
func (h *Chartr) HandshakeWithHost(path, host string) int {
	h.t.Helper()
	return h.rawHandshake(path, host, "http://"+host, nil)
}

// rawHandshake writes one websocket handshake for path over a raw TCP
// connection to the bound address, presenting host and origin verbatim, and
// appends trailer to the same write — so anything in it is on the wire before
// the response has been read, let alone acted on. It returns the response's
// status.
func (h *Chartr) rawHandshake(path, host, origin string, trailer []byte) int {
	h.t.Helper()

	conn, err := net.DialTimeout("tcp", h.authority(), 5*time.Second)
	if err != nil {
		h.t.Fatalf("chartrtest: dial %s: %v", h.authority(), err)
	}
	defer conn.Close()
	_ = conn.SetDeadline(time.Now().Add(5 * time.Second))

	var key [16]byte
	if _, err := rand.Read(key[:]); err != nil {
		h.t.Fatalf("chartrtest: generating a Sec-WebSocket-Key: %v", err)
	}
	req := "GET " + path + " HTTP/1.1\r\n" +
		"Host: " + host + "\r\n" +
		"Upgrade: websocket\r\n" +
		"Connection: Upgrade\r\n" +
		"Sec-WebSocket-Version: 13\r\n" +
		"Sec-WebSocket-Key: " + base64.StdEncoding.EncodeToString(key[:]) + "\r\n" +
		"Origin: " + origin + "\r\n" +
		"\r\n"
	if _, err := conn.Write(append([]byte(req), trailer...)); err != nil {
		h.t.Fatalf("chartrtest: writing the handshake for %s: %v", path, err)
	}

	resp, err := http.ReadResponse(bufio.NewReader(conn), &http.Request{Method: http.MethodGet})
	if err != nil {
		h.t.Fatalf("chartrtest: reading the handshake response for %s: %v", path, err)
	}
	defer resp.Body.Close()
	return resp.StatusCode
}

// authority is the host:port chartr bound, the way a URL spells it.
func (h *Chartr) authority() string {
	return strings.TrimPrefix(h.BaseURL, "http://")
}

// clientBinaryFrame encodes one masked client binary frame — the bytes a browser
// puts on the wire for a keystroke. Only the 7-bit length form is encoded, which
// is all a probe payload needs.
func (h *Chartr) clientBinaryFrame(payload []byte) []byte {
	h.t.Helper()
	if len(payload) > 125 {
		h.t.Fatalf("chartrtest: cross-origin probe payload is %d bytes, must be under 126", len(payload))
	}
	var mask [4]byte
	if _, err := rand.Read(mask[:]); err != nil {
		h.t.Fatalf("chartrtest: generating a frame mask: %v", err)
	}
	// 0x82: FIN set, opcode 2 (binary). 0x80: the payload is masked, as every
	// client frame must be.
	frame := []byte{0x82, 0x80 | byte(len(payload))}
	frame = append(frame, mask[:]...)
	for i, b := range payload {
		frame = append(frame, b^mask[i%4])
	}
	return frame
}

// TerminalConn is a connected terminal socket in a test. Raw PTY bytes arrive as
// binary frames down; keystrokes go up as binary frames; resize goes up as a
// text control frame.
type TerminalConn struct {
	c *websocket.Conn
	t testing.TB
}

// Send writes keystrokes up to the shell as a binary frame.
func (tc *TerminalConn) Send(ctx context.Context, keys string) {
	tc.t.Helper()
	if err := tc.c.Write(ctx, websocket.MessageBinary, []byte(keys)); err != nil {
		tc.t.Fatalf("chartrtest: send keystrokes: %v", err)
	}
}

// ReadUntil reads down-frames, accumulating them, until the accumulated output
// contains want, and returns everything read. It fails the test if ctx expires
// first — so a test asserts an echo or a replay by naming the marker it expects,
// not by guessing how the bytes are chunked.
func (tc *TerminalConn) ReadUntil(ctx context.Context, want string) string {
	tc.t.Helper()
	var buf []byte
	for {
		typ, data, err := tc.c.Read(ctx)
		if err != nil {
			tc.t.Fatalf("chartrtest: reading terminal until %q: %v\nso far: %q", want, err, buf)
		}
		if typ != websocket.MessageBinary {
			tc.t.Fatalf("chartrtest: terminal frame was %v, want binary", typ)
		}
		buf = append(buf, data...)
		if strings.Contains(string(buf), want) {
			return string(buf)
		}
	}
}

// Close closes the terminal socket — the test's stand-in for a browser detaching.
func (tc *TerminalConn) Close() {
	_ = tc.c.Close(websocket.StatusNormalClosure, "")
}

// ControlConn is a connected control socket in a test.
type ControlConn struct {
	c *websocket.Conn
	t testing.TB
}

// ReadSnapshot reads the next whole model snapshot. It fails the test if the
// frame is not valid JSON model.
func (cc *ControlConn) ReadSnapshot(ctx context.Context) model.Model {
	cc.t.Helper()
	typ, data, err := cc.c.Read(ctx)
	if err != nil {
		cc.t.Fatalf("chartrtest: read snapshot: %v", err)
	}
	if typ != websocket.MessageText {
		cc.t.Fatalf("chartrtest: snapshot was %v, want text", typ)
	}
	var m model.Model
	if err := json.Unmarshal(data, &m); err != nil {
		cc.t.Fatalf("chartrtest: snapshot not valid model JSON: %v (%q)", err, data)
	}
	return m
}

// WaitFor reads whole snapshots until pred is satisfied, returning the matching
// one. Because the control socket pushes a fresh snapshot on every change, a
// test asserting discovery-by-notice dials before the act it waits on, performs
// the act (dropping a map from outside — no refresh), then calls WaitFor: the
// push arrives on its own. It fails the test if ctx expires first.
func (cc *ControlConn) WaitFor(ctx context.Context, pred func(model.Model) bool) model.Model {
	cc.t.Helper()
	for {
		typ, data, err := cc.c.Read(ctx)
		if err != nil {
			cc.t.Fatalf("chartrtest: waiting for a matching snapshot: %v", err)
		}
		if typ != websocket.MessageText {
			cc.t.Fatalf("chartrtest: snapshot was %v, want text", typ)
		}
		var m model.Model
		if err := json.Unmarshal(data, &m); err != nil {
			cc.t.Fatalf("chartrtest: snapshot not valid model JSON: %v (%q)", err, data)
		}
		if pred(m) {
			return m
		}
	}
}

// Close closes the control socket.
func (cc *ControlConn) Close() {
	_ = cc.c.Close(websocket.StatusNormalClosure, "")
}
