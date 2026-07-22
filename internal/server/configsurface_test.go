package server_test

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/rengwu/chartr/internal/chartrtest"
	"github.com/rengwu/chartr/internal/model"
)

// Ticket 05 at the process boundary: the effective config surface. The read half
// is asserted through the pushed snapshot — every resolved value with the layer
// it came from and the file that layer lives in. The write half is asserted
// through the files on disk and the re-derived snapshot: a binding edit lands in
// the user layer and nowhere else, clearing it reveals the layer beneath, and the
// open action resolves only names the server knows.

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

func setBinding(t *testing.T, h *chartrtest.Chartr, spaceID string, body map[string]any) (int, string) {
	t.Helper()
	return h.Put("/api/spaces/"+spaceID+"/config/binding", body)
}

// The pushed model carries the whole surface: bindings with per-field provenance
// and PATH presence, the resolved skill library with the layer that won each
// directory, map kinds, and the path of every participating layer (stories 33–37).
func TestSnapshotCarriesTheEffectiveConfigSurface(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteFile(t, repo, ".chartr/config.toml", `
[roles.implement]
args = ["--model", "sonnet-ws"]

[maps."widget"]
kind = "implementation"
`)
	chartrtest.WriteFile(t, h.DataDir, "user.toml", fmt.Sprintf(`
[spaces.%q.roles.implement]
adapter = "codex"
`, repo))
	// A workspace fork of one skill: the committed layer wins its whole directory.
	chartrtest.WriteFile(t, repo, ".chartr/skills/implement/SKILL.md",
		"---\nname: implement\ndescription: this space's own\n---\n\nHouse rules.\n")
	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))

	resp := register(t, h, repo)
	snap := h.Snapshot(ctx(t))
	s := findSpace(t, snap, resp.ID)

	// Bindings: per-field provenance and the PATH probe.
	impl := binding(t, s, "implement")
	assertField(t, "implement.adapter", impl.Adapter, "codex", impl.AdapterFrom, "user")
	assertField(t, "implement.args", strings.Join(impl.Args, " "), "--model sonnet-ws", impl.ArgsFrom, "workspace")

	// Skills: the winning layer per whole directory, positively stated.
	if got := skill(t, s, "implement").Layer; got != "workspace" {
		t.Errorf("implement skill resolves from %q, want workspace", got)
	}
	if got := skill(t, s, "grill").Layer; got != "built-in" {
		t.Errorf("grill skill resolves from %q, want built-in", got)
	}
	if got := skill(t, s, "implement").Dir; got != filepath.Join(repo, ".chartr/skills/implement") {
		t.Errorf("implement skill dir = %q, want the workspace copy", got)
	}

	// Kinds, read-only, from the same push.
	if got := findMap(t, s, "widget").Kind; got != "implementation" {
		t.Errorf("map kind = %q, want implementation", got)
	}

	// Every participating layer names where it lives — the space's own on the
	// space, the shared ones on the model.
	if got, want := layer(t, s.Layers, "workspace-config").Path, filepath.Join(repo, ".chartr/config.toml"); got != want {
		t.Errorf("workspace config path = %q, want %q", got, want)
	}
	if !layer(t, s.Layers, "workspace-config").Exists {
		t.Error("workspace config exists on disk but the surface says it does not")
	}
	if got, want := layer(t, snap.Config, "user-config").Path, filepath.Join(h.DataDir, "user.toml"); got != want {
		t.Errorf("user config path = %q, want %q", got, want)
	}
	// The split the surface has to tell honestly: user bindings and user skills
	// are two different roots.
	if got, want := layer(t, snap.Config, "user-skills").Path, filepath.Join(h.ConfigDir, "skills"); got != want {
		t.Errorf("user skills path = %q, want %q", got, want)
	}
	if got, want := layer(t, snap.Config, "builtin-skills").Path, filepath.Join(h.DataDir, "skills"); got != want {
		t.Errorf("built-in skills path = %q, want %q", got, want)
	}
}

// A fork whose recorded `forked_from` no longer matches the shipped default is
// carried on the skill itself, not only as a warning — the surface states the
// stale-fork condition where it renders the skill (story 34).
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

// Editing a binding writes the user layer and only the user layer, leaves the
// operator's bytes around it intact, and re-derives so the new value and its new
// provenance reflect straight back (stories 39–41, 43).
func TestBindingEditWritesOnlyTheUserLayer(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	const workspaceCfg = `# committed, shared, and not the UI's to write
[roles.implement]
adapter = "claude"
args = ["--model", "sonnet-ws"]
`
	chartrtest.WriteFile(t, repo, ".chartr/config.toml", workspaceCfg)
	chartrtest.WriteFile(t, h.DataDir, "user.toml", "# my machine\n")

	resp := register(t, h, repo)

	if code, body := setBinding(t, h, resp.ID, map[string]any{
		"role": "implement", "field": "args", "value": []string{"--model", "opus"},
	}); code != 200 {
		t.Fatalf("set binding = %d, body %s", code, body)
	}

	// The value and its provenance are back over the socket with no reload.
	impl := binding(t, findSpace(t, h.Snapshot(ctx(t)), resp.ID), "implement")
	assertField(t, "implement.args", strings.Join(impl.Args, " "), "--model opus", impl.ArgsFrom, "user")
	assertField(t, "implement.adapter", impl.Adapter, "claude", impl.AdapterFrom, "workspace")

	// The committed workspace config is untouched, byte for byte.
	if got := readFile(t, filepath.Join(repo, ".chartr/config.toml")); got != workspaceCfg {
		t.Errorf("the UI wrote committed workspace config:\n%s", got)
	}
	// The user file kept the operator's comment and gained the override.
	user := readFile(t, filepath.Join(h.DataDir, "user.toml"))
	if !strings.HasPrefix(user, "# my machine\n") {
		t.Errorf("the binding edit rewrote the user file's head:\n%s", user)
	}
	if !strings.Contains(user, `args = ["--model", "opus"]`) {
		t.Errorf("the override is not in the user file:\n%s", user)
	}

	// Clearing it reveals the layer beneath — the edit is reversible.
	if code, body := setBinding(t, h, resp.ID, map[string]any{
		"role": "implement", "field": "args", "value": nil,
	}); code != 200 {
		t.Fatalf("clear binding = %d, body %s", code, body)
	}
	impl = binding(t, findSpace(t, h.Snapshot(ctx(t)), resp.ID), "implement")
	assertField(t, "implement.args", strings.Join(impl.Args, " "), "--model sonnet-ws", impl.ArgsFrom, "workspace")
}

// args round-trips as a list, and the surface refuses what it cannot honour —
// an unknown role or field, an unknown space — rather than writing anything.
func TestBindingEditArgsAndRefusals(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	resp := register(t, h, repo)

	if code, body := setBinding(t, h, resp.ID, map[string]any{
		"role": "grill", "field": "args", "value": []string{"--verbose", "-p"},
	}); code != 200 {
		t.Fatalf("set args = %d, body %s", code, body)
	}
	grill := binding(t, findSpace(t, h.Snapshot(ctx(t)), resp.ID), "grill")
	if len(grill.Args) != 2 || grill.Args[0] != "--verbose" || grill.ArgsFrom != "user" {
		t.Errorf("args resolved %v from %q, want [--verbose -p] from user", grill.Args, grill.ArgsFrom)
	}

	for name, body := range map[string]map[string]any{
		"unknown role":  {"role": "review", "field": "adapter", "value": "x"},
		"unknown field": {"role": "implement", "field": "kind", "value": "planning"},
		"empty value":   {"role": "implement", "field": "adapter", "value": ""},
		"retired field": {"role": "implement", "field": "model", "value": "opus"},
	} {
		if code, resp2 := setBinding(t, h, resp.ID, body); code != 400 {
			t.Errorf("%s: set binding = %d, want 400 (body %s)", name, code, resp2)
		}
	}
	if code, _ := h.Put("/api/spaces/no-such-space/config/binding", map[string]any{
		"role": "implement", "field": "adapter", "value": "x",
	}); code != 404 {
		t.Errorf("set binding on a missing space = %d, want 404", code)
	}
}

// Map kind stays classify-only: the surface renders it, and there is no
// config-surface write that sets it (ADR 0007 survives ticket 03's cut).
func TestSurfaceNeverWritesKind(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))
	resp := register(t, h, repo)

	if code, _ := setBinding(t, h, resp.ID, map[string]any{
		"role": "implement", "field": "kind", "value": "implementation",
	}); code != 400 {
		t.Error("the binding editor accepted a kind write; kind is classify-only")
	}
	if _, err := os.Stat(filepath.Join(repo, ".chartr/config.toml")); !os.IsNotExist(err) {
		t.Error("a refused edit wrote committed config")
	}
	if got := findMap(t, findSpace(t, h.Snapshot(ctx(t)), resp.ID), "widget").Kind; got != "" {
		t.Errorf("map kind = %q after a refused write, want unclassified", got)
	}
}

// The open action resolves a *named* layer server-side and refuses anything
// else, so a client-supplied path can never reach the editor (story 45).
func TestOpenResolvesNamedLayersOnly(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	chartrtest.WriteFile(t, repo, ".chartr/config.toml", "[roles.implement]\nadapter = \"x\"\n")
	resp := register(t, h, repo)

	// A stub editor on $VISUAL records what it was handed, so the test asserts the
	// path the *server* resolved rather than one it supplied.
	record := stubEditor(t)

	code, body := h.Post("/api/spaces/"+resp.ID+"/config/open", map[string]string{"layer": "workspace-config"})
	if code != 200 {
		t.Fatalf("open workspace-config = %d, body %s", code, body)
	}
	var opened struct {
		Path   string `json:"path"`
		Opened string `json:"opened"`
		Exists bool   `json:"exists"`
	}
	if err := json.Unmarshal([]byte(body), &opened); err != nil {
		t.Fatalf("open response not JSON: %v (%q)", err, body)
	}
	want := filepath.Join(repo, ".chartr/config.toml")
	if opened.Path != want || !opened.Exists || opened.Opened != "editor" {
		t.Errorf("open = %+v, want %q opened in the editor", opened, want)
	}
	if got := waitForFile(t, record); !strings.Contains(got, want) {
		t.Errorf("the editor was handed %q, want the server-resolved %q", got, want)
	}

	// A named skill directory resolves to the layer that actually won it.
	code, body = h.Post("/api/spaces/"+resp.ID+"/config/open", map[string]string{"layer": "skill:implement"})
	if code != 200 {
		t.Fatalf("open skill:implement = %d, body %s", code, body)
	}
	if !strings.Contains(body, filepath.Join(h.DataDir, "skills", "implement")) {
		t.Errorf("skill:implement resolved to %s, want the materialized built-in copy", body)
	}

	// Anything not a name the server knows is refused — including a path.
	for _, bad := range []string{
		"/etc/passwd",
		"../../../../etc/passwd",
		"skill:../../etc/passwd",
		"skill:no-such-skill",
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

	code, body := h.Post("/api/spaces/"+resp.ID+"/config/open", map[string]string{"layer": "workspace-config"})
	if code != 200 {
		t.Fatalf("open absent workspace-config = %d, body %s", code, body)
	}
	if !strings.Contains(body, `"exists":false`) || !strings.Contains(body, `"opened":"none"`) {
		t.Errorf("open of an absent layer = %s, want it surfaced as absent", body)
	}
	if !strings.Contains(body, filepath.Join(repo, ".chartr/config.toml")) {
		t.Errorf("open of an absent layer did not surface its path: %s", body)
	}
	if _, err := os.Stat(filepath.Join(repo, ".chartr/config.toml")); !os.IsNotExist(err) {
		t.Error("opening an absent layer created the file")
	}
}

// The global half of the surface stands on its own: the skill library resolves
// with no space in play, so "what are my skills and where do they live" is
// answerable with nothing registered — and the open action for those layers is
// reachable without borrowing a space id.
func TestGlobalSkillsResolveWithoutASpace(t *testing.T) {
	// A user fork shadows one whole directory; everything else stands on the
	// shipped floor. It is in place before the server starts, since nothing but a
	// registered space would prompt a rebuild here.
	configDir := t.TempDir()
	chartrtest.WriteFile(t, filepath.Join(configDir, "skills"), "grill/SKILL.md",
		"---\nname: grill\ndescription: mine\n---\n\nMy grill.\n")
	h := chartrtest.Start(t, chartrtest.WithConfigDir(configDir))

	snap := h.Snapshot(ctx(t))
	if len(snap.Spaces) != 0 {
		t.Fatalf("expected no registered spaces, got %d", len(snap.Spaces))
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
	if got, want := global("grill").Dir, filepath.Join(h.ConfigDir, "skills", "grill"); got != want {
		t.Errorf("grill dir = %q, want %q", got, want)
	}
	if got, want := global("core").Layer, "built-in"; got != want {
		t.Errorf("core resolves from %q, want %q", got, want)
	}
	// The method skills ship in the same library and resolve space-less too.
	for _, name := range []string{"wayfinder", "domain-modeling", "to-spec", "to-tickets"} {
		if got, want := global(name).Layer, "built-in"; got != want {
			t.Errorf("%s resolves from %q, want %q", name, got, want)
		}
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
	for _, bad := range []string{"/etc/passwd", "skill:../../etc/passwd", "workspace-config", ""} {
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
// it appears — the chartr starts the editor and deliberately does not wait on it.
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
