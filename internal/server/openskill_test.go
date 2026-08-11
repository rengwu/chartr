package server_test

import (
	"encoding/json"
	"os/exec"
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/chartrtest"
)

// The role bindings row's link button (chartr-skills-link, ticket 01): open a
// resolved `Source/skill` two different ways depending on what kind of source
// it came from. A dir source's skill is a folder on the operator's own
// machine, revealed there; a git source's skill lives in a checkout chartr
// owns and resets on refresh, so this hands back the source's own repository
// URL instead of anything local.
//
// The dir case does not assert on `opened`/`with` — that depends on a real
// file manager being launchable in the test environment, which
// TestTheSourcesFilesAreOpenableByName already declines to assert for the
// analogous editor case. What is under test here is resolution: which shape a
// given source kind gets, and the refusal for a ref that resolves to nothing.

type openSkillResult struct {
	Kind   string `json:"kind"`
	Path   string `json:"path"`
	Exists bool   `json:"exists"`
	URL    string `json:"url"`
}

func TestOpeningADirSkillAnswersWithItsLocalFile(t *testing.T) {
	h := chartrtest.Start(t)

	code, body := h.Post("/api/config/sources/open", map[string]string{"ref": "chartr-skills/grill"})
	if code != 200 {
		t.Fatalf("opening a dir source's skill = %d %s", code, body)
	}
	var r openSkillResult
	if err := json.Unmarshal([]byte(body), &r); err != nil {
		t.Fatalf("decoding response: %v\n%s", err, body)
	}
	if r.Kind != "local" {
		t.Fatalf("kind = %q, want %q (%s)", r.Kind, "local", body)
	}
	if !r.Exists || !strings.HasSuffix(r.Path, "/grill/SKILL.md") {
		t.Fatalf("path = %q, exists = %v, want an existing .../grill/SKILL.md", r.Path, r.Exists)
	}
}

func TestOpeningAGitSkillAnswersWithTheRepositoryURL(t *testing.T) {
	if _, err := exec.LookPath("git"); err != nil {
		t.Skip("git is not on PATH")
	}
	h := chartrtest.Start(t)

	origin := t.TempDir()
	writeSkill(t, origin, "grill", skillBody)
	for _, args := range [][]string{
		{"init", "--quiet"},
		{"add", "."},
		{"-c", "user.email=t@example.com", "-c", "user.name=t", "commit", "--quiet", "-m", "skills"},
	} {
		cmd := exec.Command("git", args...)
		cmd.Dir = origin
		if out, err := cmd.CombinedOutput(); err != nil {
			t.Fatalf("git %v: %v\n%s", args, err, out)
		}
	}
	if code, body := h.Post("/api/config/sources", map[string]string{
		"name": "upstream", "kind": "git", "url": origin,
	}); code != 200 {
		t.Fatalf("registering the git source: %d %s", code, body)
	}

	code, body := h.Post("/api/config/sources/open", map[string]string{"ref": "upstream/grill"})
	if code != 200 {
		t.Fatalf("opening a git source's skill = %d %s", code, body)
	}
	var r openSkillResult
	if err := json.Unmarshal([]byte(body), &r); err != nil {
		t.Fatalf("decoding response: %v\n%s", err, body)
	}
	if r.Kind != "remote" {
		t.Fatalf("kind = %q, want %q (%s)", r.Kind, "remote", body)
	}
	if r.URL != origin {
		t.Fatalf("url = %q, want the registered repository URL %q", r.URL, origin)
	}
}

func TestOpeningAnUnresolvableSkillRefIsRefused(t *testing.T) {
	h := chartrtest.Start(t)

	if code, body := h.Post("/api/config/sources/open", map[string]string{"ref": "chartr-skills/no-such-skill"}); code != 404 {
		t.Fatalf("opening an unresolvable ref = %d, want 404 (%s)", code, body)
	}
}
