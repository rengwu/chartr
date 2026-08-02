package server_test

import (
	"net/http"
	"net/url"
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/chartrtest"
)

// websocket-origin-fix ticket 01 at the process boundary: both websockets are
// scoped to the address chartr is listening on, so a page the operator has open
// in another tab can neither read the model snapshot nor type into a terminal.
// Binding to loopback is not a boundary — websockets are not subject to CORS, so
// nothing but this check stands between an arbitrary origin and the operator's
// live shells. Every assertion is on the public handshake and on what reaches the
// terminal, never on server internals.

// The origin of a page that is not chartr. Deliberately a name that resolves
// nowhere: nothing dials it, it is only ever presented in a header, which is
// exactly what an attacking page's browser does.
const foreignOrigin = "http://attacker.example"

// The Vite dev server's default origin. It is admitted only in a `chartrdev`
// build; origin_gate_test.go and origin_gate_dev_test.go assert each half.
const viteDevOrigin = "http://localhost:5173"

// The whole model snapshot is behind the check: a foreign origin gets no socket
// and therefore no space ids, no absolute paths, and no live terminal ids to
// address the terminal socket with.
func TestControlSocketRefusesForeignOrigin(t *testing.T) {
	h := chartrtest.Start(t)

	conn, status := h.DialControlOrigin(ctx(t), foreignOrigin)
	if conn != nil {
		conn.Close()
		t.Fatalf("control socket accepted a handshake from %s", foreignOrigin)
	}
	if status != http.StatusForbidden {
		t.Errorf("cross-origin control handshake = %d, want %d", status, http.StatusForbidden)
	}
}

// The terminal socket is read *and* write access to a live shell, so the same
// refusal has to hold there.
func TestTerminalSocketRefusesForeignOrigin(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	resp := register(t, h, repo)
	termID := h.OpenTerminal(resp.ID)

	conn, status := h.DialTerminalOrigin(ctx(t), termID, foreignOrigin)
	if conn != nil {
		conn.Close()
		t.Fatalf("terminal socket accepted a handshake from %s", foreignOrigin)
	}
	if status != http.StatusForbidden {
		t.Errorf("cross-origin terminal handshake = %d, want %d", status, http.StatusForbidden)
	}
}

// The regression that matters: a refused terminal handshake must write *nothing*
// to the PTY. The probe pipelines a keystroke frame straight after the request
// without waiting for the response, so a server that accepted the socket and only
// then closed it would still have typed the bytes — and this test would catch it,
// where asserting on the status alone would not.
//
// The proof is ordering, and it is exact: the PTY consumes input in the order it
// arrives, so a legitimate command sent afterwards whose *output* has come back
// means anything the probe managed to inject has already been echoed and is in
// the scrollback this attach replayed.
func TestRefusedTerminalHandshakeWritesNothingToThePTY(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	resp := register(t, h, repo)
	termID := h.OpenTerminal(resp.ID)

	const injected = "chartr-injected-4a91"
	status := h.AttemptCrossOriginTerminalWrite(termID, foreignOrigin, "echo "+injected+"\n")

	// Attach for real and run a command of our own. Its output is the marker that
	// says the shell has worked through everything it was ever given. The terminal
	// is asserted on before the status is, deliberately: what the PTY received is
	// the claim, and the status is corroboration.
	tc := h.DialTerminal(ctx(t), termID)
	defer tc.Close()
	tc.Send(ctx(t), "echo settled-$((6*7))\n")
	out := tc.ReadUntil(ctx(t), "settled-42")

	if strings.Contains(out, injected) {
		t.Errorf("a cross-origin handshake reached the PTY: terminal carried %q\nfull output: %q", injected, out)
	}
	if status != http.StatusForbidden {
		t.Errorf("cross-origin terminal handshake = %d, want %d", status, http.StatusForbidden)
	}
}

// The other half of the contract: chartr's own origin is admitted and both
// sockets still do their job.
func TestBoundOriginStillStreams(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	resp := register(t, h, repo)

	cc, status := h.DialControlOrigin(ctx(t), h.BaseURL)
	if cc == nil {
		t.Fatalf("control handshake from the bound origin %s = %d, want it accepted", h.BaseURL, status)
	}
	defer cc.Close()
	if snap := cc.ReadSnapshot(ctx(t)); snap.Spaces == nil {
		t.Error("control socket from the bound origin sent no snapshot")
	}

	termID := h.OpenTerminal(resp.ID)
	tc, status := h.DialTerminalOrigin(ctx(t), termID, h.BaseURL)
	if tc == nil {
		t.Fatalf("terminal handshake from the bound origin %s = %d, want it accepted", h.BaseURL, status)
	}
	defer tc.Close()
	tc.Send(ctx(t), "echo streams-$((21*2))\n")
	if out := tc.ReadUntil(ctx(t), "streams-42"); !strings.Contains(out, "streams-42") {
		t.Fatalf("terminal from the bound origin never streamed its output; got %q", out)
	}
}

// The patterns, not just the library's Origin-equals-Host default: the rig binds
// 127.0.0.1, so a browser at http://localhost:PORT presents an Origin that does
// not equal the Host it sends. It is the same address spelled another way and
// must be admitted — a browser sets Origin itself, so localhost:PORT means the
// page came from whatever owns that port, which on this loopback stack is chartr.
func TestLoopbackAliasOfTheBoundAddressIsAdmitted(t *testing.T) {
	h := chartrtest.Start(t)

	cc, status := h.DialControlOrigin(ctx(t), "http://localhost:"+basePort(t, h))
	if cc == nil {
		t.Fatalf("control handshake from the bound address spelled localhost = %d, want it accepted", status)
	}
	defer cc.Close()
	if snap := cc.ReadSnapshot(ctx(t)); snap.Spaces == nil {
		t.Error("control socket from the localhost alias sent no snapshot")
	}
}

// basePort is the port chartr bound, for a test that needs to spell the bound
// address a different way than the rig's BaseURL does.
func basePort(t *testing.T, h *chartrtest.Chartr) string {
	t.Helper()
	u, err := url.Parse(h.BaseURL)
	if err != nil {
		t.Fatalf("parsing %q: %v", h.BaseURL, err)
	}
	return u.Port()
}
