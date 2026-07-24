package server_test

import (
	"fmt"
	"path/filepath"
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/chartrtest"
	"github.com/rengwu/chartr/internal/model"
)

// Ticket 03 at the process boundary: a registered space's maps enter the
// snapshot live, discovered by notice under either `.plan/` layout; derived
// statuses and the frontier are asserted against
// fixture tickets; a malformed map renders as-is with its malformation surfaced;
// finished maps sort last. Every assertion is on what the design makes public —
// the control-socket snapshot — never on internals.

const mapBody = "# Fixture Map\n\n## Destination\nA map to derive.\n\n" +
	"## Decisions so far\n\n## Not yet specified\n\n## Out of scope\n"

func ticket(num int, slug, blockedBy, typ, closing string) string {
	body := fmt.Sprintf("---\ntype: %s\nblocked_by: %s\n---\n\n# %s\n\n## Question\nQ.\n", typ, blockedBy, slug)
	if closing != "" {
		body += "\n" + closing + "\n"
	}
	return body
}

func findMap(t *testing.T, s model.Space, slug string) model.Map {
	t.Helper()
	for _, m := range s.Maps {
		if m.Slug == slug {
			return m
		}
	}
	t.Fatalf("map %q not in space %s (%d maps)", slug, s.Name, len(s.Maps))
	return model.Map{}
}

func findTicket(t *testing.T, m model.Map, num int) model.Ticket {
	t.Helper()
	for _, tk := range m.Tickets {
		if tk.Num == num {
			return tk
		}
	}
	t.Fatalf("ticket %02d not in map %s (%d tickets)", num, m.Slug, len(m.Tickets))
	return model.Ticket{}
}

func hasMap(s model.Space, slug string) bool {
	for _, m := range s.Maps {
		if m.Slug == slug {
			return true
		}
	}
	return false
}

// A map dropped into a registered space from outside — a hosted shell, an
// external terminal, a `git pull` — appears in the snapshot with no refresh
// action (story 11), under both the flat `.plan/<slug>/` layout and the
// nested `.plan/maps/<slug>/` one (story 12). The test dials the control
// socket before dropping anything and waits for the pushes to arrive on their
// own: discovery is by notice.
func TestMapAppearsByNoticeBothLayouts(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	resp := register(t, h, repo)

	// A snapshot right after registration: the space is present with no maps.
	if got := findSpace(t, h.Snapshot(ctx(t)), resp.ID).Maps; len(got) != 0 {
		t.Fatalf("space starts with %d maps, want 0", len(got))
	}

	cc := h.DialControl(ctx(t))
	defer cc.Close()
	cc.ReadSnapshot(ctx(t)) // drain the initial snapshot

	// Flat layout: .plan/<slug>/. Drop it from outside; wait for the notice.
	chartrtest.WriteMap(t, repo, "flat-map", mapBody)
	chartrtest.WriteTicket(t, repo, "flat-map", "01-first.md", ticket(1, "First", "[]", "task", ""))
	cc.WaitFor(ctx(t), func(m model.Model) bool {
		return hasMap(findSpace(t, m, resp.ID), "flat-map")
	})

	// Nested layout: .plan/maps/<slug>/. Neither layout is hard-coded — the same
	// discovery finds a map by its map.md wherever wayfinder wrote it.
	nested := filepath.Join("maps", "nested-map")
	chartrtest.WriteFile(t, repo, filepath.Join(".plan", nested, "map.md"), mapBody)
	chartrtest.WriteFile(t, repo, filepath.Join(".plan", nested, "tickets", "01-first.md"), ticket(1, "First", "[]", "task", ""))
	last := cc.WaitFor(ctx(t), func(m model.Model) bool {
		return hasMap(findSpace(t, m, resp.ID), "nested-map")
	})

	s := findSpace(t, last, resp.ID)
	if !hasMap(s, "flat-map") || !hasMap(s, "nested-map") {
		t.Errorf("want both layouts discovered; maps = %v", mapSlugs(s))
	}
}

// Derived statuses cross onto the wire — open, claimed, resolved, out_of_scope
// (ADR 0004, amended) — and the frontier is vanilla wayfinder's: a dependent is
// takeable exactly when every blocker is resolved. A leftover `## Proposed
// Answer` is an unknown heading that settles nothing: its ticket reads open, and
// a dependent of it stays held because its blocker is unresolved, not because
// anything is waiting on approval.
func TestDerivedStatusesAndFrontier(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "statuses", mapBody)
	w := func(filename, body string) { chartrtest.WriteTicket(t, repo, "statuses", filename, body) }
	w("01-blessed.md", ticket(1, "Blessed", "[]", "task", "## Answer\nApproved."))
	w("02-scoped.md", ticket(2, "Scoped", "[]", "task", "## Ruled out\nOut of scope."))
	w("03-on-blessed.md", ticket(3, "OnBlessed", "[01]", "task", ""))
	w("04-on-wreckage.md", ticket(4, "OnWreckage", "[05]", "task", ""))
	w("05-wreckage.md", ticket(5, "Wreckage", "[]", "task", "## Proposed Answer\nIn-flight wreckage from the retired gate."))
	w("06-claimed.md", "---\ntype: task\nblocked_by: []\nclaimed_by: session-a\nclaimed_at: 2026-07-19T09:00:00Z\n---\n\n# Claimed\n\n## Question\nQ.\n")
	w("07-frontier.md", ticket(7, "Frontier", "[]", "task", ""))

	resp := register(t, h, repo)
	m := findMap(t, findSpace(t, h.Snapshot(ctx(t)), resp.ID), "statuses")

	wantStatus := map[int]string{
		1: "resolved", 2: "out_of_scope", 3: "open",
		4: "open", 5: "open", 6: "claimed", 7: "open",
	}
	for num, want := range wantStatus {
		if got := findTicket(t, m, num).Status; got != want {
			t.Errorf("ticket %02d status = %q, want %q", num, got, want)
		}
	}

	// 03 (blocker resolved), 05 (unknown heading, no blockers) and 07 (no
	// blockers) are takeable; 04 is held because its blocker is unresolved;
	// nothing closed or claimed is on the frontier.
	wantFrontier := map[int]bool{1: false, 2: false, 3: true, 4: false, 5: true, 6: false, 7: true}
	for num, want := range wantFrontier {
		if got := findTicket(t, m, num).Frontier; got != want {
			t.Errorf("ticket %02d frontier = %v, want %v", num, got, want)
		}
	}
}

// A malformed map — a dangling blocked_by, an unparseable ticket — renders as-is
// with the malformation surfaced, never refused (story 17). The map is present
// in the snapshot, its well-formed tickets derive normally, and the defects are
// carried as surfaced strings.
func TestMalformedMapRendersWithMalformationSurfaced(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "broken", mapBody)
	chartrtest.WriteTicket(t, repo, "broken", "01-dangling.md", ticket(1, "Dangling", "[99]", "task", ""))
	// A file whose name is not NN-slug.md cannot be parsed as a ticket.
	chartrtest.WriteTicket(t, repo, "broken", "notaticket.md", "# not a ticket\n")

	resp := register(t, h, repo)
	m := findMap(t, findSpace(t, h.Snapshot(ctx(t)), resp.ID), "broken")

	// Not refused: the map and its parseable ticket are present.
	if len(m.Tickets) != 1 || m.Tickets[0].Num != 1 {
		t.Errorf("well-formed ticket missing; tickets = %+v", m.Tickets)
	}
	if !anyContains(m.Malformations, "does not exist") {
		t.Errorf("dangling blocked_by not surfaced; malformations = %v", m.Malformations)
	}
	if !anyContains(m.Malformations, "notaticket.md") {
		t.Errorf("unparseable ticket not surfaced; malformations = %v", m.Malformations)
	}
}

// The sidebar nests spaces → maps with finished maps sorting last. Finished-last
// beats slug order, so a finished map named to sort first still lands last.
func TestFinishedMapsSortLast(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	// "aaa-done" is finished (its one ticket is resolved) yet named to sort first
	// by slug; "zzz-active" has an open ticket.
	doneBody := "# Done\n\n## Destination\nD.\n\n## Decisions so far\n\n" +
		"- [First](./tickets/01-first.md) — done.\n\n## Out of scope\n"
	chartrtest.WriteMap(t, repo, "aaa-done", doneBody)
	chartrtest.WriteTicket(t, repo, "aaa-done", "01-first.md", ticket(1, "First", "[]", "task", "## Answer\nDone."))
	chartrtest.WriteMap(t, repo, "zzz-active", mapBody)
	chartrtest.WriteTicket(t, repo, "zzz-active", "01-open.md", ticket(1, "Open", "[]", "task", ""))

	resp := register(t, h, repo)
	s := findSpace(t, h.Snapshot(ctx(t)), resp.ID)

	if len(s.Maps) != 2 {
		t.Fatalf("want 2 maps, got %v", mapSlugs(s))
	}
	if s.Maps[0].Slug != "zzz-active" || s.Maps[1].Slug != "aaa-done" {
		t.Errorf("map order = %v, want [zzz-active aaa-done] — finished sorts last", mapSlugs(s))
	}
	if !findMap(t, s, "aaa-done").Finished {
		t.Error("aaa-done is all-resolved but not marked finished")
	}
	if findMap(t, s, "zzz-active").Finished {
		t.Error("zzz-active has an open ticket but is marked finished")
	}
}

// The detail pane (ticket 07) reads the full ticket — and the map's own material
// — from the snapshot with no second fetch, so each ticket carries its markdown
// body (Question, Done-when, any closing answer) and the map carries its body
// (Destination, Notes, Decisions, fog). Asserted at the seam: the body crosses
// onto the wire, below the H1 title, with the closing answer that lets a blocker
// show its answer inline.
func TestTicketAndMapBodiesInlined(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "bodies", mapBody)
	chartrtest.WriteTicket(t, repo, "bodies", "01-blessed.md",
		ticket(1, "Blessed", "[]", "task", "## Answer\nThe blocker's blessed answer."))

	resp := register(t, h, repo)
	m := findMap(t, findSpace(t, h.Snapshot(ctx(t)), resp.ID), "bodies")

	tk := findTicket(t, m, 1)
	if !strings.Contains(tk.Body, "## Question") || !strings.Contains(tk.Body, "Q.") {
		t.Errorf("ticket body missing Question material: %q", tk.Body)
	}
	if !strings.Contains(tk.Body, "The blocker's blessed answer.") {
		t.Errorf("ticket body missing its closing answer: %q", tk.Body)
	}
	if strings.Contains(tk.Body, "# Blessed") {
		t.Errorf("ticket body should start below the H1 title, got: %q", tk.Body)
	}
	if !strings.Contains(m.Body, "## Destination") || !strings.Contains(m.Body, "A map to derive.") {
		t.Errorf("map body missing its material: %q", m.Body)
	}
}

func mapSlugs(s model.Space) []string {
	out := make([]string, 0, len(s.Maps))
	for _, m := range s.Maps {
		out = append(out, m.Slug)
	}
	return out
}

func anyContains(list []string, sub string) bool {
	for _, x := range list {
		if strings.Contains(x, sub) {
			return true
		}
	}
	return false
}
