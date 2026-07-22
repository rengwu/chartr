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
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net"
	"net/http"
	"strings"
	"testing"
	"time"

	"github.com/coder/websocket"

	"github.com/rengwu/chartr/internal/model"
	"github.com/rengwu/chartr/internal/server"
)

// Chartr is a running chartr backend under test.
type Chartr struct {
	// BaseURL is the http origin the server is listening on, e.g.
	// http://127.0.0.1:54321.
	BaseURL string
	// DataDir is the chartr-owned state root the server was given (a temp dir
	// unless overridden).
	DataDir string
	// ConfigDir is the operator's local config root the server was given (a temp
	// dir unless overridden), whose `skills/` is the user layer of the skill
	// library. It is a temp dir by default so a run never reads — or is coloured
	// by — the developer's own library.
	ConfigDir string

	t testing.TB
}

// Option configures Start.
type Option func(*server.Options)

// WithDataDir overrides the chartr state root (default: a fresh temp dir).
func WithDataDir(dir string) Option {
	return func(o *server.Options) { o.DataDir = dir }
}

// WithConfigDir overrides the operator's config root (default: a fresh temp
// dir), whose `skills/` is the user layer of the skill library.
func WithConfigDir(dir string) Option {
	return func(o *server.Options) { o.ConfigDir = dir }
}

// WithQuietAfter sets the session silence threshold (ticket 10). Tests set it
// short so an AFK session's "quiet" hint is crossable within a test rather than
// after the calm production default. It is the real config knob, tuned down — not
// a test-only seam.
func WithQuietAfter(d time.Duration) Option {
	return func(o *server.Options) { o.QuietAfter = d }
}

// Start launches chartr on a random loopback port and registers cleanup that
// shuts it down when the test ends. It fails the test on any startup error.
func Start(t testing.TB, opts ...Option) *Chartr {
	t.Helper()

	sopts := server.Options{DataDir: t.TempDir(), ConfigDir: t.TempDir()}
	for _, opt := range opts {
		opt(&sopts)
	}

	srv, err := server.New(sopts)
	if err != nil {
		t.Fatalf("chartrtest: server.New: %v", err)
	}

	ln, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatalf("chartrtest: listen: %v", err)
	}

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

	return &Chartr{
		BaseURL:   "http://" + ln.Addr().String(),
		DataDir:   sopts.DataDir,
		ConfigDir: sopts.ConfigDir,
		t:         t,
	}
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

// Ideate opens the ideate on-ramp in the space and returns its tab id (ticket
// 15). It fails the test on a non-200 response.
func (h *Chartr) Ideate(spaceID string) string {
	h.t.Helper()
	code, body := h.Post("/api/spaces/"+spaceID+"/ideate", nil)
	if code != 200 {
		h.t.Fatalf("chartrtest: ideate in %s = %d, body %s", spaceID, code, body)
	}
	var r struct {
		ID string `json:"id"`
	}
	if err := json.Unmarshal([]byte(body), &r); err != nil {
		h.t.Fatalf("chartrtest: ideate response not JSON: %v (%q)", err, body)
	}
	return r.ID
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
