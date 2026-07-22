package server_test

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/rengwu/chartr/internal/chartrtest"
	"github.com/rengwu/chartr/internal/model"
	"github.com/rengwu/chartr/internal/prompt"
)

// Ticket 09 at the process boundary: the spawn tracer bullet. With a stub agent
// CLI on PATH, spawning a frontier ticket writes the claim commit (pathspec +
// trailers), drops the gitignored payload whose content matches the preview,
// archives a per-session copy, and delivers the read-this-file opener to the
// agent's stdin — landing a live session tab bound to exactly one ticket. Binding
// a role to a missing binary hard-blocks that one spawn with the specific message
// and blocks nothing else. Every assertion is on what the design makes public —
// HTTP responses, the control-socket snapshot, the filesystem, and git history.

// implConfig is a committed workspace config declaring one map as an
// implementation map, so its `implement` role is offered (an unclassified map
// offers none).
func implConfig(slug string) string {
	return fmt.Sprintf("[maps.%q]\nkind = \"implementation\"\n", slug)
}

// spawnResp is the spawn action's own result.
type spawnResp struct {
	SessionID  string `json:"sessionId"`
	TicketNum  int    `json:"ticketNum"`
	Role       string `json:"role"`
	Agent      string `json:"agent"`
	Model      string `json:"model"`
	PayloadSha string `json:"payloadSha"`
}

func mustSpawn(t *testing.T, h *chartrtest.Chartr, spaceID, slug string, num int, role string) spawnResp {
	t.Helper()
	code, body := h.Spawn(spaceID, slug, num, role)
	if code != 200 {
		t.Fatalf("spawn %s #%d as %s = %d, body %s", slug, num, role, code, body)
	}
	var r spawnResp
	if err := json.Unmarshal([]byte(body), &r); err != nil {
		t.Fatalf("spawn response not JSON: %v (%q)", err, body)
	}
	return r
}

// gitIgnored reports whether git ignores rel within repo (check-ignore exits 0
// when the path is ignored, 1 when it is not) — proof the payload can never be
// swept into a commit.
func gitIgnored(repo, rel string) bool {
	cmd := exec.Command("git", "check-ignore", rel)
	cmd.Dir = repo
	return cmd.Run() == nil
}

func sessionTab(s model.Space) *model.Terminal {
	for i := range s.Terminals {
		if s.Terminals[i].Session != nil {
			return &s.Terminals[i]
		}
	}
	return nil
}

// The whole chain from one click: claim commit (pathspec-limited, trailers),
// gitignored payload matching the preview, an archived copy, the opener at the
// agent's stdin, and a live session tab bound to exactly one ticket.
func TestSpawnWiresTheWholeChain(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteFile(t, repo, ".chartr/config.toml", implConfig("widget"))
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))

	// A stub `claude` on PATH — the default `implement` binding's adapter — records
	// whatever is typed into it.
	stdinLog := chartrtest.StubAgent(t, "claude")

	resp := register(t, h, repo)
	sp := mustSpawn(t, h, resp.ID, "widget", 1, "implement")

	// --- The claim commit: pathspec-limited to the one ticket, with trailers. ---
	rel := filepath.Join(".plan", "widget", "tickets", "01-first.md")
	files := chartrtest.Git(t, repo, "show", "--name-only", "--format=", "HEAD")
	if got := nonEmptyLines(files); len(got) != 1 || got[0] != rel {
		t.Errorf("claim commit touched %v, want exactly [%s]", got, rel)
	}
	msg := chartrtest.Git(t, repo, "log", "-1", "--format=%B")
	for _, want := range []string{
		"Session: " + sp.SessionID,
		"Agent: claude",
		"Model: sonnet",
		"Role: implement",
		"Payload-SHA256: " + sp.PayloadSha,
		// The content provenance, re-keyed from prompt parts to skills: which
		// layer won each composed skill, and the hash of the directory it won.
		"Skill: core=built-in:" + prompt.ShippedHash("core"),
		"Skill: implement=built-in:" + prompt.ShippedHash("implement"),
		"Adapter-From: built-in",
	} {
		if !strings.Contains(msg, want) {
			t.Errorf("claim commit message missing trailer %q:\n%s", want, msg)
		}
	}

	// The ticket now derives `claimed`.
	if st := findTicket(t, findMap(t, findSpace(t, h.Snapshot(ctx(t)), resp.ID), "widget"), 1).Status; st != "claimed" {
		t.Errorf("ticket status after spawn = %q, want claimed", st)
	}

	// --- The gitignored payload, matching the preview word for word. ---
	payloadRel := filepath.Join(".chartr", "run", sp.SessionID, "payload.md")
	payloadAbs := filepath.Join(repo, payloadRel)
	got, err := os.ReadFile(payloadAbs)
	if err != nil {
		t.Fatalf("session payload not written: %v", err)
	}
	_, preview, body := getPayload(t, h, resp.ID, "widget", 1, "implement")
	if body != "" && string(got) != preview.Markdown {
		t.Errorf("gitignored payload does not match the preview:\n--- payload ---\n%s\n--- preview ---\n%s", got, preview.Markdown)
	}
	if !gitIgnored(repo, payloadRel) {
		t.Errorf("session payload %s is not gitignored — it could be swept into a commit", payloadRel)
	}

	// --- The archived copy in chartr state, outside the repo. ---
	archive := filepath.Join(h.DataDir, "sessions", sp.SessionID, "payload.md")
	arch, err := os.ReadFile(archive)
	if err != nil {
		t.Fatalf("session payload not archived: %v", err)
	}
	if string(arch) != preview.Markdown {
		t.Errorf("archived payload does not match the preview")
	}

	// --- The opener arrived at the agent's stdin, naming the payload to read. ---
	log := chartrtest.WaitForFileContains(t, stdinLog, payloadAbs, 5*time.Second)
	if !strings.Contains(log, "Read the file") {
		t.Errorf("opener typed into the agent did not read this-file:\n%s", log)
	}

	// --- A live session tab, bound to exactly one ticket. ---
	tab := sessionTab(findSpace(t, h.Snapshot(ctx(t)), resp.ID))
	if tab == nil {
		t.Fatalf("no session tab after spawn")
	}
	if !tab.Alive {
		t.Errorf("session tab is not alive")
	}
	if tab.Session.TicketNum != 1 || tab.Session.Role != "implement" || tab.Session.Agent != "claude" || tab.Session.MapSlug != "widget" {
		t.Errorf("session binding = %+v, want ticket 1 / implement / claude / widget", tab.Session)
	}
}

// An absent bound agent hard-blocks that one spawn with the specific message —
// naming the binding, its source layer, and the local-override fix — and blocks
// nothing else: no claim is written, the ticket stays open, and the space is still
// fully usable as a plain multiplexer.
func TestSpawnMissingAgentBlocksOnlyThatSpawn(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	// Classify implementation and bind `implement` to a binary that cannot exist.
	cfg := implConfig("widget") + "\n[roles.implement]\nadapter = \"wf-absent-agent-xyz\"\n"
	chartrtest.WriteFile(t, repo, ".chartr/config.toml", cfg)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))

	resp := register(t, h, repo)

	code, body := h.Spawn(resp.ID, "widget", 1, "implement")
	if code != 409 {
		t.Fatalf("spawn with an absent agent = %d, want 409; body %s", code, body)
	}
	for _, want := range []string{"wf-absent-agent-xyz", "PATH", "implement", "local override"} {
		if !strings.Contains(body, want) {
			t.Errorf("block message missing %q: %s", want, body)
		}
	}

	// Nothing was written: no commit (HEAD is still unborn), and the ticket is
	// still open with no session tab.
	if _, err := gitHEAD(repo); err == nil {
		t.Errorf("a blocked spawn should write no claim commit, but HEAD exists")
	}
	s := findSpace(t, h.Snapshot(ctx(t)), resp.ID)
	if st := findTicket(t, findMap(t, s, "widget"), 1).Status; st != "open" {
		t.Errorf("ticket after a blocked spawn = %q, want open", st)
	}
	if tab := sessionTab(s); tab != nil {
		t.Errorf("a blocked spawn left a session tab: %+v", tab.Session)
	}

	// Blocks nothing else: the space is still a working multiplexer.
	termID := h.OpenTerminal(resp.ID)
	if !hasTerminal(findSpace(t, h.Snapshot(ctx(t)), resp.ID), termID) {
		t.Errorf("space unusable after a blocked spawn — ad-hoc shell did not open")
	}
}

// An unclassified map offers no sessions (ADR 0007): spawn is refused, and a role
// from the other lifecycle is refused even once classified.
func TestSpawnRespectsKind(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	chartrtest.StubAgent(t, "claude")
	resp := register(t, h, repo)

	// Unclassified: no sessions at all.
	if code, body := h.Spawn(resp.ID, "widget", 1, "implement"); code != 409 || !strings.Contains(body, "unclassified") {
		t.Fatalf("spawn on an unclassified map = %d (%s), want 409 naming unclassified", code, body)
	}

	// Classified implementation: a planning role (grill) is not offered.
	chartrtest.WriteFile(t, repo, ".chartr/config.toml", implConfig("widget"))
	if code, body := h.Spawn(resp.ID, "widget", 1, "grill"); code != 400 || !strings.Contains(body, "not offered") {
		t.Fatalf("grill on an implementation map = %d (%s), want 400 not-offered", code, body)
	}
}

// A non-frontier ticket is not a fresh spawn's to take: a ticket held behind an
// unresolved blocker is refused.
func TestSpawnRefusesNonFrontier(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteFile(t, repo, ".chartr/config.toml", implConfig("widget"))
	chartrtest.WriteTicket(t, repo, "widget", "01-open.md", ticket(1, "Open blocker", "[]", "task", ""))
	chartrtest.WriteTicket(t, repo, "widget", "02-held.md", ticket(2, "Held", "[1]", "task", ""))
	chartrtest.StubAgent(t, "claude")
	resp := register(t, h, repo)

	// Ticket 2 is blocked by the still-open ticket 1 — not on the frontier.
	if code, body := h.Spawn(resp.ID, "widget", 2, "implement"); code != 409 || !strings.Contains(body, "frontier") {
		t.Fatalf("spawn on a held ticket = %d (%s), want 409 not-on-frontier", code, body)
	}
}

// One session per space at a time: a second spawn while a session is live is
// refused, and the refusal writes no second claim.
func TestSpawnOneSessionPerSpace(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteFile(t, repo, ".chartr/config.toml", implConfig("widget"))
	chartrtest.WriteTicket(t, repo, "widget", "01-a.md", ticket(1, "A", "[]", "task", ""))
	chartrtest.WriteTicket(t, repo, "widget", "02-b.md", ticket(2, "B", "[]", "task", ""))
	chartrtest.StubAgent(t, "claude")
	resp := register(t, h, repo)

	mustSpawn(t, h, resp.ID, "widget", 1, "implement")
	if code, body := h.Spawn(resp.ID, "widget", 2, "implement"); code != 409 || !strings.Contains(body, "already has a live session") {
		t.Fatalf("second spawn = %d (%s), want 409 already-has-a-session", code, body)
	}
	// Ticket 2 was never claimed.
	if st := findTicket(t, findMap(t, findSpace(t, h.Snapshot(ctx(t)), resp.ID), "widget"), 2).Status; st != "open" {
		t.Errorf("ticket 2 after a refused second spawn = %q, want open", st)
	}
}

func nonEmptyLines(s string) []string {
	var out []string
	for _, l := range strings.Split(s, "\n") {
		if l = strings.TrimSpace(l); l != "" {
			out = append(out, l)
		}
	}
	return out
}

func gitHEAD(repo string) (string, error) {
	cmd := exec.Command("git", "rev-parse", "--verify", "HEAD")
	cmd.Dir = repo
	out, err := cmd.CombinedOutput()
	return strings.TrimSpace(string(out)), err
}
