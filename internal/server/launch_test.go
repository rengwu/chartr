package server_test

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/rengwu/chartr/internal/chartrtest"
	"github.com/rengwu/chartr/internal/prompt"
)

// Ticket 01 at the process boundary: the skill launcher's spine. `POST /launch`
// runs any *on-ramp* skill on a chosen agent as a live, ticketless tab with an
// optional line of context — the ideate on-ramp generalised, so it shares only the
// adapter's spawn primitive with a real session (no map/ticket lookup, no claim,
// no Session, no death halt). The ideate route keeps working as the `skill=ideate`
// case; those assertions stay in ideate_test.go.

// launchAgent registers the agent these tests launch with, with a stub binary on
// PATH, and returns that stub's delivery log. The name is deliberately not a
// role's and not an adapter's — nothing about a launch resolves through either.
func launchAgent(t *testing.T, h *chartrtest.Chartr) string {
	t.Helper()
	log := chartrtest.StubAgent(t, "some-harness")
	registerAgent(t, h, "thinker", map[string]any{"adapter": "some-harness"})
	return log
}

// A launch opens a live tab carrying no Session, and the on-ramp skill's body
// reaches the agent through the same read-this-file opener a real session uses,
// byte-matching the composed payload. wayfinder opens cold (no context box), so
// this proves the bare, self-driving launch — ideate's twin on another skill.
func TestLaunchOpensOnRampSkillTab(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	deliveryLog := launchAgent(t, h)

	resp := register(t, h, repo)
	id := h.Launch(resp.ID, "thinker", "wayfinder", "")

	s := findSpace(t, h.Snapshot(ctx(t)), resp.ID)
	tab := findTerminal(t, s, id)
	if !tab.Alive {
		t.Errorf("launch tab is not alive")
	}
	if tab.Session != nil {
		t.Errorf("launch tab carries a Session binding %+v, want none — it must not read as a real session", tab.Session)
	}

	promptAbs := filepath.Join(repo, ".chartr", "run", id, "payload.md")
	got, err := os.ReadFile(promptAbs)
	if err != nil {
		t.Fatalf("launch prompt not written: %v", err)
	}
	if want := string(prompt.Launch(prompt.RootsFor(h.ConfigDir, repo), "wayfinder", "")); string(got) != want {
		t.Errorf("launch payload on disk does not match the composed skill body:\ngot:\n%s\nwant:\n%s", got, want)
	}

	log := chartrtest.WaitForFileContains(t, deliveryLog, promptAbs, 5*time.Second)
	if !strings.Contains(log, "Read the file") {
		t.Errorf("the opener the agent received did not read-this-file:\n%s", log)
	}
}

// A needs-context skill launched with a line of context carries that brief in the
// same on-disk payload it already opens — under the `## Your task` trailer, below
// the skill's body — not a fragile typed-in second line.
func TestLaunchThreadsContextIntoPayload(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	launchAgent(t, h)

	resp := register(t, h, repo)
	id := h.Launch(resp.ID, "thinker", "grill", "settle whether the widget cache is worth it")

	got, err := os.ReadFile(filepath.Join(repo, ".chartr", "run", id, "payload.md"))
	if err != nil {
		t.Fatalf("launch prompt not written: %v", err)
	}
	body := string(got)
	if !strings.Contains(body, "Role: grill") {
		t.Errorf("payload dropped the grill skill body:\n%s", body)
	}
	if !strings.Contains(body, "## Your task") || !strings.Contains(body, "settle whether the widget cache is worth it") {
		t.Errorf("the optional context did not ride in the payload:\n%s", body)
	}
}

// The pushed library is the allowlist: the server refuses a skill it does not
// resolve as on-ramp, the way spawn refuses a non-role — so a client (or a stale
// one) cannot launch an augmentative or second-step skill by merely naming it.
// A refused launch opens nothing and writes no payload.
func TestLaunchRefusesNonOnRampSkill(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	launchAgent(t, h)
	resp := register(t, h, repo)

	for _, skill := range []string{"implement", "core", "to-tickets", "no-such-skill"} {
		code, body := h.LaunchRaw(resp.ID, "thinker", skill, "")
		if code != 400 {
			t.Errorf("launch of %q = %d, want 400; body %s", skill, code, body)
		}
	}

	if s := findSpace(t, h.Snapshot(ctx(t)), resp.ID); len(s.Terminals) != 0 {
		t.Errorf("a refused launch opened a tab: %+v", s.Terminals)
	}
	if _, err := os.Stat(filepath.Join(repo, ".chartr", "run")); err == nil {
		t.Errorf("a refused launch wrote a payload into the run directory")
	}
}

// A launch settles the agent on the same doorstep a spawn and ideate use, refusing
// an unknown or PATH-absent agent the same two ways and in the same order — and
// opening nothing either way.
func TestLaunchRefusesAnUnknownOrAbsentAgent(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	chartrtest.StubAgent(t, "some-harness")

	resp := register(t, h, repo)
	registerAgent(t, h, "thinker", map[string]any{"adapter": "some-harness"})
	// Registered, but its binary was never put on PATH.
	registerAgent(t, h, "ghost", map[string]any{"adapter": "no-such-harness"})

	if code, body := h.LaunchRaw(resp.ID, "nobody", "wayfinder", ""); code != 400 {
		t.Errorf("launch with an unregistered agent = %d, want 400; body %s", code, body)
	}
	code, body := h.LaunchRaw(resp.ID, "ghost", "wayfinder", "")
	if code != 409 {
		t.Errorf("launch with a PATH-absent agent = %d, want 409; body %s", code, body)
	}
	if !strings.Contains(body, "no-such-harness") {
		t.Errorf("the refusal does not name the missing binary: %s", body)
	}

	if s := findSpace(t, h.Snapshot(ctx(t)), resp.ID); len(s.Terminals) != 0 {
		t.Errorf("a refused launch opened a tab: %+v", s.Terminals)
	}
}

// A launch remembers the agent it just ran — the same rule a spawn and ideate
// follow — so the next launch here opens on that choice. There is no remembered
// skill: the launcher is always a dropdown the operator picks from.
func TestLaunchRemembersTheAgent(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	launchAgent(t, h)

	resp := register(t, h, repo)
	h.Launch(resp.ID, "thinker", "wayfinder", "")

	if got := findSpace(t, h.Snapshot(ctx(t)), resp.ID).LastAgent; got != "thinker" {
		t.Fatalf("space remembered %q after launching, want thinker", got)
	}
}
