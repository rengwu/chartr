//go:build !chartrdev

package server_test

import (
	"net/http"
	"testing"

	"github.com/rengwu/chartr/internal/chartrtest"
)

// The shipped half of the dev gate: with no `chartrdev` tag — which is every
// build anyone ever downloads, and the build `make test` and CI run — the Vite
// dev server's origin is a foreign origin like any other. This is what says the
// development convenience cannot ship enabled; its counterpart in
// origin_gate_dev_test.go says it still works when it is asked for.
func TestViteDevOriginRefusedWithoutTheDevTag(t *testing.T) {
	h := chartrtest.Start(t)

	conn, status := h.DialControlOrigin(ctx(t), viteDevOrigin)
	if conn != nil {
		conn.Close()
		t.Fatalf("a build without -tags chartrdev accepted the Vite dev origin %s", viteDevOrigin)
	}
	if status != http.StatusForbidden {
		t.Errorf("dev-origin control handshake = %d, want %d", status, http.StatusForbidden)
	}
}
