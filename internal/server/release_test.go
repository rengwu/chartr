package server_test

import (
	"context"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/rengwu/chartr/internal/chartrtest"
	"github.com/rengwu/chartr/internal/model"
)

// Release keyed on the ticket: the way back for a claim whose session tab no
// longer exists. The halt's three actions all reach a claim through the dead tab
// holding it, and tabs live in memory — so a chartr restart, a dismissed tab, or a
// claim stamped by another chartr leaves a ticket stuck `claimed` with nothing
// able to clear it. These tests drive that state directly (a ticket file that
// already carries a claim, from a chartr that is not this one) rather than
// simulating a restart, because that is exactly what the orphan looks like on disk.

// claimedTicket is a ticket file already carrying a claim in its frontmatter —
// what a claim stamp leaves behind, and what outlives the chartr that wrote it.
func claimedTicket(num int, slug, sessionID, at string) string {
	return "---\ntype: task\nblocked_by: []\nclaimed_by: " + sessionID +
		"\nclaimed_at: " + at + "\n---\n\n# " + slug + "\n\n## Question\nQ.\n"
}

// The claim travels on the wire. Without claimed_by/claimed_at on the model there
// is nothing for the frontier to show *who* holds a stuck ticket or how long they
// have held it — the operator sees only a star that will not move.
func TestClaimedTicketCarriesItsClaimOnTheModel(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md",
		claimedTicket(1, "First", "sess-gone", "2026-07-20T09:00:00Z"))

	resp := register(t, h, repo)
	tk := findTicket(t, findMap(t, findSpace(t, h.Snapshot(ctx(t)), resp.ID), "widget"), 1)

	if tk.Status != "claimed" || tk.Frontier {
		t.Fatalf("ticket = {status:%s frontier:%v}, want {claimed false}", tk.Status, tk.Frontier)
	}
	if tk.ClaimedBy != "sess-gone" || tk.ClaimedAt != "2026-07-20T09:00:00Z" {
		t.Errorf("claim on the wire = {by:%q at:%q}, want {sess-gone 2026-07-20T09:00:00Z}",
			tk.ClaimedBy, tk.ClaimedAt)
	}
}

// The orphan is releasable: a claim held by a session this chartr has never seen
// clears back to the frontier, recorded as a line in the space's audit log — the
// same write the halt's release makes, reached by the ticket instead of the tab.
func TestReleaseTicketClearsAnOrphanedClaim(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md",
		claimedTicket(1, "First", "sess-gone", "2026-07-20T09:00:00Z"))

	resp := register(t, h, repo)
	if code, body := h.ReleaseTicket(resp.ID, "widget", 1); code != 200 {
		t.Fatalf("release = %d, body %s", code, body)
	}

	tk := findTicket(t, findMap(t, findSpace(t, h.Snapshot(ctx(t)), resp.ID), "widget"), 1)
	if tk.Status != "open" || !tk.Frontier {
		t.Errorf("ticket after release = {status:%s frontier:%v}, want {open true}", tk.Status, tk.Frontier)
	}
	if tk.ClaimedBy != "" || tk.ClaimedAt != "" {
		t.Errorf("release left the claim on the model: {by:%q at:%q}", tk.ClaimedBy, tk.ClaimedAt)
	}

	if body := ticketFileBody(t, repo, "widget", "01-first.md"); strings.Contains(body, "claimed_by") {
		t.Errorf("release left claimed_by on the ticket:\n%s", body)
	}
	// A single release record naming the released session, read off the ticket's
	// own frontmatter — by construction there is no tab left to ask. The orphan's
	// original claim was stamped by another chartr, so it left no record here.
	rel := filepath.Join(chartrtest.MapDir("widget"), "tickets", "01-first.md")
	recs := auditRecords(t, repo)
	if len(recs) != 1 || recs[0].Event != "release" || recs[0].Ticket != rel {
		t.Fatalf("audit records after release = %+v, want one release of %s", recs, rel)
	}
	if recs[0].Session != "sess-gone" {
		t.Errorf("release record names %q, want the released session sess-gone", recs[0].Session)
	}
}

// A live session holding the ticket is not a stale claim, whatever the frontmatter
// says: releasing it would strip the claim out from under an agent still editing
// the tree. Refused, and the claim survives untouched.
func TestReleaseTicketRefusedWhileSessionLive(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	chartrtest.StubAgent(t, "claude")

	resp := register(t, h, repo)
	mustSpawn(t, h, resp.ID, "widget", 1, "implement")

	code, body := h.ReleaseTicket(resp.ID, "widget", 1)
	if code != 409 {
		t.Fatalf("release of a live-held ticket = %d, body %s", code, body)
	}
	if !strings.Contains(body, "live_session_exists") {
		t.Errorf("refusal does not carry the live-session code: %s", body)
	}
	if fileBody := ticketFileBody(t, repo, "widget", "01-first.md"); !strings.Contains(fileBody, "claimed_by") {
		t.Errorf("refused release still cleared the claim:\n%s", fileBody)
	}
}

// Only a claim can be released. An open ticket has nothing to clear, and saying so
// is what keeps this from becoming a general edit-the-frontmatter endpoint.
func TestReleaseTicketRefusedWhenNotClaimed(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))

	resp := register(t, h, repo)
	code, body := h.ReleaseTicket(resp.ID, "widget", 1)
	if code != 409 {
		t.Fatalf("release of an unclaimed ticket = %d, body %s", code, body)
	}
	if n := auditCount(t, repo); n != 0 {
		t.Errorf("refused release still recorded: %d audit records", n)
	}
}

// A dead tab pinned to the ticket outlives its claim once the ticket is released
// by number: its resume and respawn would re-enter a ticket nobody holds. The
// ticket-level release drops it, exactly as the halt's own release does.
func TestReleaseTicketDropsThePinnedDeadTab(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	chartrtest.StubDyingAgent(t, "claude")

	resp := register(t, h, repo)
	spawnThenDie(t, h, resp.ID, "widget", 1, "implement")

	if code, body := h.ReleaseTicket(resp.ID, "widget", 1); code != 200 {
		t.Fatalf("release = %d, body %s", code, body)
	}

	c, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	m := h.SnapshotUntil(c, func(m model.Model) bool {
		return sessionTab(findSpace(t, m, resp.ID)) == nil
	})
	s := findSpace(t, m, resp.ID)
	if tab := sessionTab(s); tab != nil {
		t.Errorf("release left the dead tab pinned: %+v", tab.Session)
	}
	if tk := findTicket(t, findMap(t, s, "widget"), 1); tk.Status != "open" || !tk.Frontier {
		t.Errorf("ticket after release = {status:%s frontier:%v}, want {open true}", tk.Status, tk.Frontier)
	}
}
