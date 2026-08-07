package server_test

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/rengwu/chartr/internal/chartrtest"
	"github.com/rengwu/chartr/internal/config"
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

	// Every file behind the library names where it lives.
	if got, want := layer(t, snap.Config, "user-config").Path, filepath.Join(h.ConfigDir, "user.toml"); got != want {
		t.Errorf("user config path = %q, want %q", got, want)
	}
	if got := layer(t, snap.Config, "user-config").Holds; got != "agents" {
		t.Errorf("user config holds %q, want agents", got)
	}
	// Neither execution nor skills is a committed layer any more: the space
	// carries no config file of its own at all (ADR 0017).
	if len(s.Layers) != 0 {
		t.Errorf("the surface still lists space-scoped config layers: %+v", s.Layers)
	}
	// The retired skill layers are gone from the global list with them.
	for _, l := range snap.Config {
		if l.Holds == "skills" {
			t.Errorf("the surface still lists a skill layer: %+v", l)
		}
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

	// Anything not a name the server knows is refused — including a path and every
	// name the retired committed-config and skill layers used to answer to.
	for _, bad := range []string{
		"/etc/passwd",
		"../../../../etc/passwd",
		"skill:implement",
		"skill:../../etc/passwd",
		"workspace-config",
		"workspace-skills",
		"user-skills",
		"builtin-skills",
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

	// The operator has never written a notification config.
	code, body := h.Post("/api/spaces/"+resp.ID+"/config/open", map[string]string{"layer": "notify-config"})
	if code != 200 {
		t.Fatalf("open absent notify-config = %d, body %s", code, body)
	}
	if !strings.Contains(body, `"exists":false`) || !strings.Contains(body, `"opened":"none"`) {
		t.Errorf("open of an absent layer = %s, want it surfaced as absent", body)
	}
	want := filepath.Join(h.ConfigDir, "notify.toml")
	if !strings.Contains(body, want) {
		t.Errorf("open of an absent layer did not surface its path: %s", body)
	}
	if _, err := os.Stat(want); !os.IsNotExist(err) {
		t.Error("opening an absent layer created the file")
	}
}

// The global half of the surface stands on its own: the agent library resolves
// with no space in play, so "what are my agents and where do they live" is
// answerable with nothing registered — and the open action for those layers is
// reachable without borrowing a space id.
func TestGlobalLayersResolveWithoutASpace(t *testing.T) {
	h := chartrtest.Start(t)
	chartrtest.WriteFile(t, h.ConfigDir, "user.toml", "[agents.house]\nadapter = \"claude\"\n")
	// Nudge a rebuild so the freshly written library is on the snapshot.
	register(t, h, chartrtest.NewSpaceRepo(t))

	snap := h.Snapshot(ctx(t))
	if len(snap.Agents) != 1 || snap.Agents[0].Name != "house" {
		t.Fatalf("global library = %+v, want the one registered agent", snap.Agents)
	}

	// The space-less open resolves the same named layers, and refuses everything
	// else exactly as the per-space one does.
	record := stubEditor(t)
	code, body := h.Post("/api/config/open", map[string]string{"layer": "user-config"})
	if code != 200 {
		t.Fatalf("open user-config = %d, body %s", code, body)
	}
	want := filepath.Join(h.ConfigDir, "user.toml")
	if !strings.Contains(body, want) {
		t.Errorf("open user-config = %s, want the agent library at %q", body, want)
	}
	if got := waitForFile(t, record); !strings.Contains(got, want) {
		t.Errorf("the editor was handed %q, want the server-resolved %q", got, want)
	}
	for _, bad := range []string{"/etc/passwd", "skill:grill", "workspace-config", "workspace-skills", "user-skills", ""} {
		if code, _ := h.Post("/api/config/open", map[string]string{"layer": bad}); code != 400 {
			t.Errorf("global open %q = %d, want 400 — only global names resolve here", bad, code)
		}
	}
}

// The create action is the companion to open for a layer that does not exist yet:
// it stamps the file from its bundled defaults template so there is something to
// open. It writes only a layer that carries a template, never clobbers an existing
// file, and the freshly-created file shows up as existing on the next snapshot.
func TestCreateStampsTerminalConfigFromDefaults(t *testing.T) {
	h := chartrtest.Start(t)
	// A registered space so a rebuild pushes a fresh snapshot after the create.
	register(t, h, chartrtest.NewSpaceRepo(t))

	path := filepath.Join(h.ConfigDir, "terminal.toml")
	if _, err := os.Stat(path); !os.IsNotExist(err) {
		t.Fatalf("terminal.toml already present before create: %v", err)
	}
	// The surface shows the layer as not-yet-created before the action.
	if layer(t, h.Snapshot(ctx(t)).Config, "terminal-config").Exists {
		t.Fatal("terminal-config reported existing before it was created")
	}

	code, body := h.Post("/api/config/create", map[string]string{"layer": "terminal-config"})
	if code != 200 {
		t.Fatalf("create terminal-config = %d, body %s", code, body)
	}
	if !strings.Contains(body, `"created":true`) || !strings.Contains(body, path) {
		t.Errorf("create terminal-config = %s, want created at %q", body, path)
	}

	// The file is on disk with the scaffold's exact bytes, and the surface now shows
	// the layer as existing (and thus openable).
	b, err := os.ReadFile(path)
	if err != nil {
		t.Fatalf("reading created terminal.toml: %v", err)
	}
	if string(b) != string(config.ScaffoldTerminalTOML) {
		t.Error("created terminal.toml is not the defaults scaffold verbatim")
	}
	if !layer(t, h.Snapshot(ctx(t)).Config, "terminal-config").Exists {
		t.Error("terminal-config still reported missing after it was created")
	}
}

// Create never clobbers: a layer already on disk is refused with a conflict, its
// bytes untouched, so the button can only ever fill in the file the surface says
// is missing.
func TestCreateRefusesToClobberExistingFile(t *testing.T) {
	h := chartrtest.Start(t)
	register(t, h, chartrtest.NewSpaceRepo(t))
	chartrtest.WriteFile(t, h.ConfigDir, "terminal.toml", "[font]\nsize = 20\n")

	code, body := h.Post("/api/config/create", map[string]string{"layer": "terminal-config"})
	if code != 409 {
		t.Fatalf("create over an existing file = %d, want 409; body %s", code, body)
	}
	b, err := os.ReadFile(filepath.Join(h.ConfigDir, "terminal.toml"))
	if err != nil {
		t.Fatalf("reading terminal.toml: %v", err)
	}
	if !strings.Contains(string(b), "size = 20") {
		t.Errorf("the operator's terminal.toml was overwritten: %s", b)
	}
}

// Create only offers layers with a bundled template, and resolves the name
// server-side exactly as open does — an unknown name, a path, or a layer with no
// template (the agent library, a skill root) is refused rather than stamped.
func TestCreateRefusesLayersWithoutATemplate(t *testing.T) {
	h := chartrtest.Start(t)
	register(t, h, chartrtest.NewSpaceRepo(t))

	for _, bad := range []string{
		"user-config",      // a real layer, but nothing stamps the agent library
		"user-skills",      // likewise a skill root
		"builtin-skills",   //
		"workspace-config", // the retired committed layer
		"/etc/passwd",
		"skill:grill",
		"",
	} {
		code, _ := h.Post("/api/config/create", map[string]string{"layer": bad})
		if code != 400 {
			t.Errorf("create %q = %d, want 400 — only templated layers create", bad, code)
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
