// Package harnesstest is the process-boundary test rig (spec, Testing
// Decisions). It starts the real harness against a temporary space and lets a
// test drive it exactly as an operator would — over HTTP and the control
// socket — asserting only on what the design makes public: snapshots, the files
// in .plan/, and git history. No test reaches into harness internals; the one
// seam is the process boundary, and this package is how tests reach it.
//
// The rig runs the server in-process on a real TCP port through the same
// server.New path cmd/harness uses, so the tested code path is the operator's.
// Later tickets extend this package (fixture maps, stub agents on PATH) rather
// than adding new seams.
package harnesstest

import (
	"bytes"
	"context"
	"encoding/json"
	"io"
	"net"
	"net/http"
	"strings"
	"testing"
	"time"

	"github.com/coder/websocket"

	"github.com/rengwu/wayfinder-harness/internal/model"
	"github.com/rengwu/wayfinder-harness/internal/server"
)

// Harness is a running harness backend under test.
type Harness struct {
	// BaseURL is the http origin the server is listening on, e.g.
	// http://127.0.0.1:54321.
	BaseURL string
	// DataDir is the harness-owned state root the server was given (a temp dir
	// unless overridden).
	DataDir string

	t testing.TB
}

// Option configures Start.
type Option func(*server.Options)

// WithDataDir overrides the harness state root (default: a fresh temp dir).
func WithDataDir(dir string) Option {
	return func(o *server.Options) { o.DataDir = dir }
}

// Start launches a harness on a random loopback port and registers cleanup that
// shuts it down when the test ends. It fails the test on any startup error.
func Start(t testing.TB, opts ...Option) *Harness {
	t.Helper()

	sopts := server.Options{DataDir: t.TempDir()}
	for _, opt := range opts {
		opt(&sopts)
	}

	srv, err := server.New(sopts)
	if err != nil {
		t.Fatalf("harnesstest: server.New: %v", err)
	}

	ln, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatalf("harnesstest: listen: %v", err)
	}

	ctx, cancel := context.WithCancel(context.Background())
	done := make(chan error, 1)
	go func() { done <- srv.Serve(ctx, ln) }()

	t.Cleanup(func() {
		cancel()
		select {
		case err := <-done:
			if err != nil {
				t.Errorf("harnesstest: server exited with error: %v", err)
			}
		case <-time.After(5 * time.Second):
			t.Error("harnesstest: server did not shut down within 5s")
		}
	})

	return &Harness{
		BaseURL: "http://" + ln.Addr().String(),
		DataDir: sopts.DataDir,
		t:       t,
	}
}

// Get performs a GET and returns the status code and body. It fails the test on
// a transport error.
func (h *Harness) Get(path string) (int, string) {
	h.t.Helper()
	resp, err := http.Get(h.BaseURL + path)
	if err != nil {
		h.t.Fatalf("harnesstest: GET %s: %v", path, err)
	}
	defer resp.Body.Close()
	body, _ := io.ReadAll(resp.Body)
	return resp.StatusCode, string(body)
}

// Post performs a JSON POST — an operator action (ADR 0010) — and returns the
// status code and body. A nil body sends no request body.
func (h *Harness) Post(path string, body any) (int, string) {
	h.t.Helper()
	var r io.Reader
	if body != nil {
		b, err := json.Marshal(body)
		if err != nil {
			h.t.Fatalf("harnesstest: marshal POST body: %v", err)
		}
		r = bytes.NewReader(b)
	}
	resp, err := http.Post(h.BaseURL+path, "application/json", r)
	if err != nil {
		h.t.Fatalf("harnesstest: POST %s: %v", path, err)
	}
	defer resp.Body.Close()
	out, _ := io.ReadAll(resp.Body)
	return resp.StatusCode, string(out)
}

// Delete performs a DELETE and returns the status code and body.
func (h *Harness) Delete(path string) (int, string) {
	h.t.Helper()
	req, err := http.NewRequest(http.MethodDelete, h.BaseURL+path, nil)
	if err != nil {
		h.t.Fatalf("harnesstest: build DELETE %s: %v", path, err)
	}
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		h.t.Fatalf("harnesstest: DELETE %s: %v", path, err)
	}
	defer resp.Body.Close()
	out, _ := io.ReadAll(resp.Body)
	return resp.StatusCode, string(out)
}

// OpenTerminal opens an ad-hoc shell in the space and returns its terminal id.
// It fails the test on a non-200 response.
func (h *Harness) OpenTerminal(spaceID string) string {
	h.t.Helper()
	code, body := h.Post("/api/spaces/"+spaceID+"/terminals", nil)
	if code != 200 {
		h.t.Fatalf("harnesstest: open terminal in %s = %d, body %s", spaceID, code, body)
	}
	var r struct {
		ID string `json:"id"`
	}
	if err := json.Unmarshal([]byte(body), &r); err != nil {
		h.t.Fatalf("harnesstest: open-terminal response not JSON: %v (%q)", err, body)
	}
	return r.ID
}

// Snapshot connects a control socket, reads exactly one whole snapshot, and
// closes it. Because operator actions push the new model before their HTTP
// response returns, a snapshot taken after an action already reflects it.
func (h *Harness) Snapshot(ctx context.Context) model.Model {
	h.t.Helper()
	conn := h.DialControl(ctx)
	defer conn.Close()
	return conn.ReadSnapshot(ctx)
}

// DialControl connects a control socket and returns it. The caller closes it.
func (h *Harness) DialControl(ctx context.Context) *ControlConn {
	h.t.Helper()
	wsURL := "ws" + strings.TrimPrefix(h.BaseURL, "http") + "/ws/control"
	c, _, err := websocket.Dial(ctx, wsURL, nil)
	if err != nil {
		h.t.Fatalf("harnesstest: dial control socket: %v", err)
	}
	return &ControlConn{c: c, t: h.t}
}

// DialTerminal connects the binary terminal socket for a terminal id and returns
// it. The caller closes it. It fails the test if the socket cannot be dialled
// (e.g. the terminal does not exist).
func (h *Harness) DialTerminal(ctx context.Context, termID string) *TerminalConn {
	h.t.Helper()
	wsURL := "ws" + strings.TrimPrefix(h.BaseURL, "http") + "/ws/terminal/" + termID
	c, _, err := websocket.Dial(ctx, wsURL, nil)
	if err != nil {
		h.t.Fatalf("harnesstest: dial terminal socket %s: %v", termID, err)
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
		tc.t.Fatalf("harnesstest: send keystrokes: %v", err)
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
			tc.t.Fatalf("harnesstest: reading terminal until %q: %v\nso far: %q", want, err, buf)
		}
		if typ != websocket.MessageBinary {
			tc.t.Fatalf("harnesstest: terminal frame was %v, want binary", typ)
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
		cc.t.Fatalf("harnesstest: read snapshot: %v", err)
	}
	if typ != websocket.MessageText {
		cc.t.Fatalf("harnesstest: snapshot was %v, want text", typ)
	}
	var m model.Model
	if err := json.Unmarshal(data, &m); err != nil {
		cc.t.Fatalf("harnesstest: snapshot not valid model JSON: %v (%q)", err, data)
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
			cc.t.Fatalf("harnesstest: waiting for a matching snapshot: %v", err)
		}
		if typ != websocket.MessageText {
			cc.t.Fatalf("harnesstest: snapshot was %v, want text", typ)
		}
		var m model.Model
		if err := json.Unmarshal(data, &m); err != nil {
			cc.t.Fatalf("harnesstest: snapshot not valid model JSON: %v (%q)", err, data)
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
