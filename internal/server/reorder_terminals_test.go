package server_test

import (
	"testing"

	"github.com/rengwu/chartr/internal/chartrtest"
	"github.com/rengwu/chartr/internal/model"
)

// The session-tab reorder at the process boundary: POST
// /api/spaces/{id}/terminals/reorder takes the whole ordered list of a space's
// terminal ids and rearranges its tabs, scoped to that one space. Ad-hoc shells
// are the honest subject — they seat in creation order and carry no lifecycle, so
// the only thing moving them is this endpoint.

// termOrder is the space's terminal ids in pushed order — what the sidebar
// renders top to bottom.
func termOrder(s model.Space) []string {
	out := make([]string, 0, len(s.Terminals))
	for _, t := range s.Terminals {
		out = append(out, t.ID)
	}
	return out
}

func eqStrs(a, b []string) bool {
	if len(a) != len(b) {
		return false
	}
	for i := range a {
		if a[i] != b[i] {
			return false
		}
	}
	return true
}

// A reorder posts the whole new sequence and the pushed model shows the tabs in
// it — the same top-to-bottom order the operator dropped them into.
func TestReorderTerminalsRearrangesTheCardsTabs(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	resp := register(t, h, repo)

	// Three shells seat in creation order.
	a := h.OpenTerminal(resp.ID)
	b := h.OpenTerminal(resp.ID)
	c := h.OpenTerminal(resp.ID)

	s := findSpace(t, h.SnapshotUntil(ctx(t), func(m model.Model) bool {
		return len(findSpace(t, m, resp.ID).Terminals) == 3
	}), resp.ID)
	if got := termOrder(s); !eqStrs(got, []string{a, b, c}) {
		t.Fatalf("shells seated in %v, want creation order [a b c]", got)
	}

	// Move the last tab to the front.
	if code, body := h.Post("/api/spaces/"+resp.ID+"/terminals/reorder",
		map[string][]string{"ids": {c, a, b}}); code != 204 {
		t.Fatalf("reorder = %d, body %s", code, body)
	}

	s = findSpace(t, h.SnapshotUntil(ctx(t), func(m model.Model) bool {
		return eqStrs(termOrder(findSpace(t, m, resp.ID)), []string{c, a, b})
	}), resp.ID)
	if got := termOrder(s); !eqStrs(got, []string{c, a, b}) {
		t.Errorf("tabs after reorder = %v, want [c a b]", got)
	}
}

// A reorder is scoped to one space: it names only that space's tabs, and it never
// moves another space's. A list that reaches for a foreign tab is refused whole.
func TestReorderTerminalsIsScopedToItsSpace(t *testing.T) {
	h := chartrtest.Start(t)
	repoA := chartrtest.NewSpaceRepo(t)
	repoB := chartrtest.NewSpaceRepo(t)
	respA := register(t, h, repoA)
	respB := register(t, h, repoB)

	a1 := h.OpenTerminal(respA.ID)
	a2 := h.OpenTerminal(respA.ID)
	b1 := h.OpenTerminal(respB.ID)

	// A refused as soon as its posted list names B's tab — nothing changes.
	if code, _ := h.Post("/api/spaces/"+respA.ID+"/terminals/reorder",
		map[string][]string{"ids": {a1, b1}}); code != 400 {
		t.Errorf("reorder naming a foreign tab = %d, want 400", code)
	}
	// A partial list (omitting a2) is likewise refused.
	if code, _ := h.Post("/api/spaces/"+respA.ID+"/terminals/reorder",
		map[string][]string{"ids": {a1}}); code != 400 {
		t.Errorf("reorder omitting a tab = %d, want 400", code)
	}

	// A well-formed reorder of A leaves B's single tab exactly where it was.
	if code, body := h.Post("/api/spaces/"+respA.ID+"/terminals/reorder",
		map[string][]string{"ids": {a2, a1}}); code != 204 {
		t.Fatalf("reorder A = %d, body %s", code, body)
	}
	m := h.SnapshotUntil(ctx(t), func(m model.Model) bool {
		return eqStrs(termOrder(findSpace(t, m, respA.ID)), []string{a2, a1})
	})
	if got := termOrder(findSpace(t, m, respB.ID)); !eqStrs(got, []string{b1}) {
		t.Errorf("space B tabs after reordering A = %v, want [b1] (untouched)", got)
	}
}

// An unknown space id is a 404, exactly as every other space-scoped action is.
func TestReorderTerminalsUnknownSpace(t *testing.T) {
	h := chartrtest.Start(t)
	if code, _ := h.Post("/api/spaces/nope/terminals/reorder",
		map[string][]string{"ids": {}}); code != 404 {
		t.Errorf("reorder for an unknown space = %d, want 404", code)
	}
}
