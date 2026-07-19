package wayfinder

import "testing"

// The harness's one addition to the derived-status table (ADR 0004): a
// `## Proposed Answer` with prose derives `proposed`, a bare heading derives
// nothing, and a real `## Answer` always wins over a proposal it supersedes.
func TestParseProposedAnswer(t *testing.T) {
	proposed := "---\ntype: task\nblocked_by: []\nclaimed_by: session-a\nclaimed_at: 2026-07-10T09:00:00Z\n---\n\n" +
		"# Wire the socket\n\n## Question\nHow?\n\n## Proposed Answer\nWe pushed a whole snapshot.\n"
	tk, err := ParseTicket("p", "01-wire-the-socket.md", proposed)
	if err != nil {
		t.Fatal(err)
	}
	if tk.Status != StatusProposed {
		t.Errorf("status = %q, want proposed — a claim under a proposal reads proposed, not claimed", tk.Status)
	}

	// A bare `## Proposed Answer` proposes nothing: a session died mid-write.
	bare := "---\ntype: task\nblocked_by: []\n---\n\n# T\n\n## Question\nQ.\n\n## Proposed Answer\n"
	tk, err = ParseTicket("p", "02-bare.md", bare)
	if err != nil {
		t.Fatal(err)
	}
	if tk.Status != StatusOpen {
		t.Errorf("status = %q, want open — a bare Proposed Answer heading proposes nothing", tk.Status)
	}

	// A blessed `## Answer` wins even when a superseded `## Proposed Answer`
	// still sits above it.
	blessed := "---\ntype: task\nblocked_by: []\n---\n\n# T\n\n## Question\nQ.\n\n" +
		"## Proposed Answer\nDraft.\n\n## Answer\nApproved.\n"
	tk, err = ParseTicket("p", "03-blessed.md", blessed)
	if err != nil {
		t.Fatal(err)
	}
	if tk.Status != StatusResolved {
		t.Errorf("status = %q, want resolved — an Answer supersedes its Proposed Answer", tk.Status)
	}
}

// The stricter frontier is the containment: a blocker that is only `proposed`
// (committed but ungated) does not unblock its dependents — the ported Frontier
// requires a *resolved* blocker, and proposed is not resolved.
func TestProposedBlockerDoesNotUnblock(t *testing.T) {
	proposedBlocker := &Ticket{Num: 1, Path: "t.md", Title: "T1", Type: TypeTask, HasProposedAnswer: true, ProposedHeading: true}
	proposedBlocker.Derive()
	dependent := &Ticket{Num: 2, Path: "t.md", Title: "T2", Type: TypeTask, BlockedBy: []int{1}}
	dependent.Derive()

	e := &Effort{Dir: "d", Name: "e", Map: ParseMap("map.md", okMap), Tickets: []*Ticket{proposedBlocker, dependent}}
	if got := e.Frontier(); len(got) != 0 {
		t.Errorf("frontier = %v, want empty — a merely-proposed blocker must not seed its dependents", got)
	}

	// Bless it, and the dependent reaches the frontier.
	proposedBlocker.HasProposedAnswer = false
	proposedBlocker.HasAnswer, proposedBlocker.AnswerHeading = true, true
	proposedBlocker.Derive()
	got := e.Frontier()
	if len(got) != 1 || got[0].Num != 2 {
		t.Errorf("frontier = %v, want [02] once the blocker is blessed", got)
	}
}
