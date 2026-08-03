package server_test

import (
	"net/http"
	"testing"

	"github.com/rengwu/chartr/internal/chartrtest"
)

// websocket-origin-fix ticket 02 at the process boundary: no route answers to a
// name chartr is not listening on, so DNS rebinding does not hand an attacker's
// page the whole API.
//
// The attack these pin is not a cross-origin one. The attacker serves a page
// from a name whose DNS record they control, re-points it at 127.0.0.1 on a
// short TTL, and the browser then treats http://attacker.example:PORT as
// *same-origin* with chartr: it reads responses, it writes, and — because
// coder/websocket admits any handshake whose Origin equals its Host — ticket
// 01's origin patterns do not stand in the way. The name in the Host header is
// the only thing that gives it away, and every assertion below is on what a real
// browser puts on the wire.

// The Host a rebound browser sends: a name the operator's chartr has never heard
// of, carrying the port it is actually listening on. Nothing resolves it — it is
// only ever presented in a header, which is the whole of what rebinding does.
func rebindingHost(t *testing.T, h *chartrtest.Chartr) string {
	t.Helper()
	return "attacker.example:" + basePort(t, h)
}

// The API is read and write access to the operator's spaces, sessions and
// shells. Under a rebound name every route on it would answer.
func TestForeignHostRefusedOnAPIRoutes(t *testing.T) {
	h := chartrtest.Start(t)

	if code, _ := h.Get("/api/health"); code != http.StatusOK {
		t.Fatalf("sanity: /api/health on the bound address = %d, want 200", code)
	}

	code, _ := h.GetWithHost("/api/health", rebindingHost(t, h))
	if code != http.StatusForbidden {
		t.Errorf("/api/health asked for as %s = %d, want %d",
			rebindingHost(t, h), code, http.StatusForbidden)
	}
}

// The SPA route matters as much as the API: served under the attacker's name it
// is same-origin script running against a live cockpit, with every route below
// it open.
func TestForeignHostRefusedOnTheSPARoute(t *testing.T) {
	h := chartrtest.Start(t)

	code, _ := h.GetWithHost("/", rebindingHost(t, h))
	if code != http.StatusForbidden {
		t.Errorf("/ asked for as %s = %d, want %d", rebindingHost(t, h), code, http.StatusForbidden)
	}
}

// Both sockets, which is the clause that matters most: a rebinding attacker
// reaching /ws/terminal/{id} gets everything ticket 01 describes — keystrokes
// into the operator's live shell and their scrollback back out — no matter what
// the origin patterns say, because the browser's Origin *is* the Host it sends.
func TestForeignHostRefusedOnBothWebsocketRoutes(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	resp := register(t, h, repo)
	termID := h.OpenTerminal(resp.ID)

	host := rebindingHost(t, h)
	if status := h.HandshakeWithHost("/ws/control", host); status != http.StatusForbidden {
		t.Errorf("control handshake asked for as %s = %d, want %d", host, status, http.StatusForbidden)
	}
	if status := h.HandshakeWithHost("/ws/terminal/"+termID, host); status != http.StatusForbidden {
		t.Errorf("terminal handshake asked for as %s = %d, want %d", host, status, http.StatusForbidden)
	}
}

// The companion, and the reason the rule is an allowlist of three rather than
// one string comparison against the bound address: these are the spellings the
// operator's own browser and the desktop shell's ephemeral-port URL actually
// send, and all three have to keep working against a loopback bind.
func TestEveryLoopbackSpellingStillReachesEveryRoute(t *testing.T) {
	h := chartrtest.Start(t)
	port := basePort(t, h)

	for _, host := range []string{"127.0.0.1:" + port, "localhost:" + port, "[::1]:" + port} {
		if code, _ := h.GetWithHost("/api/health", host); code != http.StatusOK {
			t.Errorf("/api/health asked for as %s = %d, want 200", host, code)
		}
		if status := h.HandshakeWithHost("/ws/control", host); status != http.StatusSwitchingProtocols {
			t.Errorf("control handshake asked for as %s = %d, want %d",
				host, status, http.StatusSwitchingProtocols)
		}
	}
}

// The same thing end to end rather than at the handshake: the rig binds
// 127.0.0.1, and an operator who types `localhost` into their browser reaches it
// under a different name than the one it bound. The socket still has to stream.
func TestTheOperatorsBrowserAtLocalhostStillStreams(t *testing.T) {
	h := chartrtest.Start(t)
	authority := "localhost:" + basePort(t, h)

	cc, status := h.DialControlAt(ctx(t), authority)
	if cc == nil {
		t.Fatalf("control socket dialled at %s = %d, want it accepted", authority, status)
	}
	defer cc.Close()
	if snap := cc.ReadSnapshot(ctx(t)); snap.Spaces == nil {
		t.Error("control socket dialled at localhost sent no snapshot")
	}
}

// The port is half the address. A name we do answer to, carrying a port we are
// not listening on, is not this server — the request only arrived because
// something forwarded or forged it.
func TestTheRightNameOnTheWrongPortIsRefused(t *testing.T) {
	h := chartrtest.Start(t)

	code, _ := h.GetWithHost("/api/health", "localhost:1")
	if code != http.StatusForbidden {
		t.Errorf("/api/health asked for as localhost:1 = %d, want %d", code, http.StatusForbidden)
	}
}
