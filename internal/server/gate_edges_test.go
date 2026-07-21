package server_test

import (
	"encoding/json"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/rengwu/wayfinder-harness/internal/harnesstest"
)

// Ticket 17 at the process boundary: the gate's edges the agent review of ticket
// 12 anchored — advisory, none blocking on its own, but each makes the re-review
// loop or the abandon dialog read wrong. The browser-facing fix (the post-approve
// strip surviving approve, the Next button) has no process-boundary seam; it is
// driven by eye instead (Done-when).

// TestReReviewSupersedesStaleAndSurvivesRestart drives the exact sequence the
// review named: a rejecting verdict, a send-back fix-up that clears the finding,
// then a fresh review that passes. The gate must read the *new* verdict, not the
// one it superseded — and it must keep reading it correctly across a harness
// restart that drops every session's tab from memory, proving the read no longer
// depends on `s.terms` at all.
func TestReReviewSupersedesStaleAndSurvivesRestart(t *testing.T) {
	dataDir := t.TempDir()
	repo := harnesstest.NewSpaceRepo(t)

	harnesstest.WriteMap(t, repo, "widget", mapBody)
	harnesstest.WriteFile(t, repo, ".wayfinder-harness/config.toml", implConfig("widget"))
	ticketRel := filepath.Join(".plan", "widget", "tickets", "01-widget.md")
	harnesstest.WriteTicket(t, repo, "widget", "01-widget.md", reviewTicket(1))
	harnesstest.Git(t, repo, "add", "-A")
	harnesstest.Git(t, repo, "commit", "-q", "-m", "Fixture map")

	harnesstest.StubProposingAgent(t, "claude", ticketRel, proposedBody)
	harnesstest.StubDyingAgent(t, "codex")

	h1 := harnesstest.Start(t, harnesstest.WithDataDir(dataDir))
	resp := register(t, h1, repo)
	walkToProposed(t, h1, resp.ID, "widget", 1)

	// The first review: rejects.
	rev1 := mustSpawn(t, h1, resp.ID, "widget", 1, "review")
	harnesstest.WriteFile(t, repo, filepath.Join(".wayfinder-harness", "run", rev1.SessionID, "verdict.md"), rejectVerdict)
	if code, body := h1.ReviewBrief(resp.ID, rev1.SessionID); code != 200 {
		t.Fatalf("assembling the first brief = %d, body %s", code, body)
	}

	// Send back: a fix-up that rewrites the proposal in place and clears the
	// finding, then dies. Installed last, so it shadows the proposing stub — the
	// same pattern take-it-further's own test uses.
	harnesstest.StubRewritingAgent(t, "claude", ticketRel,
		"Amended: benches 500 rows too and stays under the 16ms budget.", "work.txt")
	if code, body := h1.FollowUp(resp.ID, "widget", 1, map[string]any{"role": "implement"}); code != 200 {
		t.Fatalf("send back = %d, body %s", code, body)
	}
	waitFor(t, 10*time.Second, func() bool {
		return strings.Contains(gitQuiet(repo, "log", "--format=%s"), "Follow-up: amend the proposal")
	}, "the fix-up's commit")

	// A fresh review, seated on the fixed-up proposal. The first (rejecting)
	// review's tab is still sitting in s.terms, dead and pinned, right alongside
	// it — the exact ordering the shadow bug depended on.
	rev2 := mustSpawn(t, h1, resp.ID, "widget", 1, "review")
	harnesstest.WriteFile(t, repo, filepath.Join(".wayfinder-harness", "run", rev2.SessionID, "verdict.md"), passVerdict)
	if code, body := h1.ReviewBrief(resp.ID, rev2.SessionID); code != 200 {
		t.Fatalf("assembling the second brief = %d, body %s", code, body)
	}

	// Restart: a brand-new process, s.terms empty, knowing nothing about either
	// review session. Only the repo on disk survives.
	h2 := harnesstest.Start(t, harnesstest.WithDataDir(dataDir))
	resp2 := register(t, h2, repo)
	if resp2.ID != resp.ID {
		t.Fatalf("re-registered id = %s, want the same stable id %s", resp2.ID, resp.ID)
	}

	code, body := h2.ReviewRead(resp2.ID, "widget", 1)
	if code != 200 {
		t.Fatalf("review read after restart = %d, body %s", code, body)
	}
	var rr struct {
		SessionID      string `json:"sessionId"`
		Recommendation string `json:"recommendation"`
	}
	if err := json.Unmarshal([]byte(body), &rr); err != nil {
		t.Fatalf("review read not JSON: %v (%q)", err, body)
	}
	if rr.SessionID != rev2.SessionID {
		t.Errorf("the gate read session %s, want the fresh review %s — the stale brief shadowed it", rr.SessionID, rev2.SessionID)
	}
	if rr.Recommendation != "Approve" {
		t.Errorf("recommendation = %q, want Approve — the fix-up's finding is cleared in the new verdict", rr.Recommendation)
	}

	// Approve needs no acknowledgement now: the gate is reading the passing
	// verdict, not the superseded rejection.
	if code, body := h2.Approve(resp2.ID, "widget", 1, false); code != 200 {
		t.Fatalf("approve on the new passing verdict = %d, body %s", code, body)
	}
}

// TestTicketDiffReadScopeAnchoredError: a scope=read diff whose `since` sha is
// stale or unknown must surface as the anchored error it is — never as an honest-
// looking empty diff (the review's finding: `patch, _ := git(...)` discarded the
// failure and the hub rendered "nothing changed in this scope").
func TestTicketDiffReadScopeAnchoredError(t *testing.T) {
	g := atTheGate(t, passVerdict, true)

	code, body := g.h.TicketDiff(g.spaceID, "widget", 1, "read", strings.Repeat("d", 40))
	if code == 200 {
		t.Fatalf("diff at an unknown since sha = 200 (body %s), want an anchored error", body)
	}
	var er struct {
		Error string `json:"error"`
	}
	if err := json.Unmarshal([]byte(body), &er); err != nil {
		t.Fatalf("error response not JSON: %v (%q)", err, body)
	}
	if er.Error == "" || !strings.Contains(strings.ToLower(er.Error), "resolve") {
		t.Errorf("the error does not explain the anchor problem: %q", er.Error)
	}
}

// TestAbandonRevertsTrailerIdentifiedWorkNotSubjectGuessed: an agent's own commit
// whose subject happens to collide with one of the harness's own lifecycle
// prefixes must still count as work. The old subject-prefix match silently
// excluded it; only the `Harness-Write` trailer may now.
func TestAbandonRevertsTrailerIdentifiedWorkNotSubjectGuessed(t *testing.T) {
	g := atTheGate(t, rejectVerdict, false)

	harnesstest.WriteFile(t, g.repo, "extra.txt", "more of the same attempt\n")
	harnesstest.Git(t, g.repo, "add", "--", "extra.txt")
	harnesstest.Git(t, g.repo, "commit", "-q", "-m", "Resolve the flaky retry loop in the widget's paint scheduler")
	collision := harnesstest.Git(t, g.repo, "rev-parse", "HEAD")

	// Abandon without a lever ticked: the point here is what the response reports
	// as work, not what a lever then does to it.
	code, body := g.h.Abandon(g.spaceID, "widget", 1, map[string]any{"reason": "needs another look"})
	if code != 200 {
		t.Fatalf("abandon = %d, body %s", code, body)
	}
	var dr struct {
		WorkCommits []string `json:"workCommits"`
	}
	if err := json.Unmarshal([]byte(body), &dr); err != nil {
		t.Fatalf("abandon response not JSON: %v (%q)", err, body)
	}
	found := false
	for _, c := range dr.WorkCommits {
		if c == collision {
			found = true
		}
	}
	if !found {
		t.Errorf("the agent's own commit (subject colliding with a harness prefix) was excluded from the work set: %v", dr.WorkCommits)
	}
}

// TestResetAvailableMatchesAbandonsOwnTipCheck: the review-read response's
// resetAvailable hint must track the same tip check abandon itself enforces, so
// the dialog never offers a lever the backend would then refuse. At the ordinary
// gate — a proposal a review has already claimed and read — the review's own
// claim commit sits on top of the work, so neither reads the tip as reachable;
// the pairing is what a browser pass would actually see.
func TestResetAvailableMatchesAbandonsOwnTipCheck(t *testing.T) {
	g := atTheGate(t, passVerdict, false)

	code, body := g.h.ReviewRead(g.spaceID, "widget", 1)
	if code != 200 {
		t.Fatalf("review read = %d, body %s", code, body)
	}
	var rr struct {
		ResetAvailable bool `json:"resetAvailable"`
	}
	if err := json.Unmarshal([]byte(body), &rr); err != nil {
		t.Fatalf("review read not JSON: %v (%q)", err, body)
	}
	if rr.ResetAvailable {
		t.Errorf("resetAvailable = true with the review's own claim on top of the work, want false")
	}

	// The backend itself refuses reset here — the hint matches the real gate.
	if code, body := g.h.Abandon(g.spaceID, "widget", 1, map[string]any{"reason": "x", "reset": true}); code != 409 {
		t.Errorf("abandon with reset off the tip = %d, want 409 (body %s)", code, body)
	}
}

// TestAbandonResetHardResetsToBeforeTheWork drives the reset lever end to end,
// on the one shape of the gate where it is reachable: a proposal abandoned before
// any review has claimed on top of it, so the implementer's own commits are still
// verifiably the tip. The work commits (and the claim beneath them) come out of
// history entirely, and the ticket is back on the frontier.
func TestAbandonResetHardResetsToBeforeTheWork(t *testing.T) {
	h := harnesstest.Start(t)
	repo := harnesstest.NewSpaceRepo(t)

	harnesstest.WriteMap(t, repo, "widget", mapBody)
	harnesstest.WriteFile(t, repo, ".wayfinder-harness/config.toml", implConfig("widget"))
	harnesstest.WriteTicket(t, repo, "widget", "01-widget.md", reviewTicket(1))
	harnesstest.Git(t, repo, "add", "-A")
	harnesstest.Git(t, repo, "commit", "-q", "-m", "Fixture map")
	harnesstest.StubProposingAgent(t, "claude", filepath.Join(".plan", "widget", "tickets", "01-widget.md"), proposedBody)

	resp := register(t, h, repo)
	walkToProposed(t, h, resp.ID, "widget", 1)

	before := harnesstest.Git(t, repo, "log", "--format=%s")
	if !strings.Contains(before, "Propose answer") {
		t.Fatalf("fixture: expected a propose commit in history: %s", before)
	}

	code, body := h.Abandon(resp.ID, "widget", 1, map[string]any{"reason": "starting fresh", "reset": true})
	if code != 200 {
		t.Fatalf("abandon with reset = %d, body %s", code, body)
	}
	var dr struct {
		Reset bool `json:"reset"`
	}
	if err := json.Unmarshal([]byte(body), &dr); err != nil {
		t.Fatalf("abandon response not JSON: %v (%q)", err, body)
	}
	if !dr.Reset {
		t.Errorf("reset did not run: %s", body)
	}

	after := harnesstest.Git(t, repo, "log", "--format=%s")
	if strings.Contains(after, "Propose answer") {
		t.Errorf("reset did not remove the work commits from history:\n%s", after)
	}
	// The claim commit itself is where reset lands — "before the work" means right
	// after the claim, not before it — so it stays an ancestor; what must be gone
	// is its *effect*, the stale claim on the ticket the abandon commit strips.
	onDisk := readFile(t, filepath.Join(repo, ".plan", "widget", "tickets", "01-widget.md"))
	if strings.Contains(onDisk, "claimed_by:") {
		t.Errorf("the stale claim survived reset:\n%s", onDisk)
	}

	m := findMap(t, findSpace(t, h.Snapshot(ctx(t)), resp.ID), "widget")
	if tk := findTicket(t, m, 1); tk.Status != "open" || !tk.Frontier {
		t.Errorf("ticket after reset = status %q frontier %v, want open and on the frontier", tk.Status, tk.Frontier)
	}
}

// TestAbandonTwiceGroupsUnderOneRejectedAttempts: a second abandonment must file
// its section under the same `## Rejected attempts` heading the first created,
// not wherever the second attempt's own proposal happened to land — the review's
// "the record can scatter" finding.
func TestAbandonTwiceGroupsUnderOneRejectedAttempts(t *testing.T) {
	g := atTheGate(t, passVerdict, false)

	if code, body := g.h.Abandon(g.spaceID, "widget", 1, map[string]any{"reason": "first pass missed the budget"}); code != 200 {
		t.Fatalf("first abandon = %d, body %s", code, body)
	}

	const secondProposal = "Second attempt: batches the renders instead of virtualizing."
	harnesstest.StubProposingAgent(t, "claude", g.ticketRel, secondProposal)
	walkToProposed(t, g.h, g.spaceID, "widget", 1)

	if code, body := g.h.Abandon(g.spaceID, "widget", 1, map[string]any{"reason": "second pass regressed accessibility"}); code != 200 {
		t.Fatalf("second abandon = %d, body %s", code, body)
	}

	onDisk := readFile(t, filepath.Join(g.repo, g.ticketRel))
	const parent = "## Rejected attempts"
	if n := strings.Count(onDisk, parent); n != 1 {
		t.Fatalf("ticket carries %d %q headings, want exactly 1 — both attempts must group under one:\n%s", n, parent, onDisk)
	}
	if n := strings.Count(onDisk, "### Rejected — "); n != 2 {
		t.Fatalf("ticket carries %d rejected-attempt sections, want 2:\n%s", n, onDisk)
	}
	// Nothing else may head a section between the one parent and end of file — a
	// third heading in between would mean the second section scattered away from
	// the first rather than grouping under it.
	parentIdx := strings.Index(onDisk, parent)
	rest := onDisk[parentIdx+len(parent):]
	for _, line := range strings.Split(rest, "\n") {
		lt := strings.TrimSpace(line)
		if strings.HasPrefix(lt, "## ") || (strings.HasPrefix(lt, "# ") && !strings.HasPrefix(lt, "## ")) {
			t.Errorf("a heading interrupts the rejected-attempts group: %q\n%s", lt, onDisk)
		}
	}
	if !strings.Contains(onDisk, proposedBody) {
		t.Errorf("the first rejected proposal is not kept verbatim:\n%s", onDisk)
	}
	if !strings.Contains(onDisk, secondProposal) {
		t.Errorf("the second rejected proposal is not kept verbatim:\n%s", onDisk)
	}
}
