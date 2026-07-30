package server_test

import (
	"context"
	"runtime"
	"testing"
	"time"

	"github.com/rengwu/chartr/internal/chartrtest"
	"github.com/rengwu/chartr/internal/model"
)

// The dot on the session's card (session-notifications, ticket 04) at the process
// boundary: a real shell works for a real span, the run clock folds the states the
// tab actually published, and the flag it raises rides the pushed snapshot. The
// assertions are on what the operator gets — the model a fresh browser reads and
// the endpoint their focus posts — never on the clock's internals, which
// `internal/terminal` already table-tests.
//
// The dot is the quieter half of the same event the OS notification carries; both
// hang off the one RunFinished seam, and nothing here touches the notifier.

// dotCtx is the run tests' deadline. They wait on the sampler's cadence through a
// real run rather than on a function call, so the 5s the snapshot tests use is not
// enough; it is a ceiling on a hang, not a sleep.
func dotCtx(t *testing.T) context.Context {
	t.Helper()
	c, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	t.Cleanup(cancel)
	return c
}

// requireRunnableShell skips the tests that drive a run with `sleep`, which needs
// a POSIX shell — the same line the stub agents draw.
func requireRunnableShell(t *testing.T) {
	t.Helper()
	if runtime.GOOS == "windows" {
		t.Skip("the run is driven by a POSIX shell command; not supported on Windows")
	}
}

// unseenDot reads one tab's dot out of a snapshot. Non-fatal — it is called from
// inside snapshot predicates, where a missing tab means "not yet", not a failure.
func unseenDot(m model.Model, spaceID, termID string) bool {
	return terminalIn(m, spaceID, termID).FinishedUnseen
}

func terminalIn(m model.Model, spaceID, termID string) model.Terminal {
	for _, s := range m.Spaces {
		if s.ID != spaceID {
			continue
		}
		for _, term := range s.Terminals {
			if term.ID == termID {
				return term
			}
		}
	}
	return model.Terminal{}
}

func terminalStatus(m model.Model, spaceID, termID string) string {
	return terminalIn(m, spaceID, termID).Status
}

// A tab that works past the threshold and stops carries a dot, the dot is in the
// snapshot a freshly connected browser reads (which is what surviving a reload
// means — the flag is server state, and the run may well have ended with nothing
// attached), and focusing the tab clears it.
func TestFinishedRunRaisesTheDotAndFocusClearsIt(t *testing.T) {
	requireRunnableShell(t)
	h := chartrtest.Start(t)
	// A threshold and a settle small enough that one real command is a qualifying
	// run; the shipped 60s/10s would make this test a minute long for no more proof.
	chartrtest.WriteFile(t, h.ConfigDir, "notify.toml", "after = \"10ms\"\nsettle = \"10ms\"\n")
	resp := register(t, h, chartrtest.NewSpaceRepo(t))

	termID := h.OpenTerminal(resp.ID)
	tc := h.DialTerminal(dotCtx(t), termID)
	defer tc.Close()
	tc.Send(dotCtx(t), "sleep 2; echo done-$((6*7))\n")

	// The tab really was seen working — otherwise a dot that never appears would
	// prove nothing about the rule.
	h.SnapshotUntil(dotCtx(t), func(m model.Model) bool {
		return terminalStatus(m, resp.ID, termID) == model.TerminalWorking
	})
	tc.ReadUntil(dotCtx(t), "done-42")

	// Each poll dials its own control socket, so the snapshot this matches on is
	// the one a browser opened after the run would read.
	h.SnapshotUntil(dotCtx(t), func(m model.Model) bool { return unseenDot(m, resp.ID, termID) })

	// Focusing the tab is the whole acknowledgement — there is no dismiss.
	if code, body := h.Post("/api/spaces/"+resp.ID+"/terminals/"+termID+"/seen", nil); code != 204 {
		t.Fatalf("posting seen = %d, body %s", code, body)
	}
	// The action pushes the new model before its response returns, so this snapshot
	// already reflects it.
	if unseenDot(h.Snapshot(dotCtx(t)), resp.ID, termID) {
		t.Error("the dot survived the operator focusing the tab")
	}
}

// Short work stays silent: the single threshold gates the dot exactly as it gates
// the notification, so a run under it records nothing at all.
func TestRunUnderThresholdRaisesNoDot(t *testing.T) {
	requireRunnableShell(t)
	h := chartrtest.Start(t)
	chartrtest.WriteFile(t, h.ConfigDir, "notify.toml", "after = \"1m\"\nsettle = \"10ms\"\n")
	resp := register(t, h, chartrtest.NewSpaceRepo(t))

	termID := h.OpenTerminal(resp.ID)
	tc := h.DialTerminal(dotCtx(t), termID)
	defer tc.Close()
	tc.Send(dotCtx(t), "sleep 2; echo done-$((6*7))\n")

	h.SnapshotUntil(dotCtx(t), func(m model.Model) bool {
		return terminalStatus(m, resp.ID, termID) == model.TerminalWorking
	})
	tc.ReadUntil(dotCtx(t), "done-42")

	// The tab reading idle again is the sample the clock ended the run on — the
	// decision is already made by the time this snapshot shows it — so a dot absent
	// here is a dot that was never raised, not one that has yet to appear.
	m := h.SnapshotUntil(dotCtx(t), func(m model.Model) bool {
		return terminalStatus(m, resp.ID, termID) == model.TerminalIdle
	})
	if unseenDot(m, resp.ID, termID) {
		t.Error("a two-second run under a one-minute threshold raised a dot")
	}
}

// Focus posts without knowing whether there was anything to clear, so the endpoint
// has to be an ordinary no-op on a tab with no dot. An id that names no tab is a
// 404, exactly as ending one is.
func TestSeenOnATabWithNoDotIsANoOp(t *testing.T) {
	h := chartrtest.Start(t)
	resp := register(t, h, chartrtest.NewSpaceRepo(t))
	termID := h.OpenTerminal(resp.ID)

	if code, body := h.Post("/api/spaces/"+resp.ID+"/terminals/"+termID+"/seen", nil); code != 204 {
		t.Fatalf("posting seen for an unflagged tab = %d, body %s", code, body)
	}
	term := findTerminal(t, findSpace(t, h.Snapshot(ctx(t)), resp.ID), termID)
	if term.FinishedUnseen || !term.Alive {
		t.Errorf("the no-op changed the tab: %+v", term)
	}

	if code, _ := h.Post("/api/spaces/"+resp.ID+"/terminals/nope/seen", nil); code != 404 {
		t.Errorf("posting seen for an unknown tab = %d, want 404", code)
	}
}
