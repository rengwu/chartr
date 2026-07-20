package server_test

import (
	"strings"
	"testing"

	"github.com/rengwu/wayfinder-harness/internal/harnesstest"
	"github.com/rengwu/wayfinder-harness/internal/model"
)

// Ticket 05 at the process boundary: an ad-hoc shell opens in the space's
// working tree, echoes keystrokes, and survives detach/reattach with scrollback
// replayed; the terminal socket is driven directly (open over HTTP, write and
// read back over the binary socket, reattach and assert replay); a mapless space
// is fully usable this way. Ad-hoc shells are deliberately not sessions — no
// ticket, no lifecycle, ended by the human — so the assertions are on the public
// terminal socket and the pushed snapshot, never on any session apparatus (which
// does not yet exist). Each shell runs a real command whose *output* carries a
// computed marker, so a match proves keystrokes went up and bytes came down —
// not merely that the PTY echoed what was typed.

// A shell opens in the working tree, a typed command runs, and its output
// streams back down the socket. The marker is computed by the shell (mark-42),
// so matching it proves the command executed, not just that input echoed.
func TestAdHocShellRunsCommandAndStreamsOutput(t *testing.T) {
	h := harnesstest.Start(t)
	repo := harnesstest.NewSpaceRepo(t)
	resp := register(t, h, repo)

	termID := h.OpenTerminal(resp.ID)

	tc := h.DialTerminal(ctx(t), termID)
	defer tc.Close()

	tc.Send(ctx(t), "echo mark-$((6*7))\n")
	out := tc.ReadUntil(ctx(t), "mark-42")
	if !strings.Contains(out, "mark-42") {
		t.Fatalf("terminal output never carried the command result; got %q", out)
	}
}

// Detach (close the socket) then reattach: the server buffered the shell's
// output as scrollback and replays it on the fresh connection, so the operator
// walks back into the running shell rather than a blank pane.
func TestScrollbackReplayedOnReattach(t *testing.T) {
	h := harnesstest.Start(t)
	repo := harnesstest.NewSpaceRepo(t)
	resp := register(t, h, repo)

	termID := h.OpenTerminal(resp.ID)

	first := h.DialTerminal(ctx(t), termID)
	first.Send(ctx(t), "echo replay-$((21*2))\n")
	first.ReadUntil(ctx(t), "replay-42")
	first.Close() // detach

	// Reattach a fresh socket: the first frame(s) replay the buffered scrollback,
	// so the earlier output is present without re-running anything.
	second := h.DialTerminal(ctx(t), termID)
	defer second.Close()
	replayed := second.ReadUntil(ctx(t), "replay-42")
	if !strings.Contains(replayed, "replay-42") {
		t.Fatalf("reattach did not replay scrollback; got %q", replayed)
	}
}

// A mapless space is fully usable as a plain multiplexer: with no maps at all,
// opening a shell surfaces a terminal tab in the pushed snapshot, alive, and the
// space carries no maps. Ad-hoc terminals are not sessions, so they appear even
// where there is nothing to spawn against.
func TestMaplessSpaceOffersAdHocShell(t *testing.T) {
	h := harnesstest.Start(t)
	repo := harnesstest.NewSpaceRepo(t) // no .plan, no maps
	resp := register(t, h, repo)

	if s := findSpace(t, h.Snapshot(ctx(t)), resp.ID); len(s.Maps) != 0 {
		t.Fatalf("precondition: mapless space reports %d maps", len(s.Maps))
	}

	termID := h.OpenTerminal(resp.ID)

	s := findSpace(t, h.Snapshot(ctx(t)), resp.ID)
	term := findTerminal(t, s, termID)
	if !term.Alive {
		t.Errorf("freshly opened terminal is not alive")
	}
	if term.Title == "" {
		t.Errorf("terminal carries no tab title")
	}
}

// Ending a shell on the human's command (DELETE) drops its tab from the pushed
// model — by notice, not refresh — since ad-hoc shells are ended only by the
// operator and leave no lifecycle behind.
func TestClosingShellDropsTheTab(t *testing.T) {
	h := harnesstest.Start(t)
	repo := harnesstest.NewSpaceRepo(t)
	resp := register(t, h, repo)

	termID := h.OpenTerminal(resp.ID)
	if s := findSpace(t, h.Snapshot(ctx(t)), resp.ID); !hasTerminal(s, termID) {
		t.Fatalf("precondition: opened terminal absent from snapshot")
	}

	cc := h.DialControl(ctx(t))
	defer cc.Close()
	cc.ReadSnapshot(ctx(t)) // drain the current snapshot before ending the shell

	if code, body := h.Delete("/api/spaces/" + resp.ID + "/terminals/" + termID); code != 204 {
		t.Fatalf("close terminal = %d, body %s", code, body)
	}

	last := cc.WaitFor(ctx(t), func(m model.Model) bool {
		return !hasTerminal(findSpace(t, m, resp.ID), termID)
	})
	if hasTerminal(findSpace(t, last, resp.ID), termID) {
		t.Errorf("terminal tab lingered after the shell was ended")
	}
}

// Dialling the terminal socket for an id that names no live terminal is refused,
// not silently accepted — a stale tab or a bad deep link fails closed.
func TestTerminalSocketRejectsUnknownID(t *testing.T) {
	h := harnesstest.Start(t)
	if code, _ := h.Get("/ws/terminal/nope"); code != 404 {
		t.Errorf("terminal socket for an unknown id = %d, want 404", code)
	}
}

func findTerminal(t *testing.T, s model.Space, id string) model.Terminal {
	t.Helper()
	for _, term := range s.Terminals {
		if term.ID == id {
			return term
		}
	}
	t.Fatalf("terminal %s not in space %s (%d terminals)", id, s.Name, len(s.Terminals))
	return model.Terminal{}
}

func hasTerminal(s model.Space, id string) bool {
	for _, term := range s.Terminals {
		if term.ID == id {
			return true
		}
	}
	return false
}
