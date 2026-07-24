package server_test

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/rengwu/chartr/internal/chartrtest"
	"github.com/rengwu/chartr/internal/model"
)

// The settings surface at the process boundary (ticket 05): the agent library and
// the paths of the files behind it, each openable in the operator's editor. There
// is no committed execution layer and no per-field provenance any more — ADR 0014
// is superseded — so what is asserted is the library, the file paths, and the
// named-layer open action that refuses anything it does not itself resolve.

func layer(t *testing.T, layers []model.ConfigLayer, name string) model.ConfigLayer {
	t.Helper()
	for _, l := range layers {
		if l.Name == name {
			return l
		}
	}
	t.Fatalf("config layer %q not in %+v", name, layers)
	return model.ConfigLayer{}
}

func skill(t *testing.T, s model.Space, name string) model.ResolvedSkill {
	t.Helper()
	for _, sk := range s.Skills {
		if sk.Name == name {
			return sk
		}
	}
	t.Fatalf("skill %q not in space %s (%d skills)", name, s.Name, len(s.Skills))
	return model.ResolvedSkill{}
}

// The pushed model carries the surface: the operator's registered agent library,
// and the path of every file behind it. A repository that still carries a
// pre-cut `.chartr/config.toml` (old role bindings and a stale `[maps.*]` table)
// must cost a real checkout nothing — it is neither read, warned about, nor an
// error (story 36).
func TestSnapshotCarriesTheAgentLibraryAndPaths(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	// A leftover committed config from before this cut. Nothing reads it now.
	chartrtest.WriteFile(t, repo, ".chartr/config.toml", `
[roles.implement]
adapter = "codex"
args = ["--model", "sonnet-ws"]

[maps."widget"]
kind = "implementation"
`)
	// The agent library lives in the operator's own config.
	chartrtest.WriteFile(t, h.ConfigDir, "user.toml", `
[agents.house]
adapter = "claude"
args = ["--model", "opus"]
`)
	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))

	resp := register(t, h, repo)
	snap := h.Snapshot(ctx(t))
	s := findSpace(t, snap, resp.ID)

	// The library is on the snapshot, resolved from the operator's config alone.
	if len(snap.Agents) != 1 || snap.Agents[0].Name != "house" {
		t.Fatalf("library = %+v, want the one registered agent", snap.Agents)
	}

	// The leftover config.toml is inert: neither read nor warned about.
	if len(s.Warnings) != 0 {
		t.Errorf("a space carrying a stale .chartr/config.toml warned: %v", s.Warnings)
	}

	// Every file behind the library names where it lives — the shared ones on the
	// model, the space's committed skill library on the space.
	if got, want := layer(t, snap.Config, "user-config").Path, filepath.Join(h.ConfigDir, "user.toml"); got != want {
		t.Errorf("user config path = %q, want %q", got, want)
	}
	if got := layer(t, snap.Config, "user-config").Holds; got != "agents" {
		t.Errorf("user config holds %q, want agents", got)
	}
	if got, want := layer(t, snap.Config, "user-skills").Path, filepath.Join(h.ConfigDir, "skills"); got != want {
		t.Errorf("user skills path = %q, want %q", got, want)
	}
	if got, want := layer(t, snap.Config, "builtin-skills").Path, filepath.Join(h.ConfigDir, "builtin-skills"); got != want {
		t.Errorf("built-in skills path = %q, want %q", got, want)
	}
	if got, want := layer(t, s.Layers, "workspace-skills").Path, filepath.Join(repo, ".chartr/skills"); got != want {
		t.Errorf("workspace skills path = %q, want %q", got, want)
	}
	// Execution is no longer a committed layer: the space carries no config file.
	for _, l := range s.Layers {
		if l.Name == "workspace-config" {
			t.Errorf("the surface still lists a committed workspace-config layer: %+v", l)
		}
	}
}

// A fork whose recorded `forked_from` no longer matches the shipped default is
// carried on the skill itself and surfaced as a warning — skill resolution is the
// content half of the config story, untouched by this cut (story 34).
func TestResolvedSkillCarriesStaleFork(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	chartrtest.WriteFile(t, repo, ".chartr/skills/grill/SKILL.md",
		"---\nname: grill\ndescription: forked\nforked_from: deadbeef\n---\n\nMine.\n")

	resp := register(t, h, repo)
	s := findSpace(t, h.Snapshot(ctx(t)), resp.ID)

	sk := skill(t, s, "grill")
	if sk.Layer != "workspace" {
		t.Errorf("forked grill resolves from %q, want workspace", sk.Layer)
	}
	if sk.ForkedFrom != "deadbeef" || !sk.Stale {
		t.Errorf("forked grill: forkedFrom=%q stale=%v, want deadbeef and stale", sk.ForkedFrom, sk.Stale)
	}
	if !hasSubstring(s.Warnings, "grill") {
		t.Errorf("a stale fork produced no warning; warnings = %v", s.Warnings)
	}
}

// The open action resolves a *named* layer server-side and refuses anything
// else, so a client-supplied path can never reach the editor (story 45). The name
// a committed config layer used to have is now simply unknown, and refused.
func TestOpenResolvesNamedLayersOnly(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	chartrtest.WriteFile(t, h.ConfigDir, "user.toml", "[agents.house]\nadapter = \"claude\"\n")
	resp := register(t, h, repo)

	// A stub editor on $VISUAL records what it was handed, so the test asserts the
	// path the *server* resolved rather than one it supplied.
	record := stubEditor(t)

	code, body := h.Post("/api/spaces/"+resp.ID+"/config/open", map[string]string{"layer": "user-config"})
	if code != 200 {
		t.Fatalf("open user-config = %d, body %s", code, body)
	}
	want := filepath.Join(h.ConfigDir, "user.toml")
	if !strings.Contains(body, `"opened":"editor"`) || !strings.Contains(body, want) {
		t.Errorf("open user-config = %s, want %q opened in the editor", body, want)
	}
	if got := waitForFile(t, record); !strings.Contains(got, want) {
		t.Errorf("the editor was handed %q, want the server-resolved %q", got, want)
	}

	// A named skill directory resolves to the layer that actually won it.
	code, body = h.Post("/api/spaces/"+resp.ID+"/config/open", map[string]string{"layer": "skill:implement"})
	if code != 200 {
		t.Fatalf("open skill:implement = %d, body %s", code, body)
	}
	if !strings.Contains(body, filepath.Join(h.ConfigDir, "builtin-skills", "implement")) {
		t.Errorf("skill:implement resolved to %s, want the materialized built-in copy", body)
	}

	// Anything not a name the server knows is refused — including a path and the
	// name the retired committed-config layer used to answer to.
	for _, bad := range []string{
		"/etc/passwd",
		"../../../../etc/passwd",
		"skill:../../etc/passwd",
		"skill:no-such-skill",
		"workspace-config",
		"",
	} {
		if code, _ := h.Post("/api/spaces/"+resp.ID+"/config/open",
			map[string]string{"layer": bad}); code != 400 {
			t.Errorf("open %q = %d, want 400 — only server-known names resolve", bad, code)
		}
	}
}

// A layer with nothing on disk yet is reported with its path and left alone: the
// surface says where the value would go, and a read-shaped action creates nothing.
func TestOpenAbsentLayerSurfacesThePath(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	resp := register(t, h, repo)

	// The space's committed skill library does not exist yet.
	code, body := h.Post("/api/spaces/"+resp.ID+"/config/open", map[string]string{"layer": "workspace-skills"})
	if code != 200 {
		t.Fatalf("open absent workspace-skills = %d, body %s", code, body)
	}
	if !strings.Contains(body, `"exists":false`) || !strings.Contains(body, `"opened":"none"`) {
		t.Errorf("open of an absent layer = %s, want it surfaced as absent", body)
	}
	want := filepath.Join(repo, ".chartr/skills")
	if !strings.Contains(body, want) {
		t.Errorf("open of an absent layer did not surface its path: %s", body)
	}
	if _, err := os.Stat(want); !os.IsNotExist(err) {
		t.Error("opening an absent layer created the directory")
	}
}

// The global half of the surface stands on its own: the agent library and the
// skill library both resolve with no space in play, so "what are my agents and
// skills and where do they live" is answerable with nothing registered — and the
// open action for those layers is reachable without borrowing a space id.
func TestGlobalLayersResolveWithoutASpace(t *testing.T) {
	// A user fork shadows one whole skill directory; everything else stands on the
	// shipped floor. It is in place before the server starts, since nothing but a
	// registered space would prompt a rebuild here.
	configDir := t.TempDir()
	chartrtest.WriteFile(t, filepath.Join(configDir, "skills"), "grill/SKILL.md",
		"---\nname: grill\ndescription: mine\n---\n\nMy grill.\n")
	h := chartrtest.Start(t, chartrtest.WithConfigDir(configDir))
	chartrtest.WriteFile(t, h.ConfigDir, "user.toml", "[agents.house]\nadapter = \"claude\"\n")
	// Nudge a rebuild so the freshly written library is on the snapshot.
	register(t, h, chartrtest.NewSpaceRepo(t))

	snap := h.Snapshot(ctx(t))
	if len(snap.Agents) != 1 || snap.Agents[0].Name != "house" {
		t.Fatalf("global library = %+v, want the one registered agent", snap.Agents)
	}
	if len(snap.Skills) == 0 {
		t.Fatal("the global scope lists no skills; it should never take a space to read the library")
	}
	global := func(name string) model.ResolvedSkill {
		t.Helper()
		for _, sk := range snap.Skills {
			if sk.Name == name {
				return sk
			}
		}
		t.Fatalf("skill %q not in the global library (%d skills)", name, len(snap.Skills))
		return model.ResolvedSkill{}
	}
	if got, want := global("grill").Layer, "user"; got != want {
		t.Errorf("forked grill resolves from %q, want %q", got, want)
	}
	if got, want := global("core").Layer, "built-in"; got != want {
		t.Errorf("core resolves from %q, want %q", got, want)
	}

	// The space-less open resolves the same named layers, and refuses everything
	// else exactly as the per-space one does.
	record := stubEditor(t)
	code, body := h.Post("/api/config/open", map[string]string{"layer": "skill:grill"})
	if code != 200 {
		t.Fatalf("open skill:grill = %d, body %s", code, body)
	}
	want := filepath.Join(h.ConfigDir, "skills", "grill")
	if !strings.Contains(body, want) {
		t.Errorf("open skill:grill = %s, want the user fork at %q", body, want)
	}
	if got := waitForFile(t, record); !strings.Contains(got, want) {
		t.Errorf("the editor was handed %q, want the server-resolved %q", got, want)
	}
	for _, bad := range []string{"/etc/passwd", "skill:../../etc/passwd", "workspace-config", "workspace-skills", ""} {
		if code, _ := h.Post("/api/config/open", map[string]string{"layer": bad}); code != 400 {
			t.Errorf("global open %q = %d, want 400 — only global names resolve here", bad, code)
		}
	}
}

// stubEditor installs a $VISUAL that records its arguments instead of opening
// anything, and returns the record file's path.
func stubEditor(t *testing.T) string {
	t.Helper()
	dir := t.TempDir()
	record := filepath.Join(dir, "opened.log")
	script := filepath.Join(dir, "stub-editor")
	body := fmt.Sprintf("#!/bin/sh\nprintf '%%s\\n' \"$@\" >> %q\n", record)
	if err := os.WriteFile(script, []byte(body), 0o755); err != nil {
		t.Fatalf("writing stub editor: %v", err)
	}
	t.Setenv("VISUAL", script)
	return record
}

// waitForFile reads a file the stub editor writes asynchronously, retrying until
// it appears — chartr starts the editor and deliberately does not wait on it.
func waitForFile(t *testing.T, path string) string {
	t.Helper()
	for i := 0; i < 100; i++ {
		if b, err := os.ReadFile(path); err == nil && len(b) > 0 {
			return string(b)
		}
		time.Sleep(10 * time.Millisecond)
	}
	t.Fatalf("%s never appeared; the editor was not launched", path)
	return ""
}
