package server_test

import (
	"context"
	"encoding/json"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/rengwu/wayfinder-harness/internal/harnesstest"
	"github.com/rengwu/wayfinder-harness/internal/model"
)

// Ticket 12 at the process boundary: the human review hub — the gate, whole. Each
// test walks a real space to a ticket sitting at the gate (proposed, reviewed,
// brief on disk) and then drives exactly one exit over HTTP, asserting only on
// what the design makes public: the snapshot, the files in `.plan/`, the payload
// and its archive, and git history.
//
// The four exits: approve (the promotion commit, its shape, its trailers, its
// unblocking, its behaviour beside a live session and under a concurrent writer),
// the acknowledgement tick that gates approving over a rejection, send back (the
// note riding the payload and nowhere else), take it further (the proposal
// rewritten in place, commits stacking), and abandon (the demotion returning the
// ticket to the frontier with the levers untouched).

const (
	// The verdict a passing review writes: no finding cites a Done-when clause, so
	// nothing blocks and the mechanical recommendation is Approve.
	passVerdict = "## Verdict\n\npass\n\n" +
		"## Findings\n\n" +
		"- advisory — consider memoizing the layout.\n" +
		"- advisory — the variable naming is inconsistent.\n"

	// The verdict a rejecting review writes: one finding anchored to a Done-when
	// clause, which is the only thing that can block.
	rejectVerdict = "## Verdict\n\nfail\n\n" +
		"## Findings\n\n" +
		"- blocking (Done-when: \"The widget renders in under 16ms.\") — with 500 rows the widget renders in 40ms, missing the 16ms budget.\n" +
		"- advisory — the empty-state path has no tests.\n"

	blockingText = "missing the 16ms budget"
)

// gateRig is a space walked to the human gate: ticket 01 proposed by a stub
// implementer and reviewed by a stub reviewer whose verdict is written and whose
// brief is assembled; ticket 02 blocked by it, so approval's unblocking is
// assertable. liveReviewer keeps the reviewer's process running, which is how the
// "approval never waits on a live session" case is driven.
type gateRig struct {
	h         *harnesstest.Harness
	repo      string
	spaceID   string
	reviewSID string
	ticketRel string
}

func atTheGate(t *testing.T, verdict string, liveReviewer bool) gateRig {
	t.Helper()
	h := harnesstest.Start(t)
	repo := harnesstest.NewSpaceRepo(t)

	harnesstest.WriteMap(t, repo, "widget", mapBody)
	harnesstest.WriteFile(t, repo, ".wayfinder-harness/config.toml", implConfig("widget"))
	ticketRel := filepath.Join(".plan", "widget", "tickets", "01-widget.md")
	harnesstest.WriteTicket(t, repo, "widget", "01-widget.md", reviewTicket(1))
	harnesstest.WriteTicket(t, repo, "widget", "02-dependent.md", ticket(2, "Dependent", "[1]", "task", ""))
	// A committed baseline, so the claim is not the repository's first commit and
	// history reads the way an operator's would.
	harnesstest.Git(t, repo, "add", "-A")
	harnesstest.Git(t, repo, "commit", "-q", "-m", "Fixture map")

	harnesstest.StubProposingAgent(t, "claude", ticketRel, proposedBody)
	if liveReviewer {
		harnesstest.StubAgent(t, "codex")
	} else {
		harnesstest.StubDyingAgent(t, "codex")
	}

	resp := register(t, h, repo)
	walkToProposed(t, h, resp.ID, "widget", 1)
	rev := mustSpawn(t, h, resp.ID, "widget", 1, "review")

	if err := os.WriteFile(filepath.Join(repo, ".wayfinder-harness", "run", rev.SessionID, "verdict.md"),
		[]byte(verdict), 0o644); err != nil {
		t.Fatalf("writing verdict: %v", err)
	}
	if code, body := h.ReviewBrief(resp.ID, rev.SessionID); code != 200 {
		t.Fatalf("assembling the brief = %d, body %s", code, body)
	}
	if !liveReviewer {
		// Wait for the reviewer's death to land, so a follow-up is not refused for a
		// live session that is merely on its way out.
		c, cancel := context.WithTimeout(context.Background(), 8*time.Second)
		defer cancel()
		h.SnapshotUntil(c, func(m model.Model) bool {
			s := findSpaceOK(m, resp.ID)
			if s == nil {
				return false
			}
			for _, term := range s.Terminals {
				if term.Session != nil && term.Session.Role == "review" && !term.Alive {
					return true
				}
			}
			return false
		})
	}
	return gateRig{h: h, repo: repo, spaceID: resp.ID, reviewSID: rev.SessionID, ticketRel: ticketRel}
}

// approveResp is the approve action's own result — the promotion commit, what it
// unblocked, the next suggestion, and the smear report when another writer got
// there first.
type approveResp struct {
	Commit                string `json:"commit"`
	Unblocked             []int  `json:"unblocked"`
	ApprovedOverRejection bool   `json:"approvedOverRejection"`
	SmearedInto           string `json:"smearedInto"`
	Warning               string `json:"warning"`
	Next                  *struct {
		Num   int    `json:"num"`
		Title string `json:"title"`
	} `json:"next"`
}

// The gate's one-click exit, whole: the promotion is its own pathspec-limited
// commit carrying the gate trailers, it lands *while the reviewer is still live*
// without touching that session's staged work, and it unblocks the dependent onto
// the stricter frontier — which the post-approve strip then offers as the next
// ticket.
func TestApprovePromotesUnblocksAndCommitsNarrowly(t *testing.T) {
	g := atTheGate(t, passVerdict, true)

	// A live session's staged work, sitting in the shared index. The promotion must
	// not sweep it in (ADR 0008) — that is what the pathspec limit buys.
	harnesstest.WriteFile(t, g.repo, "debris.txt", "a live session's staged work\n")
	harnesstest.Git(t, g.repo, "add", "--", "debris.txt")

	// The reviewer is live throughout: approval never waits on a session.
	before := findSpace(t, g.h.Snapshot(ctx(t)), g.spaceID)
	if !hasLiveSession(before) {
		t.Fatalf("fixture: expected a live review session during approval")
	}
	headBefore := harnesstest.Git(t, g.repo, "rev-parse", "HEAD")

	code, body := g.h.Approve(g.spaceID, "widget", 1, false)
	if code != 200 {
		t.Fatalf("approve = %d, body %s", code, body)
	}
	var ar approveResp
	if err := json.Unmarshal([]byte(body), &ar); err != nil {
		t.Fatalf("approve response not JSON: %v (%q)", err, body)
	}

	// --- Commit shape: its own commit on top of HEAD, never an amend. ---
	if ar.Commit == "" || ar.Commit == headBefore {
		t.Fatalf("approve produced no new commit (commit=%q, head was %q)", ar.Commit, headBefore)
	}
	if parent := harnesstest.Git(t, g.repo, "rev-parse", ar.Commit+"^"); parent != headBefore {
		t.Errorf("promotion parent = %s, want the previous HEAD %s — promotion must never rewrite history", parent, headBefore)
	}
	if !strings.Contains(harnesstest.Git(t, g.repo, "cat-file", "-t", headBefore), "commit") {
		t.Errorf("the pre-promotion commit is gone from history — the promotion amended rather than appended")
	}

	// --- Pathspec-limited: the ticket file and nothing else. ---
	touched := strings.Fields(harnesstest.Git(t, g.repo, "show", "--name-only", "--format=", ar.Commit))
	if len(touched) != 1 || touched[0] != filepath.ToSlash(g.ticketRel) {
		t.Errorf("promotion touched %v, want only %s", touched, g.ticketRel)
	}
	if status := harnesstest.Git(t, g.repo, "status", "--porcelain", "--", "debris.txt"); !strings.HasPrefix(status, "A ") {
		t.Errorf("the live session's staged file is %q, want it still staged and uncommitted", status)
	}

	// --- Trailers: git alone answers who blessed this, and on what verdict. ---
	msg := harnesstest.Git(t, g.repo, "log", "-1", "--format=%B", ar.Commit)
	for _, want := range []string{"Ticket: 01", "Role: gate", "Gate: human", "Verdict: Approve"} {
		if !strings.Contains(msg, want) {
			t.Errorf("promotion commit missing trailer %q:\n%s", want, msg)
		}
	}
	if strings.Contains(msg, "Approved-Over-Rejection") {
		t.Errorf("a passing verdict recorded an override:\n%s", msg)
	}

	// --- The promotion is what unblocks the dependent. ---
	s := findSpace(t, g.h.Snapshot(ctx(t)), g.spaceID)
	m := findMap(t, s, "widget")
	if tk := findTicket(t, m, 1); tk.Status != "resolved" {
		t.Errorf("approved ticket status = %q, want resolved", tk.Status)
	}
	if dep := findTicket(t, m, 2); !dep.Frontier {
		t.Errorf("the dependent did not reach the frontier after approval")
	}
	if len(ar.Unblocked) != 1 || ar.Unblocked[0] != 2 {
		t.Errorf("approve reported unblocked = %v, want [2]", ar.Unblocked)
	}
	if ar.Next == nil || ar.Next.Num != 2 {
		t.Errorf("post-approve suggestion = %+v, want ticket 2", ar.Next)
	}
	if !hasLiveSession(findSpace(t, g.h.Snapshot(ctx(t)), g.spaceID)) {
		t.Errorf("the live review session did not survive the approval")
	}
}

// A rejecting verdict costs exactly one tick — no more, and never none. Without
// it the approval is refused and nothing is written; with it the promotion lands
// and records the override.
func TestApproveOverRejectionNeedsTheTick(t *testing.T) {
	g := atTheGate(t, rejectVerdict, true)
	headBefore := harnesstest.Git(t, g.repo, "rev-parse", "HEAD")

	code, body := g.h.Approve(g.spaceID, "widget", 1, false)
	if code != 409 {
		t.Fatalf("approve over a rejection without the tick = %d, want 409 (body %s)", code, body)
	}
	if !strings.Contains(body, "blocking finding") {
		t.Errorf("the refusal does not name the blocking finding: %s", body)
	}
	if head := harnesstest.Git(t, g.repo, "rev-parse", "HEAD"); head != headBefore {
		t.Errorf("a refused approval still committed something (%s → %s)", headBefore, head)
	}
	if tk := findTicket(t, findMap(t, findSpace(t, g.h.Snapshot(ctx(t)), g.spaceID), "widget"), 1); tk.Status != "proposed" {
		t.Errorf("ticket status after a refused approval = %q, want proposed", tk.Status)
	}

	// The tick, and only the tick, opens the gate.
	code, body = g.h.Approve(g.spaceID, "widget", 1, true)
	if code != 200 {
		t.Fatalf("approve with the acknowledgement = %d, body %s", code, body)
	}
	var ar approveResp
	_ = json.Unmarshal([]byte(body), &ar)
	if !ar.ApprovedOverRejection {
		t.Errorf("approving over a rejection was not reported as such: %s", body)
	}
	msg := harnesstest.Git(t, g.repo, "log", "-1", "--format=%B", ar.Commit)
	for _, want := range []string{"Approved-Over-Rejection: true", "Acknowledged-Blocking:", "Verdict: Send back"} {
		if !strings.Contains(msg, want) {
			t.Errorf("the override is not recorded in the commit (%q missing):\n%s", want, msg)
		}
	}
}

// ADR 0008's residual race, driven for real: a concurrent writer's `commit -a`
// sweeps the promotion edit in before the harness commits it. The answer is
// promoted all the same — only the attribution is theirs — and the harness
// detects its own empty commit and reports it rather than failing or retrying.
func TestApproveDetectsTheAttributionSmear(t *testing.T) {
	g := atTheGate(t, passVerdict, true)
	harnesstest.InstallSweepHook(t, g.repo, "Agent sweep")

	code, body := g.h.Approve(g.spaceID, "widget", 1, false)
	if code != 200 {
		t.Fatalf("approve under a concurrent writer = %d, body %s", code, body)
	}
	var ar approveResp
	_ = json.Unmarshal([]byte(body), &ar)
	if ar.SmearedInto == "" {
		t.Fatalf("the attribution smear went undetected: %s", body)
	}
	if !strings.Contains(ar.Warning, "attribution") {
		t.Errorf("the smear was detected but not reported to the operator: %q", ar.Warning)
	}
	if subject := harnesstest.Git(t, g.repo, "log", "-1", "--format=%s"); subject != "Agent sweep" {
		t.Errorf("HEAD subject = %q, want the concurrent writer's commit", subject)
	}
	if carrier := harnesstest.Git(t, g.repo, "rev-parse", "HEAD"); carrier != ar.SmearedInto {
		t.Errorf("smearedInto = %s, want the commit that actually carries the edit (%s)", ar.SmearedInto, carrier)
	}
	// The promotion itself still happened: the ticket resolves and the dependent is
	// on the frontier.
	m := findMap(t, findSpace(t, g.h.Snapshot(ctx(t)), g.spaceID), "widget")
	if tk := findTicket(t, m, 1); tk.Status != "resolved" {
		t.Errorf("ticket status after a smeared promotion = %q, want resolved", tk.Status)
	}
}

// Send back: the fix-up session is briefed with the blocking finding and the
// human's note through the *payload* and its archive — and the note reaches
// nothing else. The ticket file is the permanent record and stays untouched; only
// abandonment writes there.
func TestSendBackNoteRidesThePayloadAndNowhereElse(t *testing.T) {
	g := atTheGate(t, rejectVerdict, false)
	// The follow-up runs a live, inert agent: this test is about what it was told,
	// not what it does. Installed last, so it shadows the proposing stub.
	harnesstest.StubAgent(t, "claude")

	const note = "the 500-row case is the one that matters — bench it before you touch the renderer"
	code, body := g.h.FollowUp(g.spaceID, "widget", 1, map[string]any{
		"role":       "implement",
		"note":       note,
		"advisories": []int{0},
	})
	if code != 200 {
		t.Fatalf("send back = %d, body %s", code, body)
	}
	var fr struct {
		SessionID string `json:"sessionId"`
	}
	if err := json.Unmarshal([]byte(body), &fr); err != nil {
		t.Fatalf("follow-up response not JSON: %v (%q)", err, body)
	}

	// --- The payload the session was actually handed. ---
	payload := readFile(t, filepath.Join(g.repo, ".wayfinder-harness", "run", fr.SessionID, "payload.md"))
	for _, want := range []string{note, blockingText, "empty-state path"} {
		if !strings.Contains(payload, want) {
			t.Errorf("the fix-up payload is missing %q:\n%s", want, payload)
		}
	}
	// --- And its archive, so the record survives the gitignored copy. ---
	archived := readFile(t, filepath.Join(g.h.DataDir, "sessions", fr.SessionID, "payload.md"))
	if !strings.Contains(archived, note) {
		t.Errorf("the archived payload is missing the operator's note")
	}

	// --- Nowhere else: not the ticket on disk, not the ticket's history. ---
	onDisk := readFile(t, filepath.Join(g.repo, g.ticketRel))
	if strings.Contains(onDisk, note) {
		t.Errorf("the operator's note leaked into the ticket file:\n%s", onDisk)
	}
	if strings.Contains(onDisk, blockingText) {
		t.Errorf("the reviewer's finding leaked into the ticket file:\n%s", onDisk)
	}
	if log := harnesstest.Git(t, g.repo, "log", "-p", "--", g.ticketRel); strings.Contains(log, note) {
		t.Errorf("the operator's note reached the ticket's history")
	}

	// The ticket stays proposed throughout — the fix-up comes back to this hub.
	c, cancel := context.WithTimeout(context.Background(), 8*time.Second)
	defer cancel()
	g.h.SnapshotUntil(c, func(m model.Model) bool {
		s := findSpaceOK(m, g.spaceID)
		return s != nil && ticketOK(mapOK(*s, "widget"), 1) != nil &&
			ticketOK(mapOK(*s, "widget"), 1).Status == "proposed"
	})
}

// Take it further: another session on the same still-proposed ticket. Its commits
// accumulate on the proposal and the `## Proposed Answer` is rewritten in place —
// one section, not a growing pile — with the prior text surviving in git.
func TestTakeItFurtherStacksCommitsAndRewritesTheProposal(t *testing.T) {
	g := atTheGate(t, passVerdict, false)

	const rewritten = "Amended: the list flushes every 500 rows now; the bench wall-time halved."
	harnesstest.StubRewritingAgent(t, "claude", g.ticketRel, rewritten, "work.txt")

	commitsBefore := len(strings.Split(harnesstest.Git(t, g.repo, "log", "--format=%H"), "\n"))

	code, body := g.h.FollowUp(g.spaceID, "widget", 1, map[string]any{"role": "implement"})
	if code != 200 {
		t.Fatalf("take it further = %d, body %s", code, body)
	}

	// The follow-up rewrites, commits, and dies; wait for its commit to land.
	waitFor(t, 10*time.Second, func() bool {
		return strings.Contains(gitQuiet(g.repo, "log", "--format=%s"), "Follow-up: amend the proposal")
	}, "the follow-up's commit")

	// --- Rewritten in place: one proposal section, carrying the new text. ---
	onDisk := readFile(t, filepath.Join(g.repo, g.ticketRel))
	if n := strings.Count(onDisk, "## Proposed Answer"); n != 1 {
		t.Errorf("the ticket carries %d `## Proposed Answer` headings, want exactly 1 rewritten in place:\n%s", n, onDisk)
	}
	if strings.Contains(onDisk, proposedBody) {
		t.Errorf("the prior proposal text is still in the file — it should live only in git:\n%s", onDisk)
	}

	// --- Commits stack; the priors stay in history. ---
	subjects := harnesstest.Git(t, g.repo, "log", "--format=%s")
	for _, want := range []string{"Propose answer", "Follow-up: amend the proposal"} {
		if !strings.Contains(subjects, want) {
			t.Errorf("history is missing %q — follow-up commits must accumulate, never replace:\n%s", want, subjects)
		}
	}
	commitsAfter := len(strings.Split(harnesstest.Git(t, g.repo, "log", "--format=%H"), "\n"))
	if commitsAfter <= commitsBefore {
		t.Errorf("commit count did not grow (%d → %d)", commitsBefore, commitsAfter)
	}
	if log := harnesstest.Git(t, g.repo, "log", "-p", "--", g.ticketRel); !strings.Contains(log, proposedBody) {
		t.Errorf("the prior proposal is not recoverable from git")
	}

	// --- And the ticket is still proposed, back at this hub. ---
	if tk := findTicket(t, findMap(t, findSpace(t, g.h.Snapshot(ctx(t)), g.spaceID), "widget"), 1); tk.Status != "proposed" {
		t.Errorf("ticket status after a follow-up = %q, want proposed — the gate has not been passed", tk.Status)
	}
}

// Abandon: one demotion commit, the reason addressed to the next attempt written
// into the ticket, and the ticket back on the frontier. It destroys nothing —
// with the levers untouched, every work commit is exactly where it was.
func TestAbandonDemotesToTheFrontierAndDestroysNothing(t *testing.T) {
	g := atTheGate(t, rejectVerdict, false)

	const reason = "the budget was measured on 50 rows; the next attempt must bench 500 first"
	workBefore := harnesstest.Git(t, g.repo, "log", "--format=%s")

	code, body := g.h.Abandon(g.spaceID, "widget", 1, map[string]any{"reason": reason})
	if code != 200 {
		t.Fatalf("abandon = %d, body %s", code, body)
	}
	var dr struct {
		Commit   string `json:"commit"`
		Reverted bool   `json:"reverted"`
		Reset    bool   `json:"reset"`
	}
	if err := json.Unmarshal([]byte(body), &dr); err != nil {
		t.Fatalf("abandon response not JSON: %v (%q)", err, body)
	}

	// --- The ticket derives open and is back on the frontier. ---
	m := findMap(t, findSpace(t, g.h.Snapshot(ctx(t)), g.spaceID), "widget")
	tk := findTicket(t, m, 1)
	if tk.Status != "open" {
		t.Errorf("abandoned ticket status = %q, want open", tk.Status)
	}
	if !tk.Frontier {
		t.Errorf("the abandoned ticket did not return to the frontier")
	}
	if dep := findTicket(t, m, 2); dep.Frontier {
		t.Errorf("abandonment leaked the dependent onto the frontier")
	}

	// --- The record: dated `### Rejected` prose carrying the reason, with the
	//     rejected proposal kept verbatim beneath it. ---
	onDisk := readFile(t, filepath.Join(g.repo, g.ticketRel))
	if strings.Contains(onDisk, "## Proposed Answer") {
		t.Errorf("the proposal was not demoted:\n%s", onDisk)
	}
	for _, want := range []string{"### Rejected — " + time.Now().UTC().Format("2006-01-02"), reason, proposedBody} {
		if !strings.Contains(onDisk, want) {
			t.Errorf("the demoted ticket is missing %q:\n%s", want, onDisk)
		}
	}
	if strings.Contains(onDisk, "claimed_by:") {
		t.Errorf("the stale claim survived abandonment — the ticket is not takeable:\n%s", onDisk)
	}

	// --- One pathspec-limited commit with the gate trailers. ---
	touched := strings.Fields(harnesstest.Git(t, g.repo, "show", "--name-only", "--format=", dr.Commit))
	if len(touched) != 1 || touched[0] != filepath.ToSlash(g.ticketRel) {
		t.Errorf("the demotion touched %v, want only %s", touched, g.ticketRel)
	}
	msg := harnesstest.Git(t, g.repo, "log", "-1", "--format=%B", dr.Commit)
	for _, want := range []string{"Ticket: 01", "Role: gate", "Verdict: abandoned"} {
		if !strings.Contains(msg, want) {
			t.Errorf("demotion commit missing trailer %q:\n%s", want, msg)
		}
	}

	// --- Levers untouched by default: nothing reverted, nothing reset, every work
	//     commit still in history. ---
	if dr.Reverted || dr.Reset {
		t.Errorf("abandon pulled a lever nobody ticked (reverted=%v reset=%v)", dr.Reverted, dr.Reset)
	}
	after := harnesstest.Git(t, g.repo, "log", "--format=%s")
	for _, line := range strings.Split(workBefore, "\n") {
		if !strings.Contains(after, line) {
			t.Errorf("abandonment lost the commit %q — it must destroy nothing", line)
		}
	}
	if strings.Contains(after, "Revert ") {
		t.Errorf("abandonment reverted work with the lever unticked:\n%s", after)
	}

	// A reason is the one thing it demands.
	if code, body := g.h.Abandon(g.spaceID, "widget", 2, map[string]any{"reason": ""}); code == 200 {
		t.Errorf("abandon accepted an empty reason: %d %s", code, body)
	}
}

// The hub renders the brief on disk and nothing else (story 62): what the GUI
// fetches is byte-identical to the markdown a TUI-only operator reads.
func TestHubReadsTheBriefOffDisk(t *testing.T) {
	g := atTheGate(t, rejectVerdict, true)

	code, body := g.h.ReviewRead(g.spaceID, "widget", 1)
	if code != 200 {
		t.Fatalf("review read = %d, body %s", code, body)
	}
	var rr struct {
		Brief          string `json:"brief"`
		Recommendation string `json:"recommendation"`
		Blocking       []struct {
			Text   string `json:"text"`
			Clause string `json:"clause"`
		} `json:"blocking"`
		Advisories []struct{} `json:"advisories"`
	}
	if err := json.Unmarshal([]byte(body), &rr); err != nil {
		t.Fatalf("review read not JSON: %v (%q)", err, body)
	}
	onDisk := readFile(t, filepath.Join(g.repo, ".wayfinder-harness", "run", g.reviewSID, "brief.md"))
	if rr.Brief != onDisk {
		t.Errorf("the hub's brief differs from the file on disk:\n--- served ---\n%s\n--- disk ---\n%s", rr.Brief, onDisk)
	}
	if rr.Recommendation != "Send back" {
		t.Errorf("recommendation = %q, want Send back", rr.Recommendation)
	}
	if len(rr.Blocking) != 1 || rr.Blocking[0].Clause == "" {
		t.Errorf("the hub was handed %d blocking findings, want exactly the anchored one: %+v", len(rr.Blocking), rr.Blocking)
	}

	// The same gate signal rides the snapshot, so the star-map and the queue know a
	// human is being waited on without opening the hub.
	tk := findTicket(t, findMap(t, findSpace(t, g.h.Snapshot(ctx(t)), g.spaceID), "widget"), 1)
	if tk.Review == nil {
		t.Fatalf("the snapshot carries no review signal on a ticket at the gate")
	}
	if tk.Review.Recommendation != "Send back" || tk.Review.Blocking != 1 {
		t.Errorf("snapshot review signal = %+v, want Send back with one blocking finding", tk.Review)
	}
}

func hasLiveSession(s model.Space) bool {
	for _, term := range s.Terminals {
		if term.Session != nil && term.Alive {
			return true
		}
	}
	return false
}

// waitFor polls cond until it holds or the deadline passes — for the asynchronous
// half of an action a stub agent completes on its own (it commits, then dies), where
// the signal is git history rather than the snapshot.
func waitFor(t *testing.T, within time.Duration, cond func() bool, what string) {
	t.Helper()
	deadline := time.Now().Add(within)
	for {
		if cond() {
			return
		}
		if time.Now().After(deadline) {
			t.Fatalf("timed out waiting for %s", what)
		}
		time.Sleep(30 * time.Millisecond)
	}
}

// gitQuiet runs git inside a poll, where a fatal helper would kill the wait.
func gitQuiet(repo string, args ...string) string {
	cmd := exec.Command("git", args...)
	cmd.Dir = repo
	out, _ := cmd.CombinedOutput()
	return string(out)
}
