package server_test

import (
	"encoding/json"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/rengwu/chartr/internal/chartrtest"
	"github.com/rengwu/chartr/internal/model"
)

// The agent library at the process boundary (registering agents from the surface,
// assigning them to roles, spawning through one). The assertions that matter are
// the ones a curated flag list could never make: whatever the operator typed
// reaches the launched process verbatim, in order, whether or not this repository
// has ever heard of the flag.

func agents(t *testing.T, h *chartrtest.Chartr) []model.Agent {
	t.Helper()
	return h.Snapshot(ctx(t)).Agents
}

func registerAgent(t *testing.T, h *chartrtest.Chartr, name string, body map[string]any) {
	t.Helper()
	if code, resp := h.Put("/api/config/agents/"+name, body); code != 200 {
		t.Fatalf("registering agent %s = %d, body %s", name, code, resp)
	}
}

func assignAgent(t *testing.T, h *chartrtest.Chartr, spaceID, role string, name any) {
	t.Helper()
	code, resp := h.Put("/api/spaces/"+spaceID+"/config/binding",
		map[string]any{"role": role, "field": "agent", "value": name})
	if code != 200 {
		t.Fatalf("assigning %s to %v = %d, body %s", role, name, code, resp)
	}
}

// The whole loop the surface drives: register an agent with flags this repository
// knows nothing about, assign a role to it, and spawn — and every flag arrives on
// the process, in the order it was registered, with the opener still delivered.
func TestRegisteredAgentDrivesTheSpawn(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteFile(t, repo, ".chartr/config.toml", implConfig("widget"))
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	delivery := chartrtest.StubAgent(t, "some-harness")

	resp := register(t, h, repo)

	// Flags no adapter row models — the ordinary case, and the reason the library
	// takes an opaque list rather than a curated set of toggles. The model is one
	// of them: this harness spells it `-m`, and nothing here had to be taught that.
	registerAgent(t, h, "harness-yolo", map[string]any{
		"adapter": "some-harness",
		"args":    []string{"-m", "big", "--dangerously-skip-permissions", "--sandbox", "danger-full-access"},
		"prompt":  "argv",
	})
	assignAgent(t, h, resp.ID, "implement", "harness-yolo")

	// The binding now reads as the agent, wholesale.
	b := binding(t, findSpace(t, h.Snapshot(ctx(t)), resp.ID), "implement")
	if b.Agent != "harness-yolo" || b.Adapter != "some-harness" {
		t.Fatalf("binding after assignment = %+v", b)
	}

	sp := mustSpawn(t, h, resp.ID, "widget", 1, "implement")
	payloadAbs := filepath.Join(repo, ".chartr", "run", sp.SessionID, "payload.md")
	log := chartrtest.WaitForFileContains(t, delivery, payloadAbs, 5*time.Second)

	// Every argument the operator typed, in order, then the opener last — the
	// positional cannot be pushed out of final place by the agent's own flags.
	want := []string{
		"argv: -m", "argv: big",
		"argv: --dangerously-skip-permissions",
		"argv: --sandbox", "argv: danger-full-access",
		"argv: Read the file " + payloadAbs,
	}
	if !inOrder(log, want) {
		t.Errorf("the registered agent's argv did not reach the process in order.\nwant %v\ngot:\n%s", want, log)
	}

	// The claim commit records what actually ran — the argv, not an agent-and-model
	// pair, so the permission and sandbox flags are in the audit trail beside the
	// model rather than invisible to it.
	msg := chartrtest.Git(t, repo, "log", "-1", "--format=%B")
	for _, w := range []string{
		"Agent: some-harness",
		"Args: -m big --dangerously-skip-permissions --sandbox danger-full-access",
	} {
		if !strings.Contains(msg, w) {
			t.Errorf("claim commit missing %q:\n%s", w, msg)
		}
	}
}

// Nothing is ever added to an agent's argv but the opener: no `--model`, no
// anything. What the operator registered is what launches.
func TestNothingIsAddedToTheRegisteredArgv(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteFile(t, repo, ".chartr/config.toml", implConfig("widget"))
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	delivery := chartrtest.StubAgent(t, "bare-harness")

	resp := register(t, h, repo)
	registerAgent(t, h, "plain", map[string]any{
		"adapter": "bare-harness",
		"args":    []string{"--fast"},
		"prompt":  "argv",
	})
	assignAgent(t, h, resp.ID, "implement", "plain")

	sp := mustSpawn(t, h, resp.ID, "widget", 1, "implement")
	payloadAbs := filepath.Join(repo, ".chartr", "run", sp.SessionID, "payload.md")
	log := chartrtest.WaitForFileContains(t, delivery, payloadAbs, 5*time.Second)
	if strings.Contains(log, "--model") {
		t.Errorf("a flag nobody registered was invented:\n%s", log)
	}
	if !inOrder(log, []string{"argv: --fast", "argv: Read the file "}) {
		t.Errorf("argv = \n%s", log)
	}
}

// The library is global: one registration serves every space, and each space
// assigns independently. This is the whole reason the library does not live in a
// repository — and the reason assignment does.
func TestOneLibraryServesEverySpace(t *testing.T) {
	h := chartrtest.Start(t)
	repoA, repoB := chartrtest.NewSpaceRepo(t), chartrtest.NewSpaceRepo(t)
	a, b := register(t, h, repoA), register(t, h, repoB)

	registerAgent(t, h, "shared", map[string]any{"adapter": "claude", "args": []string{"--model", "opus"}})
	assignAgent(t, h, a.ID, "implement", "shared")

	snap := h.Snapshot(ctx(t))
	if len(snap.Agents) != 1 || snap.Agents[0].Name != "shared" {
		t.Fatalf("library = %+v, want one shared agent", snap.Agents)
	}
	if got := binding(t, findSpace(t, snap, a.ID), "implement").Agent; got != "shared" {
		t.Errorf("space A implement.agent = %q, want shared", got)
	}
	if got := binding(t, findSpace(t, snap, b.ID), "implement").Agent; got != "" {
		t.Errorf("assigning in space A also assigned space B (%q) — assignment is per space", got)
	}

	// Nothing was written into either repository: the library and its assignments
	// are the operator's, and a teammate cannot be handed a permission-skipping
	// agent by pulling.
	for _, repo := range []string{repoA, repoB} {
		if _, err := gitHEAD(repo); err == nil {
			t.Errorf("%s grew a commit from a library edit", repo)
		}
	}
}

// The library's own edits: what the surface writes is what the snapshot reads
// back, including the resolved delivery and the command preview the operator is
// shown — which comes from the same seam that builds the real argv.
func TestLibraryRoundTripsThroughTheSnapshot(t *testing.T) {
	h := chartrtest.Start(t)

	registerAgent(t, h, "claude-yolo", map[string]any{
		"adapter": "claude",
		"args":    []string{"--model", "sonnet", "--dangerously-skip-permissions"},
	})
	lib := agents(t, h)
	if len(lib) != 1 {
		t.Fatalf("library = %+v, want one agent", lib)
	}
	got := lib[0]
	if got.Prompt != "" || got.Delivery != "argv" {
		t.Errorf("delivery = %q (prompt %q), want claude's own argv default resolved for the surface", got.Delivery, got.Prompt)
	}
	want := "claude --model sonnet --dangerously-skip-permissions ‹opener›"
	if strings.Join(got.Command, " ") != want {
		t.Errorf("command preview = %q, want %q", strings.Join(got.Command, " "), want)
	}

	// An update is a replacement: a dropped flag is dropped, not merged back in.
	registerAgent(t, h, "claude-yolo", map[string]any{"adapter": "claude", "prompt": "type"})
	got = agents(t, h)[0]
	if len(got.Args) != 0 {
		t.Errorf("update merged the old spec back in: %+v", got)
	}
	if got.Delivery != "type" || strings.Join(got.Command, " ") != "claude" {
		t.Errorf("typed agent = %q / command %q, want the opener off the command line", got.Delivery, got.Command)
	}

	// Delete reports what it stranded, and empties the library.
	code, body := h.Delete("/api/config/agents/claude-yolo")
	if code != 200 {
		t.Fatalf("delete = %d, body %s", code, body)
	}
	if len(agents(t, h)) != 0 {
		t.Errorf("library survived the delete: %+v", agents(t, h))
	}
}

// Deleting an agent a role is assigned to names the assignment rather than
// quietly rewriting it, and the stranded role keeps spawning off its own fields.
func TestDeletingAnAssignedAgentReportsAndStrands(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	resp := register(t, h, repo)

	registerAgent(t, h, "doomed", map[string]any{"adapter": "claude", "args": []string{"--model", "opus"}})
	assignAgent(t, h, resp.ID, "implement", "doomed")

	code, body := h.Delete("/api/config/agents/doomed")
	if code != 200 {
		t.Fatalf("delete = %d, body %s", code, body)
	}
	var out struct {
		Assigned []string `json:"assigned"`
	}
	if err := json.Unmarshal([]byte(body), &out); err != nil {
		t.Fatalf("delete response not JSON: %v (%s)", err, body)
	}
	if len(out.Assigned) != 1 || !strings.Contains(out.Assigned[0], "implement") {
		t.Errorf("delete reported assignments %v, want the one implement assignment", out.Assigned)
	}

	// The role still resolves — visibly stranded, never blocked.
	b := binding(t, findSpace(t, h.Snapshot(ctx(t)), resp.ID), "implement")
	if b.AgentMissing == "" {
		t.Errorf("stranded role carries no explanation: %+v", b)
	}
	if b.Adapter != "claude" || strings.Join(b.Args, " ") != "--model sonnet" {
		t.Errorf("stranded role did not fall back to its own fields: %+v", b)
	}
}

// The surface refuses what it cannot hold, with the reason, rather than writing
// something the resolver would then have to ignore.
func TestAgentSurfaceRefusals(t *testing.T) {
	h := chartrtest.Start(t)

	for name, body := range map[string]map[string]any{
		"no adapter":           {"args": []string{"-m", "x"}},
		"bad delivery":         {"adapter": "claude", "prompt": "stdin"},
		"flags in the adapter": {"adapter": "claude --yolo"},
	} {
		if code, resp := h.Put("/api/config/agents/x", body); code != 400 {
			t.Errorf("%s: PUT = %d (%s), want 400", name, code, resp)
		}
	}
	if code, resp := h.Put("/api/config/agents/not%20a%20name", map[string]any{"adapter": "claude"}); code != 400 {
		t.Errorf("exotic name: PUT = %d (%s), want 400", code, resp)
	}
	if len(agents(t, h)) != 0 {
		t.Errorf("a refused registration still landed in the library: %+v", agents(t, h))
	}
}

// inOrder reports whether every want appears in s, each after the one before it.
func inOrder(s string, want []string) bool {
	at := 0
	for _, w := range want {
		i := strings.Index(s[at:], w)
		if i < 0 {
			return false
		}
		at += i + len(w)
	}
	return true
}
