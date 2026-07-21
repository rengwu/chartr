package server_test

import (
	"context"
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/rengwu/wayfinder-harness/internal/harnesstest"
	"github.com/rengwu/wayfinder-harness/internal/model"
)

// Ticket 11 at the process boundary: propose and agent review. A stub implementer
// walks a ticket to `proposed` (committed `## Proposed Answer`); the state flows
// through the snapshot as the thing a review hangs on. A stub reviewer seats on the
// proposed ticket — its payload provably carrying the ticket's Done-when and the
// spec — and writes a clause-anchored verdict. From that verdict the harness
// assembles the review brief as plain markdown on disk: the proposed answer
// verbatim, a mechanically derived recommendation, clause-anchoring respected (an
// unanchored finding lands advisory), and the observed models named. Every
// assertion is on what the design makes public — the snapshot, the filesystem, git.

// reviewTicket is a ticket with a real `## Done-when` carrying two distinct
// clauses, so the review payload's Done-when guarantee and the brief's
// clause-anchoring are assertable against exact clause text.
func reviewTicket(num int) string {
	return "---\ntype: task\nblocked_by: []\n---\n\n" +
		"# Widget\n\n## Question\nBuild the widget.\n\n" +
		"## Done-when\n" +
		"- The widget renders in under 16ms.\n" +
		"- The widget passes the accessibility audit.\n"
}

// proposedBody is the implementer's proposed answer prose — a distinctive string
// the brief must carry verbatim.
const proposedBody = "The widget uses a virtualized list to stay under budget in the common case."

// walkToProposed spawns a stub implementer that proposes-commits-dies, then waits
// until the ticket derives `proposed` on the snapshot with the implementer's
// session pinned dead (so a review can then seat and the space has no live
// session). It returns the dead implementer session id.
func walkToProposed(t *testing.T, h *harnesstest.Harness, spaceID, slug string, num int) string {
	t.Helper()
	sp := mustSpawn(t, h, spaceID, slug, num, "implement")
	c, cancel := context.WithTimeout(context.Background(), 8*time.Second)
	defer cancel()
	h.SnapshotUntil(c, func(m model.Model) bool {
		s := findSpaceOK(m, spaceID)
		if s == nil {
			return false
		}
		tk := ticketOK(mapOK(*s, slug), num)
		tab := deadSessionTab(*s)
		return tk != nil && tk.Status == "proposed" && tab != nil
	})
	return sp.SessionID
}

// A stub implementer's committed `## Proposed Answer` flows through the snapshot as
// `proposed` — the state a review hangs on (and never onto the frontier, so its
// dependents stay blocked).
func TestProposedFlowsToSnapshot(t *testing.T) {
	h := harnesstest.Start(t)
	repo := harnesstest.NewSpaceRepo(t)

	harnesstest.WriteMap(t, repo, "widget", mapBody)
	harnesstest.WriteFile(t, repo, ".wayfinder-harness/config.toml", implConfig("widget"))
	ticketRel := filepath.Join(".plan", "widget", "tickets", "01-widget.md")
	harnesstest.WriteTicket(t, repo, "widget", "01-widget.md", reviewTicket(1))
	harnesstest.WriteTicket(t, repo, "widget", "02-dependent.md", ticket(2, "Dependent", "[1]", "task", ""))
	harnesstest.StubProposingAgent(t, "claude", ticketRel, proposedBody)

	resp := register(t, h, repo)
	walkToProposed(t, h, resp.ID, "widget", 1)

	s := findSpace(t, h.Snapshot(ctx(t)), resp.ID)
	tk := findTicket(t, findMap(t, s, "widget"), 1)
	if tk.Status != "proposed" {
		t.Fatalf("ticket status = %q, want proposed", tk.Status)
	}
	if tk.Frontier {
		t.Errorf("a proposed ticket must not be on the frontier")
	}
	// A merely-proposed blocker never unblocks its dependent (the stricter frontier).
	if dep := findTicket(t, findMap(t, s, "widget"), 2); dep.Frontier {
		t.Errorf("ticket 2 unblocked by a merely-proposed blocker — the gate leaked")
	}
}

// Review is refused on anything but a proposed ticket, and accepted on one: the
// gate widened from the fresh-spawn frontier to exactly `proposed` (ticket 09's
// flagged widening).
func TestReviewRunsOnlyOnProposed(t *testing.T) {
	h := harnesstest.Start(t)
	repo := harnesstest.NewSpaceRepo(t)

	harnesstest.WriteMap(t, repo, "widget", mapBody)
	harnesstest.WriteFile(t, repo, ".wayfinder-harness/config.toml", implConfig("widget"))
	harnesstest.WriteTicket(t, repo, "widget", "01-widget.md", reviewTicket(1))
	harnesstest.StubAgent(t, "codex")
	resp := register(t, h, repo)

	// Open ticket, not proposed: review refused.
	if code, body := h.Spawn(resp.ID, "widget", 1, "review"); code != 409 || !strings.Contains(body, "proposed") {
		t.Fatalf("review on an open ticket = %d (%s), want 409 naming proposed", code, body)
	}
}

// The whole review pipeline from a proposed ticket: the reviewer's payload carries
// the Done-when and the spec; its verdict assembles into a brief on disk with the
// proposed answer verbatim, a Send-back recommendation mechanically derived from an
// anchored blocking finding, an unanchored "blocking" finding demoted to advisory,
// and the observed models named. The brief on disk is byte-identical to what the
// action returns (what the GUI renders).
func TestReviewBriefAssembly(t *testing.T) {
	h := harnesstest.Start(t)
	repo := harnesstest.NewSpaceRepo(t)

	harnesstest.WriteMap(t, repo, "widget", mapBody)
	harnesstest.WriteFile(t, repo, ".wayfinder-harness/config.toml", implConfig("widget"))
	ticketRel := filepath.Join(".plan", "widget", "tickets", "01-widget.md")
	harnesstest.WriteTicket(t, repo, "widget", "01-widget.md", reviewTicket(1))
	// The implementer runs claude/sonnet (built-in implement binding); the reviewer
	// runs codex/gpt-5 (built-in review binding) — heterogeneous by config.
	harnesstest.StubProposingAgent(t, "claude", ticketRel, proposedBody)
	harnesstest.StubAgent(t, "codex")

	resp := register(t, h, repo)
	walkToProposed(t, h, resp.ID, "widget", 1)

	// Spawn the reviewer on the proposed ticket.
	rev := mustSpawn(t, h, resp.ID, "widget", 1, "review")

	// --- The review payload carries the Done-when and the spec, by assembly. ---
	payloadAbs := filepath.Join(repo, ".wayfinder-harness", "run", rev.SessionID, "payload.md")
	payload := harnesstest.WaitForFileContains(t, payloadAbs, "Done-when (the review contract)", 5*time.Second)
	for _, want := range []string{
		"The widget renders in under 16ms.", // a Done-when clause, carried
		"Done-when (the review contract)",   // the Done-when section label
		"Spec (",                            // the spec section, carried by assembly
	} {
		if !strings.Contains(payload, want) {
			t.Errorf("review payload missing %q:\n%s", want, payload)
		}
	}

	// --- The reviewer writes its verdict beside its payload (clause-anchored). ---
	verdict := "## Verdict\n\nfail\n\n" +
		"## Done-when\n\n" +
		"- met — \"The widget passes the accessibility audit.\"\n" +
		"- unmet — \"The widget renders in under 16ms.\"\n\n" +
		"## Findings\n\n" +
		"- blocking (Done-when: \"The widget renders in under 16ms.\") — with 500 rows the widget renders in 40ms, missing the 16ms budget.\n" +
		"- blocking — the empty-state path has no tests.\n" +
		"- advisory — variable naming is inconsistent.\n"
	verdictPath := filepath.Join(repo, ".wayfinder-harness", "run", rev.SessionID, "verdict.md")
	if err := os.WriteFile(verdictPath, []byte(verdict), 0o644); err != nil {
		t.Fatalf("writing verdict: %v", err)
	}

	// --- Assemble the brief. ---
	code, body := h.ReviewBrief(resp.ID, rev.SessionID)
	if code != 200 {
		t.Fatalf("review-brief = %d, body %s", code, body)
	}
	var br struct {
		Brief          string `json:"brief"`
		Recommendation string `json:"recommendation"`
	}
	if err := json.Unmarshal([]byte(body), &br); err != nil {
		t.Fatalf("review-brief response not JSON: %v (%q)", err, body)
	}

	// The mechanical recommendation matches the verdict — an anchored blocking
	// finding means send back.
	if br.Recommendation != "Send back" {
		t.Errorf("recommendation = %q, want Send back", br.Recommendation)
	}

	// The brief on disk is exactly what the action returns — plain markdown the GUI
	// merely renders.
	briefPath := filepath.Join(repo, ".wayfinder-harness", "run", rev.SessionID, "brief.md")
	onDisk, err := os.ReadFile(briefPath)
	if err != nil {
		t.Fatalf("brief not written to disk: %v", err)
	}
	if string(onDisk) != br.Brief {
		t.Errorf("brief on disk differs from the returned brief:\n--- disk ---\n%s\n--- returned ---\n%s", onDisk, br.Brief)
	}

	brief := string(onDisk)
	// Verbatim proposed answer.
	if !strings.Contains(brief, proposedBody) {
		t.Errorf("brief missing the verbatim proposed answer %q:\n%s", proposedBody, brief)
	}
	// Mechanical recommendation, rendered.
	if !strings.Contains(brief, "**Send back.**") {
		t.Errorf("brief missing the Send-back recommendation:\n%s", brief)
	}
	// The anchored blocking finding leads.
	if !strings.Contains(brief, "16ms budget") {
		t.Errorf("brief missing the anchored blocking finding:\n%s", brief)
	}
	// Clause-anchoring respected: the unanchored "blocking" finding is demoted to an
	// advisory, with a note saying why — it never gates.
	adv := afterHeading(brief, "## Advisories")
	if !strings.Contains(adv, "empty-state path") {
		t.Errorf("the unanchored finding did not land in Advisories:\n%s", brief)
	}
	if !strings.Contains(adv, "advisory by rule") {
		t.Errorf("the demoted finding carries no clause-anchoring note:\n%s", adv)
	}
	if strings.Contains(afterHeading(brief, "### Blocking finding"), "empty-state path") {
		t.Errorf("an unanchored finding leaked into the blocking finding:\n%s", brief)
	}
	// The reviewer's per-clause Done-when assessment is passed through verbatim.
	dw := afterHeading(brief, "## Done-when assessment")
	if !strings.Contains(dw, "unmet") || !strings.Contains(dw, "The widget renders in under 16ms.") {
		t.Errorf("brief missing the per-clause Done-when assessment:\n%s", dw)
	}
	// Observed models named — implement vs review, and the heterogeneity note.
	obs := afterHeading(brief, "## Observed models")
	for _, want := range []string{"sonnet", "gpt-5", "heterogeneous"} {
		if !strings.Contains(obs, want) {
			t.Errorf("observed-models line missing %q:\n%s", want, obs)
		}
	}
}

// A pass verdict — no finding cites a clause — assembles to an Approve
// recommendation, derived mechanically rather than from the agent's word.
func TestReviewBriefApproveOnPass(t *testing.T) {
	h := harnesstest.Start(t)
	repo := harnesstest.NewSpaceRepo(t)

	harnesstest.WriteMap(t, repo, "widget", mapBody)
	harnesstest.WriteFile(t, repo, ".wayfinder-harness/config.toml", implConfig("widget"))
	ticketRel := filepath.Join(".plan", "widget", "tickets", "01-widget.md")
	harnesstest.WriteTicket(t, repo, "widget", "01-widget.md", reviewTicket(1))
	harnesstest.StubProposingAgent(t, "claude", ticketRel, proposedBody)
	harnesstest.StubAgent(t, "codex")

	resp := register(t, h, repo)
	walkToProposed(t, h, resp.ID, "widget", 1)
	rev := mustSpawn(t, h, resp.ID, "widget", 1, "review")

	verdict := "## Verdict\n\npass\n\n" +
		"## Findings\n\n" +
		"- advisory — consider memoizing the layout.\n"
	verdictPath := filepath.Join(repo, ".wayfinder-harness", "run", rev.SessionID, "verdict.md")
	if err := os.WriteFile(verdictPath, []byte(verdict), 0o644); err != nil {
		t.Fatalf("writing verdict: %v", err)
	}

	code, body := h.ReviewBrief(resp.ID, rev.SessionID)
	if code != 200 {
		t.Fatalf("review-brief = %d, body %s", code, body)
	}
	var br struct {
		Brief          string `json:"brief"`
		Recommendation string `json:"recommendation"`
	}
	_ = json.Unmarshal([]byte(body), &br)
	if br.Recommendation != "Approve" {
		t.Errorf("recommendation on a clause-free verdict = %q, want Approve", br.Recommendation)
	}
	if !strings.Contains(br.Brief, "**Approve.**") {
		t.Errorf("brief missing the Approve recommendation:\n%s", br.Brief)
	}
}

// Assembling a brief before the reviewer has written a verdict is refused, not
// invented — the pipeline surfaces the absence rather than fabricate a pass.
func TestReviewBriefRefusedWithoutVerdict(t *testing.T) {
	h := harnesstest.Start(t)
	repo := harnesstest.NewSpaceRepo(t)

	harnesstest.WriteMap(t, repo, "widget", mapBody)
	harnesstest.WriteFile(t, repo, ".wayfinder-harness/config.toml", implConfig("widget"))
	ticketRel := filepath.Join(".plan", "widget", "tickets", "01-widget.md")
	harnesstest.WriteTicket(t, repo, "widget", "01-widget.md", reviewTicket(1))
	harnesstest.StubProposingAgent(t, "claude", ticketRel, proposedBody)
	harnesstest.StubAgent(t, "codex")

	resp := register(t, h, repo)
	walkToProposed(t, h, resp.ID, "widget", 1)
	rev := mustSpawn(t, h, resp.ID, "widget", 1, "review")

	if code, body := h.ReviewBrief(resp.ID, rev.SessionID); code != 409 || !strings.Contains(body, "verdict") {
		t.Fatalf("review-brief with no verdict = %d (%s), want 409 naming the missing verdict", code, body)
	}
}

// afterHeading returns the brief text under a heading, up to the next markdown
// heading — enough to assert a finding landed under the right section and nowhere
// else.
func afterHeading(s, heading string) string {
	i := strings.Index(s, heading)
	if i < 0 {
		return ""
	}
	rest := s[i+len(heading):]
	var out []string
	for _, l := range strings.Split(rest, "\n") {
		if strings.HasPrefix(strings.TrimSpace(l), "#") {
			break
		}
		out = append(out, l)
	}
	return strings.Join(out, "\n")
}

// findSpaceOK / mapOK / ticketOK / deadSessionTab are non-fatal lookups for use
// inside a SnapshotUntil predicate, where a fatal helper would kill the poll.
func findSpaceOK(m model.Model, id string) *model.Space {
	for i := range m.Spaces {
		if m.Spaces[i].ID == id {
			return &m.Spaces[i]
		}
	}
	return nil
}

func mapOK(s model.Space, slug string) model.Map {
	for _, m := range s.Maps {
		if m.Slug == slug {
			return m
		}
	}
	return model.Map{}
}

func ticketOK(m model.Map, num int) *model.Ticket {
	for i := range m.Tickets {
		if m.Tickets[i].Num == num {
			return &m.Tickets[i]
		}
	}
	return nil
}

func deadSessionTab(s model.Space) *model.Terminal {
	for i := range s.Terminals {
		if s.Terminals[i].Session != nil && !s.Terminals[i].Alive {
			return &s.Terminals[i]
		}
	}
	return nil
}
