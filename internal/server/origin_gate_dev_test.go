//go:build chartrdev

package server_test

import (
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/chartrtest"
)

// The development half of the dev gate: built with `-tags chartrdev` — what
// `make dev-backend` does and nothing else does — the Vite dev server's origin is
// admitted on both sockets, so `make dev-web`'s proxy still reaches the backend
// with the cross-origin check on. Its counterpart in origin_gate_test.go asserts
// the same origin is refused in every build that ships.
//
// Run with: go test -tags chartrdev ./internal/server/
func TestViteDevOriginAdmittedUnderTheDevTag(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	resp := register(t, h, repo)

	cc, status := h.DialControlOrigin(ctx(t), viteDevOrigin)
	if cc == nil {
		t.Fatalf("a chartrdev build refused the Vite dev origin %s on the control socket = %d",
			viteDevOrigin, status)
	}
	defer cc.Close()
	if snap := cc.ReadSnapshot(ctx(t)); snap.Spaces == nil {
		t.Error("control socket from the dev origin sent no snapshot")
	}

	termID := h.OpenTerminal(resp.ID)
	tc, status := h.DialTerminalOrigin(ctx(t), termID, viteDevOrigin)
	if tc == nil {
		t.Fatalf("a chartrdev build refused the Vite dev origin %s on the terminal socket = %d",
			viteDevOrigin, status)
	}
	defer tc.Close()
	tc.Send(ctx(t), "echo proxied-$((21*2))\n")
	if out := tc.ReadUntil(ctx(t), "proxied-42"); !strings.Contains(out, "proxied-42") {
		t.Fatalf("terminal from the dev origin never streamed its output; got %q", out)
	}
}
