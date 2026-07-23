package server_test

import (
	"context"
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/rengwu/chartr/internal/chartrtest"
	"github.com/rengwu/chartr/internal/model"
)

// decodeSpawn parses a spawn/respawn success body (both return the same shape).
func decodeSpawn(t *testing.T, body string) spawnResp {
	t.Helper()
	var r spawnResp
	if err := json.Unmarshal([]byte(body), &r); err != nil {
		t.Fatalf("spawn response not JSON: %v (%q)", err, body)
	}
	return r
}

// decodeResume parses a resume success body.
func decodeResume(t *testing.T, body string) struct {
	SessionID string `json:"sessionId"`
	Resumed   bool   `json:"resumed"`
} {
	t.Helper()
	var r struct {
		SessionID string `json:"sessionId"`
		Resumed   bool   `json:"resumed"`
	}
	if err := json.Unmarshal([]byte(body), &r); err != nil {
		t.Fatalf("resume response not JSON: %v (%q)", err, body)
	}
	return r
}

// Ticket 10 at the process boundary: liveness and the death halt. With a stub
// agent that dies on cue, chartr detects the death, pins the dead session to
// its ticket with scrollback intact, and does nothing on its own — the operator
// resolves it exactly three ways (resume, respawn, release), each an HTTP action,
// so the absence of autonomous action is asserted, not assumed. Separately: a
// dirtied tree badges while a spawn still proceeds. Every
// assertion is on what the design makes public — snapshots, the filesystem, git.
//
// The "quiet" hint this file used to assert is gone (agent-state-detection ticket
// 01): it measured PTY silence, which any cursor blink resets, so it never fired
// for the TUI agents it was written for. A tab's activity now comes from the
// evidence the agent broadcasts about itself, asserted where that lives — the
// engine table test and the process-boundary test in internal/terminal.

// commitCount is the number of commits reachable from HEAD — one after a claim,
// two once a release or a re-claim appends its own commit.
func commitCount(t *testing.T, repo string) string {
	t.Helper()
	return chartrtest.Git(t, repo, "rev-list", "--count", "HEAD")
}

// ticketFileBody reads a ticket file's current bytes from the working tree.
func ticketFileBody(t *testing.T, repo, slug, filename string) string {
	t.Helper()
	b, err := os.ReadFile(filepath.Join(repo, ".plan", slug, "tickets", filename))
	if err != nil {
		t.Fatalf("reading ticket file: %v", err)
	}
	return string(b)
}

// spawnThenDie spawns a session against a dying stub and waits until it is pinned
// dead — the precondition every halt test starts from. It returns the dead
// session's id.
func spawnThenDie(t *testing.T, h *chartrtest.Chartr, spaceID, slug string, num int, role string) string {
	t.Helper()
	sp := mustSpawn(t, h, spaceID, slug, num, role)
	waitForDeadSession(t, h, spaceID)
	return sp.SessionID
}

// waitForDeadSession polls until the space's session tab reads dead and pinned.
func waitForDeadSession(t *testing.T, h *chartrtest.Chartr, spaceID string) model.Terminal {
	t.Helper()
	c, cancel := context.WithTimeout(context.Background(), 8*time.Second)
	defer cancel()
	m := h.SnapshotUntil(c, func(m model.Model) bool {
		tab := sessionTab(findSpace(t, m, spaceID))
		return tab != nil && !tab.Alive && tab.Status == model.TerminalDead
	})
	return *sessionTab(findSpace(t, m, spaceID))
}

// A session whose process exits is detected dead, stays pinned to its ticket with
// its scrollback preserved, and chartr takes no action of its own: the claim
// stands, no commit beyond it is written, and the dead session lingers untouched.
func TestDeadSessionHaltsPinnedWithScrollback(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	marker := chartrtest.StubDyingAgent(t, "claude")

	resp := register(t, h, repo)
	sid := spawnThenDie(t, h, resp.ID, "widget", 1, "implement")

	// The dead session is pinned to its ticket, bound as it was spawned.
	s := findSpace(t, h.Snapshot(ctx(t)), resp.ID)
	tab := sessionTab(s)
	if tab == nil {
		t.Fatalf("dead session dropped from the model instead of pinning to its ticket")
	}
	if tab.ID != sid || tab.Alive || tab.Status != model.TerminalDead {
		t.Errorf("pinned tab = {id:%s alive:%v status:%s}, want {%s false dead}", tab.ID, tab.Alive, tab.Status, sid)
	}
	if tab.Session == nil || tab.Session.TicketNum != 1 || tab.Session.MapSlug != "widget" {
		t.Errorf("dead session lost its ticket binding: %+v", tab.Session)
	}

	// Scrollback survives death: attaching the dead session's terminal socket
	// replays what the agent printed before it exited.
	tc := h.DialTerminal(ctx(t), sid)
	defer tc.Close()
	if out := tc.ReadUntil(ctx(t), marker); !strings.Contains(out, marker) {
		t.Errorf("dead session's scrollback did not survive; got %q", out)
	}

	// chartr took nothing on its own: the ticket still derives claimed, and
	// only the claim commit exists — no auto-release, no auto-requeue.
	if st := findTicket(t, findMap(t, s, "widget"), 1).Status; st != "claimed" {
		t.Errorf("ticket after a death = %q, want claimed (the stale claim stands)", st)
	}
	if n := commitCount(t, repo); n != "1" {
		t.Errorf("commits after a death = %s, want 1 (just the claim; nothing autonomous)", n)
	}

	// And it stays that way: a window later, still dead, still claimed, still one
	// commit — no state change without an operator call.
	time.Sleep(400 * time.Millisecond)
	s2 := findSpace(t, h.Snapshot(ctx(t)), resp.ID)
	if tab2 := sessionTab(s2); tab2 == nil || tab2.Alive {
		t.Errorf("dead session did not stay pinned across a window: %+v", tab2)
	}
	if st := findTicket(t, findMap(t, s2, "widget"), 1).Status; st != "claimed" {
		t.Errorf("ticket drifted without an operator action: %q", st)
	}
	if n := commitCount(t, repo); n != "1" {
		t.Errorf("a commit appeared with no operator action: count now %s", n)
	}
}

// Release: the third halt choice clears the claim back to the frontier. The ticket
// derives open and takeable again, recorded as its own pathspec-limited commit that
// removes the claim, and the dead tab drops.
func TestHaltReleaseReturnsTicketToFrontier(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	chartrtest.StubDyingAgent(t, "claude")

	resp := register(t, h, repo)
	sid := spawnThenDie(t, h, resp.ID, "widget", 1, "implement")

	if code, body := h.Release(resp.ID, sid); code != 200 {
		t.Fatalf("release = %d, body %s", code, body)
	}

	// The ticket is back on the frontier: open and takeable.
	s := findSpace(t, h.Snapshot(ctx(t)), resp.ID)
	tk := findTicket(t, findMap(t, s, "widget"), 1)
	if tk.Status != "open" || !tk.Frontier {
		t.Errorf("ticket after release = {status:%s frontier:%v}, want {open true}", tk.Status, tk.Frontier)
	}
	if tab := sessionTab(s); tab != nil {
		t.Errorf("release left the dead tab pinned: %+v", tab.Session)
	}

	// The release is its own commit, touching exactly the ticket file, and it
	// removed the claim.
	if n := commitCount(t, repo); n != "2" {
		t.Errorf("commits after release = %s, want 2 (claim + release)", n)
	}
	rel := filepath.Join(".plan", "widget", "tickets", "01-first.md")
	files := chartrtest.Git(t, repo, "show", "--name-only", "--format=", "HEAD")
	if got := nonEmptyLines(files); len(got) != 1 || got[0] != rel {
		t.Errorf("release commit touched %v, want exactly [%s]", got, rel)
	}
	if body := ticketFileBody(t, repo, "widget", "01-first.md"); strings.Contains(body, "claimed_by") {
		t.Errorf("release left claimed_by on the ticket:\n%s", body)
	}
}

// Respawn: a fresh session on the same ticket. A new claim supersedes the stale
// one (re-stamped in place, its own commit), so the ticket stays claimed but by the
// new session, and nothing is doubled.
func TestHaltRespawnStartsFreshOnSameTicket(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	chartrtest.StubDyingAgent(t, "claude")

	resp := register(t, h, repo)
	oldSid := spawnThenDie(t, h, resp.ID, "widget", 1, "implement")

	code, body := h.Respawn(resp.ID, oldSid)
	if code != 200 {
		t.Fatalf("respawn = %d, body %s", code, body)
	}
	newSid := decodeSpawn(t, body).SessionID
	if newSid == "" || newSid == oldSid {
		t.Fatalf("respawn session id = %q, want a fresh id (was %q)", newSid, oldSid)
	}

	// Two commits: the original claim and the re-claim; the ticket now names the
	// new session exactly once (re-stamped, not doubled), and still derives claimed.
	if n := commitCount(t, repo); n != "2" {
		t.Errorf("commits after respawn = %s, want 2 (claim + re-claim)", n)
	}
	tbody := ticketFileBody(t, repo, "widget", "01-first.md")
	if strings.Count(tbody, "claimed_by:") != 1 {
		t.Errorf("respawn did not re-stamp the claim cleanly:\n%s", tbody)
	}
	if !strings.Contains(tbody, "claimed_by: "+newSid) || strings.Contains(tbody, oldSid) {
		t.Errorf("ticket claim = wrong session after respawn:\n%s", tbody)
	}

	// The pinned dead tab is replaced by a session bound to the same ticket under
	// the new id (it may have died again against the dying stub — presence is what
	// matters, not liveness).
	s := findSpace(t, h.Snapshot(ctx(t)), resp.ID)
	if findTicket(t, findMap(t, s, "widget"), 1).Status != "claimed" {
		t.Errorf("ticket after respawn is not claimed")
	}
	tab := sessionTab(s)
	if tab == nil || tab.ID != newSid || tab.Session.TicketNum != 1 {
		t.Errorf("respawn did not seat a session on the same ticket: %+v", tab)
	}
}

// Resume: same-ticket crash recovery. The same session id relaunches on its own
// ticket; the claim stands (no new commit), and the payload is still in place for
// the agent to walk back into.
func TestHaltResumeRelaunchesSameSession(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	chartrtest.StubDyingAgent(t, "claude")

	resp := register(t, h, repo)
	sid := spawnThenDie(t, h, resp.ID, "widget", 1, "implement")

	code, body := h.Resume(resp.ID, sid)
	if code != 200 {
		t.Fatalf("resume = %d, body %s", code, body)
	}
	if got := decodeResume(t, body); got.SessionID != sid || !got.Resumed {
		t.Errorf("resume response = %+v, want same session id %s resumed", got, sid)
	}

	// Crash recovery carries nothing across and writes nothing: the claim stands as
	// the only commit, and the ticket still derives claimed by the same session.
	if n := commitCount(t, repo); n != "1" {
		t.Errorf("commits after resume = %s, want 1 (resume writes no claim)", n)
	}
	s := findSpace(t, h.Snapshot(ctx(t)), resp.ID)
	if findTicket(t, findMap(t, s, "widget"), 1).Status != "claimed" {
		t.Errorf("ticket after resume is not claimed")
	}
	tab := sessionTab(s)
	if tab == nil || tab.ID != sid || tab.Session.TicketNum != 1 {
		t.Errorf("resume did not seat the same session on its ticket: %+v", tab)
	}
	// The payload the opener points at is on disk for the relaunched agent.
	if _, err := os.Stat(filepath.Join(repo, ".chartr", "run", sid, "payload.md")); err != nil {
		t.Errorf("resume did not keep the session's payload in place: %v", err)
	}
}

// The halt actions require a dead session: a live one is refused, so nothing the
// operator has not explicitly ended can be resumed, respawned, or released.
func TestHaltRefusesLiveSession(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	chartrtest.StubAgent(t, "claude") // blocking: the session stays live

	resp := register(t, h, repo)
	sp := mustSpawn(t, h, resp.ID, "widget", 1, "implement")

	for _, act := range []struct {
		name string
		fn   func(string, string) (int, string)
	}{{"resume", h.Resume}, {"respawn", h.Respawn}, {"release", h.Release}} {
		if code, body := act.fn(resp.ID, sp.SessionID); code != 409 || !strings.Contains(body, "still live") {
			t.Errorf("%s of a live session = %d (%s), want 409 still-live", act.name, code, body)
		}
	}
}

// The `quiet` hint this file used to assert is gone (agent-state-detection ticket
// 01). It measured PTY silence, which any cursor blink resets, so it never fired
// for the TUI agents it was written for; a tab's activity now comes from the
// evidence the agent broadcasts about itself, and is asserted where that lives —
// the engine table test and the process-boundary test in internal/terminal.

// A dirtied working tree — debris a session or an ad-hoc shell left behind — is a
// badge, never a spawn gate: the space reports dirty, and a spawn into it still
// proceeds.
func TestDirtyTreeBadgesButSpawnProceeds(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	// Commit the map and config so the tree is clean to start, then leave debris.
	chartrtest.Git(t, repo, "add", "-A")
	chartrtest.Git(t, repo, "commit", "-q", "-m", "seed")
	chartrtest.StubAgent(t, "claude")

	resp := register(t, h, repo)
	if findSpace(t, h.Snapshot(ctx(t)), resp.ID).Dirty {
		t.Fatalf("precondition: a freshly committed tree reads dirty")
	}

	// Debris left in the working tree — as a session or an ad-hoc shell would.
	chartrtest.WriteFile(t, repo, "scratch.txt", "uncommitted debris\n")
	if !findSpace(t, h.SnapshotUntil(ctx(t), func(m model.Model) bool {
		return findSpace(t, m, resp.ID).Dirty
	}), resp.ID).Dirty {
		t.Fatalf("dirty tree not badged after leaving debris")
	}

	// The badge is not a gate: a spawn into the dirty tree still proceeds.
	registerAgent(t, h, "claude", map[string]any{"adapter": "claude"})
	if code, body := h.SpawnWithAgent(resp.ID, "widget", 1, "implement", "claude"); code != 200 {
		t.Fatalf("spawn into a dirty tree = %d (%s), want 200 — dirty is a badge, not a gate", code, body)
	}
	if findSpace(t, h.Snapshot(ctx(t)), resp.ID).Dirty != true {
		t.Errorf("tree should still read dirty after the spawn (the debris remains)")
	}
}

// Ticket 03: a respawn launches the agent the dead session ran, not a
// re-resolution of its role. "Start over cleanly" composes a fresh payload and
// writes a fresh claim; it does not change what executes.
func TestHaltRespawnReusesTheDeadSessionsAgent(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))

	// The role's bound adapter is present *and* recording, so re-resolving the
	// binding would leave a trace. The chosen agent's binary dies on cue, which is
	// what gets the session to the halt in the first place.
	claudeDelivery := chartrtest.StubAgent(t, "claude")
	chartrtest.StubDyingAgent(t, "some-harness")

	resp := register(t, h, repo)
	registerAgent(t, h, "harness-yolo", map[string]any{
		"adapter": "some-harness",
		"args":    []string{"-m", "big", "--think"},
	})

	code, body := h.SpawnWithAgent(resp.ID, "widget", 1, "implement", "harness-yolo")
	if code != 200 {
		t.Fatalf("spawn naming harness-yolo = %d, body %s", code, body)
	}
	oldSid := decodeSpawn(t, body).SessionID
	waitForDeadSession(t, h, resp.ID)

	code, body = h.Respawn(resp.ID, oldSid)
	if code != 200 {
		t.Fatalf("respawn = %d, body %s", code, body)
	}
	fresh := decodeSpawn(t, body)
	if fresh.Agent != "some-harness" || fresh.AgentName != "harness-yolo" {
		t.Errorf("respawn ran %q (%q), want some-harness (harness-yolo) — the dead session's own agent",
			fresh.Agent, fresh.AgentName)
	}
	if b, _ := os.ReadFile(claudeDelivery); len(b) > 0 {
		t.Errorf("respawn re-resolved the role's binding and launched claude:\n%s", b)
	}

	// The fresh claim records the same choice and the same mechanism as the first.
	msg := chartrtest.Git(t, repo, "log", "-1", "--format=%B")
	for _, w := range []string{"Agent: harness-yolo", "Adapter: some-harness", "Args: -m big --think"} {
		if !strings.Contains(msg, w) {
			t.Errorf("re-claim commit missing trailer %q:\n%s", w, msg)
		}
	}
}

// A respawn whose agent has since been deregistered is refused with the message
// any other absent agent gets — surfaced, never silently substituted onto
// whatever the role would have resolved to. The halt is left exactly as it was,
// so the operator can register the agent again and retry.
func TestHaltRespawnRefusesWhenTheAgentIsGone(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	chartrtest.StubAgent(t, "claude") // the role's binding, present and never wanted
	chartrtest.StubDyingAgent(t, "some-harness")

	resp := register(t, h, repo)
	registerAgent(t, h, "harness-yolo", map[string]any{"adapter": "some-harness"})

	code, body := h.SpawnWithAgent(resp.ID, "widget", 1, "implement", "harness-yolo")
	if code != 200 {
		t.Fatalf("spawn naming harness-yolo = %d, body %s", code, body)
	}
	oldSid := decodeSpawn(t, body).SessionID
	waitForDeadSession(t, h, resp.ID)
	before := commitCount(t, repo)

	if code, body := h.Delete("/api/config/agents/harness-yolo"); code != 200 {
		t.Fatalf("deleting harness-yolo = %d, body %s", code, body)
	}

	code, body = h.Respawn(resp.ID, oldSid)
	if code != 400 {
		t.Fatalf("respawn on a deregistered agent = %d, want 400; body %s", code, body)
	}
	if !strings.Contains(body, "harness-yolo") {
		t.Errorf("the refusal does not name the agent that is gone: %s", body)
	}

	// No re-claim, and the dead tab is still pinned to its ticket to retry from.
	if after := commitCount(t, repo); after != before {
		t.Errorf("a refused respawn wrote a commit: %s -> %s", before, after)
	}
	tab := sessionTab(findSpace(t, h.Snapshot(ctx(t)), resp.ID))
	if tab == nil || tab.ID != oldSid {
		t.Errorf("a refused respawn dropped the dead tab: %+v", tab)
	}
}
