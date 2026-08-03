package server_test

import (
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/chartrtest"
)

// websocket-origin-fix ticket 04 at the process boundary: an operator who binds
// chartr somewhere other than loopback is told what that exposes, and one who
// does not is not told anything.
//
// This is the finding nothing else in the map touches. `-addr :9000` is
// documented in the README as an ordinary option, and there is no authentication
// anywhere in the server, so a wildcard bind hands shell and agent-spawn access
// to the whole network segment — no browser and no cross-origin trick involved,
// which is exactly why tickets 01 and 02 do not stand in its way. Nothing here
// refuses the bind; refusing is the trust-boundary map's decision to make.

// The wildcard bind, which is what `-addr :9000` resolves to.
func TestANonLoopbackBindWarnsAboutWhatItExposes(t *testing.T) {
	h := chartrtest.Start(t, chartrtest.WithBindAddress("0.0.0.0:0"))

	// Serve logs the warning before it serves, so an answered request proves the
	// line has been written if it is ever going to be.
	if code, _ := h.Get("/api/health"); code != 200 {
		t.Fatalf("sanity: /api/health on a wildcard-bound chartr = %d, want 200", code)
	}

	logged := h.Logged()
	if !strings.Contains(logged, "WARNING") {
		t.Fatalf("a wildcard bind logged no warning; log was:\n%s", logged)
	}
	// The warning has to name what is exposed, not merely that something is: an
	// operator who reads "not loopback" and nothing else has not been told that
	// the port is shell access to their machine.
	for _, want := range []string{"no authentication", "shells", "127.0.0.1"} {
		if !strings.Contains(logged, want) {
			t.Errorf("the warning does not mention %q; log was:\n%s", want, logged)
		}
	}
}

// The other half, and the half that decides whether the warning above is ever
// read: a loopback bind says nothing. Both spellings an operator or the product
// actually produces are here — the desktop shell's ephemeral `127.0.0.1:0`,
// which is every launch of the packaged app, and `-addr localhost:0`, where the
// flag names loopback rather than addressing it. A warning printed on either is
// one printed on every launch, and a warning printed on every launch is not read
// when it is true.
func TestALoopbackBindDoesNotWarn(t *testing.T) {
	for _, tc := range []struct {
		name string
		opts []chartrtest.Option
	}{
		// What cmd/webview binds: loopback, port chosen by the kernel, which is
		// also the rig's default.
		{name: "the desktop shell's ephemeral port", opts: nil},
		// The flag pointed at loopback by name rather than by address.
		{name: "-addr localhost:0", opts: []chartrtest.Option{chartrtest.WithBindAddress("localhost:0")}},
	} {
		t.Run(tc.name, func(t *testing.T) {
			h := chartrtest.Start(t, tc.opts...)

			if code, _ := h.Get("/api/health"); code != 200 {
				t.Fatalf("sanity: /api/health on a loopback-bound chartr = %d, want 200", code)
			}

			if logged := h.Logged(); strings.Contains(logged, "WARNING") {
				t.Errorf("a loopback bind warned; log was:\n%s", logged)
			}
		})
	}
}
