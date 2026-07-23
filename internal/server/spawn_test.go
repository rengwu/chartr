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
	"github.com/rengwu/chartr/internal/config"
	"github.com/rengwu/chartr/internal/model"
	"github.com/rengwu/chartr/internal/prompt"
	"github.com/rengwu/chartr/internal/wayfinder"
)

// Ticket 09 at the process boundary: the spawn tracer bullet. With a stub agent
// CLI on PATH, spawning a frontier ticket writes the claim commit (pathspec +
// trailers), drops the gitignored payload whose content matches the preview,
// archives a per-session copy, and delivers the read-this-file opener to the
// agent — landing a live session tab bound to exactly one ticket. Binding
// a role to a missing binary hard-blocks that one spawn with the specific message
// and blocks nothing else. Every assertion is on what the design makes public —
// HTTP responses, the control-socket snapshot, the filesystem, and git history.

// spawnResp is the spawn action's own result.
type spawnResp struct {
	SessionID string `json:"sessionId"`
	TicketNum int    `json:"ticketNum"`
	Role      string `json:"role"`
	Agent     string `json:"agent"`
	// AgentName is the registered agent the operator picked, empty when the
	// request named none and the role's binding decided.
	AgentName  string   `json:"agentName"`
	Args       []string `json:"args"`
	PayloadSha string   `json:"payloadSha"`
}

// mustSpawn is the standard spawn for the many tests that care about a role, a
// ticket, or a session's lifecycle rather than about agent selection. A spawn now
// names an agent (ticket 04), so it registers a default `claude` agent —
// reproducing the old built-in default of `claude --model sonnet` — and spawns
// with it. Tests whose subject *is* the agent (its flags, delivery, or memory)
// call SpawnWithAgent directly instead. The registration is an idempotent PUT, so
// calling mustSpawn more than once in a space is harmless; it assumes a `claude`
// binary is on PATH, which every mustSpawn caller already stubs.
func mustSpawn(t *testing.T, h *chartrtest.Chartr, spaceID, slug string, num int, role string) spawnResp {
	t.Helper()
	registerAgent(t, h, "claude", map[string]any{"adapter": "claude", "args": []string{"--model", "sonnet"}})
	code, body := h.SpawnWithAgent(spaceID, slug, num, role, "claude")
	if code != 200 {
		t.Fatalf("spawn %s #%d as %s = %d, body %s", slug, num, role, code, body)
	}
	var r spawnResp
	if err := json.Unmarshal([]byte(body), &r); err != nil {
		t.Fatalf("spawn response not JSON: %v (%q)", err, body)
	}
	return r
}

// spawnWithAgent spawns naming a specific registered agent and asserts success —
// the helper for tests whose subject is the chosen agent (its flags, its audit
// trailers) rather than the role or lifecycle mustSpawn stands in for.
func spawnWithAgent(t *testing.T, h *chartrtest.Chartr, spaceID, slug string, num int, role, agent string) spawnResp {
	t.Helper()
	code, body := h.SpawnWithAgent(spaceID, slug, num, role, agent)
	if code != 200 {
		t.Fatalf("spawn %s #%d as %s with %s = %d, body %s", slug, num, role, agent, code, body)
	}
	return decodeSpawn(t, body)
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
// gitignored payload matching the preview, an archived copy, the opener reaching
// the agent, and a live session tab bound to exactly one ticket.
func TestSpawnWiresTheWholeChain(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))

	// A stub `claude` on PATH — the default `implement` binding's adapter — records
	// how it was launched and what was typed into it.
	deliveryLog := chartrtest.StubAgent(t, "claude")

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
		// The registered name the operator chose, and the binary and flags it
		// stands for: the name says which of their agents ran, the adapter and args
		// say what that means on any other machine.
		"Agent: claude",
		"Adapter: claude",
		"Args: --model sonnet",
		"Role: implement",
		"Payload-SHA256: " + sp.PayloadSha,
		// The content provenance, re-keyed from prompt parts to skills: which
		// layer won each composed skill, and the hash of the directory it won.
		"Skill: core=built-in:" + prompt.ShippedHash("core"),
		"Skill: implement=built-in:" + prompt.ShippedHash("implement"),
	} {
		if !strings.Contains(msg, want) {
			t.Errorf("claim commit message missing trailer %q:\n%s", want, msg)
		}
	}
	// No config-layer provenance: the explicit-agent path consults no layers, so
	// the `*-From` trailers are gone with the binding they described.
	if strings.Contains(msg, "-From:") {
		t.Errorf("claim commit carries config-layer provenance for an explicit agent:\n%s", msg)
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

	// --- The opener reached the agent, naming the payload to read. Which delivery
	// carried it is the adapter's business, asserted on its own below. ---
	log := chartrtest.WaitForFileContains(t, deliveryLog, payloadAbs, 5*time.Second)
	if !strings.Contains(log, "Read the file") {
		t.Errorf("the opener the agent received did not read this-file:\n%s", log)
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

// A spawn that names no agent is refused (ticket 04) — there is no binding to fall
// back to any more — and the refusal blocks nothing else: no claim is written, the
// ticket stays open with no session tab, and the space is still fully usable as a
// plain multiplexer. With agents registered the message is the "pick one" of a
// picker never opened, distinct from the empty-library case below.
func TestSpawnWithoutAnAgentIsRefusedAndBlocksNothingElse(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	chartrtest.StubAgent(t, "claude")

	resp := register(t, h, repo)
	// An agent exists, so the refusal is "pick one", not "register one".
	registerAgent(t, h, "claude", map[string]any{"adapter": "claude"})

	code, body := h.Spawn(resp.ID, "widget", 1, "implement") // Spawn sends no agent
	if code != 400 {
		t.Fatalf("spawn naming no agent = %d, want 400; body %s", code, body)
	}
	if !strings.Contains(body, "an agent is required") {
		t.Errorf("refusal does not say an agent must be picked: %s", body)
	}

	// Nothing was written: no commit (HEAD is still unborn), and the ticket is
	// still open with no session tab.
	if _, err := gitHEAD(repo); err == nil {
		t.Errorf("a refused spawn should write no claim commit, but HEAD exists")
	}
	s := findSpace(t, h.Snapshot(ctx(t)), resp.ID)
	if st := findTicket(t, findMap(t, s, "widget"), 1).Status; st != "open" {
		t.Errorf("ticket after a refused spawn = %q, want open", st)
	}
	if tab := sessionTab(s); tab != nil {
		t.Errorf("a refused spawn left a session tab: %+v", tab.Session)
	}

	// Blocks nothing else: the space is still a working multiplexer.
	termID := h.OpenTerminal(resp.ID)
	if !hasTerminal(findSpace(t, h.Snapshot(ctx(t)), resp.ID), termID) {
		t.Errorf("space unusable after a refused spawn — ad-hoc shell did not open")
	}
}

// With nothing registered at all, a spawn is refused with the distinct
// empty-library message that names the fix — where the fresh operator hit the
// wall, not only in settings — and, like every refusal, it leaves the space
// untouched: no claim, no payload, no tab.
func TestSpawnRefusedWhenLibraryEmpty(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))

	resp := register(t, h, repo)
	// Deliberately register nothing.

	code, body := h.Spawn(resp.ID, "widget", 1, "implement")
	if code != 409 {
		t.Fatalf("spawn against an empty library = %d, want 409; body %s", code, body)
	}
	// Distinct from "an agent is required" — it says the library is empty and how
	// to fix it.
	if !strings.Contains(body, "no agents are registered") || !strings.Contains(body, "register") {
		t.Errorf("empty-library refusal is not the specific message: %s", body)
	}

	// The space is exactly as it was: no claim commit, ticket open, no tab, and no
	// payload dropped into the run directory.
	if _, err := gitHEAD(repo); err == nil {
		t.Errorf("a refused spawn wrote a claim commit against an empty library")
	}
	s := findSpace(t, h.Snapshot(ctx(t)), resp.ID)
	if st := findTicket(t, findMap(t, s, "widget"), 1).Status; st != "open" {
		t.Errorf("ticket after an empty-library refusal = %q, want open", st)
	}
	if tab := sessionTab(s); tab != nil {
		t.Errorf("an empty-library refusal left a session tab: %+v", tab.Session)
	}
	if _, err := os.Stat(filepath.Join(repo, ".chartr", "run")); err == nil {
		t.Errorf("an empty-library refusal wrote a payload into the run directory")
	}
}

// A discovered map is live: with no committed chartr config at all, it spawns
// the moment it is found — and in a role the map itself would once have had to
// be declared `planning` to offer. There is no classification step left to gate
// it, and the route that used to write one is gone.
func TestDiscoveredMapSpawnsWithNoConfig(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	chartrtest.StubAgent(t, "claude")
	resp := register(t, h, repo)

	if sp := mustSpawn(t, h, resp.ID, "widget", 1, "grill"); sp.Role != "grill" {
		t.Errorf("spawned role = %q, want grill", sp.Role)
	}

	// The classify route is unregistered, so the POST falls through to the SPA
	// handler rather than 404ing (an unmatched /api/ path has always been served
	// index.html — pre-existing, not this cut's to change). What matters is that
	// it is inert: nothing handles it, so nothing is written.
	if code, body := h.Post(fmt.Sprintf("/api/spaces/%s/maps/widget/classify", resp.ID),
		map[string]any{"kind": "implementation"}); strings.Contains(body, "kind") {
		t.Errorf("something still answers classify: %d %s", code, body)
	}
	if _, err := os.Stat(filepath.Join(repo, ".chartr/config.toml")); !os.IsNotExist(err) {
		t.Error("a POST to the dead classify route wrote committed config")
	}
}

// The behavioural delta of the kind cut: a `task` ticket — which wayfinder
// explicitly permits on a planning map — spawns as `implement`, the role its own
// type names. The map's kind used to clamp that away to `grill`, which is not
// what the person who typed `type: task` meant.
func TestSpawnHonoursTheTicketsOwnType(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	chartrtest.StubAgent(t, "claude")

	// The role a `task` ticket defaults to, derived from the type alone.
	if got := config.RoleForTicketType(wayfinder.TypeTask); got != config.RoleImplement {
		t.Fatalf("default role for a task ticket = %q, want implement", got)
	}

	resp := register(t, h, repo)
	sp := mustSpawn(t, h, resp.ID, "widget", 1, "implement")
	if sp.Role != "implement" {
		t.Errorf("spawned role = %q, want implement", sp.Role)
	}

	// It really is an implement session, seated and bound to the ticket.
	tab := sessionTab(findSpace(t, h.Snapshot(ctx(t)), resp.ID))
	if tab == nil || tab.Session.Role != "implement" || tab.Session.TicketNum != 1 {
		t.Errorf("session tab after spawn = %+v, want ticket 1 / implement", tab)
	}
}

// A string that is not one of the four roles is a malformed request, answered
// 400 — not the 500 it became when the kind cut removed `KindOffersRole`, which
// had been the only thing checking the role was a role at all. The match is
// exact: a wrong-case role is not a role. The preview path answers the same
// input the same way, so the two never disagree about one request.
func TestSpawnRefusesAStringThatIsNotARole(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	chartrtest.StubAgent(t, "claude")
	resp := register(t, h, repo)

	for _, role := range []string{"bogus", "IMPLEMENT", "Implement"} {
		if code, body := h.Spawn(resp.ID, "widget", 1, role); code != 400 {
			t.Errorf("spawn as %q = %d (%s), want 400", role, code, body)
		}
		code, body := h.Get(fmt.Sprintf("/api/spaces/%s/maps/widget/tickets/1/payload?role=%s", resp.ID, role))
		if code != 400 {
			t.Errorf("payload preview as %q = %d (%s), want 400 — the two paths must agree", role, code, body)
		}
	}

	// The four real ones still go through, so the check refuses only non-roles.
	mustSpawn(t, h, resp.ID, "widget", 1, "grill")
}

// A non-frontier ticket is not a fresh spawn's to take: a ticket held behind an
// unresolved blocker is refused.
func TestSpawnRefusesNonFrontier(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
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
	chartrtest.WriteTicket(t, repo, "widget", "01-a.md", ticket(1, "A", "[]", "task", ""))
	chartrtest.WriteTicket(t, repo, "widget", "02-b.md", ticket(2, "B", "[]", "task", ""))
	chartrtest.StubAgent(t, "claude")
	resp := register(t, h, repo)

	mustSpawn(t, h, resp.ID, "widget", 1, "implement")
	// The second spawn names an agent so it reaches the one-session-per-space gate
	// rather than being turned away earlier for naming none.
	if code, body := h.SpawnWithAgent(resp.ID, "widget", 2, "implement", "claude"); code != 409 || !strings.Contains(body, "already has a live session") {
		t.Fatalf("second spawn = %d (%s), want 409 already-has-a-session", code, body)
	}
	// Ticket 2 was never claimed.
	if st := findTicket(t, findMap(t, findSpace(t, h.Snapshot(ctx(t)), resp.ID), "widget"), 2).Status; st != "open" {
		t.Errorf("ticket 2 after a refused second spawn = %q, want open", st)
	}
}

// Prompt delivery at the process boundary. A known agent is *told on its command
// line* — the opener is already submitted when the TUI comes up, so nothing waits
// on a human pressing enter. An operator running a harness that wants keystrokes
// instead registers an agent whose `prompt` says `type`, and the same opener
// arrives on stdin. Both are asserted through the agent process itself, which
// records how each line reached it. Delivery is a property of the chosen agent
// now, not of a role binding.
func TestSpawnDeliversTheOpenerTheWayTheAgentSays(t *testing.T) {
	for _, tc := range []struct {
		name   string
		prompt string
		want   string // the tagged line the agent must record
	}{
		{name: "argv by default", prompt: "argv", want: "argv: Read the file "},
		{name: "typed when the agent says so", prompt: "type", want: "stdin: Read the file "},
	} {
		t.Run(tc.name, func(t *testing.T) {
			h := chartrtest.Start(t)
			repo := chartrtest.NewSpaceRepo(t)

			chartrtest.WriteMap(t, repo, "widget", mapBody)
			chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
			delivery := chartrtest.StubAgent(t, "claude")

			resp := register(t, h, repo)
			registerAgent(t, h, "runner", map[string]any{"adapter": "claude", "prompt": tc.prompt})

			code, body := h.SpawnWithAgent(resp.ID, "widget", 1, "implement", "runner")
			if code != 200 {
				t.Fatalf("spawn = %d, body %s", code, body)
			}
			sp := decodeSpawn(t, body)

			payloadAbs := filepath.Join(repo, ".chartr", "run", sp.SessionID, "payload.md")
			log := chartrtest.WaitForFileContains(t, delivery, payloadAbs, 5*time.Second)
			if !strings.Contains(log, tc.want) {
				t.Errorf("the opener did not reach the agent as %q:\n%s", strings.TrimSuffix(tc.want, "Read the file "), log)
			}
		})
	}
}

// A prompt delivery the adapter seam cannot read never reaches the command line:
// the agent's own default stands and the operator is told, rather than the spawn
// dying or the CLI being handed a flag it will refuse. The library's own writer
// refuses such a delivery at the gate, so the case that survives is a hand-written
// agent whose delivery ResolveAgents drops on the way through.
func TestUnreadablePromptDeliveryWarnsAndFallsBack(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	// Hand-written, since the registration surface would refuse "stdin" outright.
	chartrtest.WriteFile(t, h.DataDir, "user.toml",
		"[agents.runner]\nadapter = \"claude\"\nprompt = \"stdin\"\n")
	delivery := chartrtest.StubAgent(t, "claude")

	resp := register(t, h, repo)
	s := findSpace(t, h.Snapshot(ctx(t)), resp.ID)
	if !hasSubstring(s.Warnings, "unreadable prompt delivery") {
		t.Errorf("no warning for an unreadable prompt delivery: %v", s.Warnings)
	}

	sp := spawnWithAgent(t, h, resp.ID, "widget", 1, "implement", "runner")
	payloadAbs := filepath.Join(repo, ".chartr", "run", sp.SessionID, "payload.md")
	log := chartrtest.WaitForFileContains(t, delivery, payloadAbs, 5*time.Second)
	if !strings.Contains(log, "argv: Read the file ") {
		t.Errorf("a typo in the delivery changed how the agent was launched:\n%s", log)
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

// Ticket 01 at the process boundary: the agent is chosen at the moment of
// spawning. The request may name a registered agent, that agent runs instead of
// whatever the role was bound to, and the space quietly remembers the choice.
// Sending no name is still the old behaviour exactly — the expand step adds a
// path beside the binding rather than replacing it.

// Naming a registered agent launches *that* agent, and every flag the operator
// typed reaches the process verbatim and in order. Both binaries are on PATH, so
// what is under test is the choice deciding — not the binding's adapter happening
// to be absent.
func TestSpawnWithAnExplicitAgentLaunchesThatAgent(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))

	claudeDelivery := chartrtest.StubAgent(t, "claude") // what `implement` is bound to
	delivery := chartrtest.StubAgent(t, "some-harness") // what the operator picks

	resp := register(t, h, repo)
	registerAgent(t, h, "harness-yolo", map[string]any{
		"adapter": "some-harness",
		"args":    []string{"-m", "big", "--dangerously-skip-permissions"},
		"prompt":  "argv",
	})

	code, body := h.SpawnWithAgent(resp.ID, "widget", 1, "implement", "harness-yolo")
	if code != 200 {
		t.Fatalf("spawn naming harness-yolo = %d, body %s", code, body)
	}
	var sp spawnResp
	if err := json.Unmarshal([]byte(body), &sp); err != nil {
		t.Fatalf("spawn response not JSON: %v (%q)", err, body)
	}
	if sp.Agent != "some-harness" || sp.AgentName != "harness-yolo" {
		t.Errorf("spawn ran %q (%q), want some-harness (harness-yolo)", sp.Agent, sp.AgentName)
	}

	payloadAbs := filepath.Join(repo, ".chartr", "run", sp.SessionID, "payload.md")
	log := chartrtest.WaitForFileContains(t, delivery, payloadAbs, 5*time.Second)
	want := []string{
		"argv: -m", "argv: big",
		"argv: --dangerously-skip-permissions",
		"argv: Read the file " + payloadAbs,
	}
	if !inOrder(log, want) {
		t.Errorf("the chosen agent's argv did not reach the process in order.\nwant %v\ngot:\n%s", want, log)
	}

	// The role's own binding was not consulted: nothing was launched as `claude`.
	if b, _ := os.ReadFile(claudeDelivery); len(b) > 0 {
		t.Errorf("the role's bound adapter ran even though an agent was named:\n%s", b)
	}

	// The claim carries the local name *and* what it means anywhere else.
	msg := chartrtest.Git(t, repo, "log", "-1", "--format=%B")
	for _, w := range []string{
		"Agent: harness-yolo",
		"Adapter: some-harness",
		"Args: -m big --dangerously-skip-permissions",
	} {
		if !strings.Contains(msg, w) {
			t.Errorf("claim commit missing trailer %q:\n%s", w, msg)
		}
	}
}

// Both refusals land on the doorstep — before the claim, before any write — so a
// blocked spawn leaves the space exactly as it was (story 33). An unregistered
// name is a malformed request; a registered agent whose binary has gone is a
// conflict carrying the library's own diagnosis, which is what stops a stale
// picker from launching nothing (story 18).
func TestSpawnRefusesAnUnknownOrAbsentAgentWithoutClaiming(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	chartrtest.StubAgent(t, "claude")

	resp := register(t, h, repo)
	// Registered, but its binary was never installed.
	registerAgent(t, h, "ghost", map[string]any{"adapter": "not-a-real-binary"})

	for _, tc := range []struct {
		name, agent, wantIn string
		wantCode            int
	}{
		{"unregistered", "never-registered", "never-registered", 400},
		{"off PATH", "ghost", "not-a-real-binary", 409},
	} {
		t.Run(tc.name, func(t *testing.T) {
			code, body := h.SpawnWithAgent(resp.ID, "widget", 1, "implement", tc.agent)
			if code != tc.wantCode {
				t.Fatalf("spawn naming %q = %d, want %d (body %s)", tc.agent, code, tc.wantCode, body)
			}
			if !strings.Contains(body, tc.wantIn) {
				t.Errorf("refusal does not say what was wrong: %s", body)
			}
		})
	}

	// No claim commit — HEAD is still unborn — and the ticket is still takeable.
	if _, err := gitHEAD(repo); err == nil {
		t.Errorf("a refused spawn should write no claim commit, but HEAD exists")
	}
	tk := findTicket(t, findMap(t, findSpace(t, h.Snapshot(ctx(t)), resp.ID), "widget"), 1)
	if tk.Status != "open" || !tk.Frontier {
		t.Errorf("ticket after a refused spawn = %q (frontier %v), want open and on the frontier", tk.Status, tk.Frontier)
	}
}

// The space remembers what it last spawned with: the name appears in the pushed
// model, a refused spawn does not disturb it, and it is a property of the space
// rather than of the running process — it survives restarting the server against
// the same data root (stories 12, 20).
func TestSpaceRemembersTheAgentItSpawnedWith(t *testing.T) {
	dataDir := t.TempDir()
	h := chartrtest.Start(t, chartrtest.WithDataDir(dataDir))
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	chartrtest.WriteTicket(t, repo, "widget", "02-second.md", ticket(2, "Second", "[]", "task", ""))
	chartrtest.StubAgent(t, "some-harness")

	resp := register(t, h, repo)
	registerAgent(t, h, "harness-yolo", map[string]any{"adapter": "some-harness"})
	registerAgent(t, h, "ghost", map[string]any{"adapter": "not-a-real-binary"})

	if code, body := h.SpawnWithAgent(resp.ID, "widget", 1, "implement", "harness-yolo"); code != 200 {
		t.Fatalf("spawn = %d, body %s", code, body)
	}
	if got := findSpace(t, h.Snapshot(ctx(t)), resp.ID).LastAgent; got != "harness-yolo" {
		t.Fatalf("space remembered %q after spawning, want harness-yolo", got)
	}

	// A refusal changes nothing durable — it is turned away before the memory is
	// touched, exactly as it is before the claim.
	if code, _ := h.SpawnWithAgent(resp.ID, "widget", 2, "implement", "ghost"); code != 409 {
		t.Fatalf("spawn naming an off-PATH agent = %d, want 409", code)
	}
	if got := findSpace(t, h.Snapshot(ctx(t)), resp.ID).LastAgent; got != "harness-yolo" {
		t.Errorf("a refused spawn changed the remembered agent to %q", got)
	}

	// A second server over the same data root reads the same memory: it is state
	// on the space, not on the tab.
	h2 := chartrtest.Start(t, chartrtest.WithDataDir(dataDir))
	if got := findSpace(t, h2.Snapshot(ctx(t)), resp.ID).LastAgent; got != "harness-yolo" {
		t.Errorf("remembered agent after restart = %q, want harness-yolo", got)
	}
}
