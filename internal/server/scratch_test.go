package server_test

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/chartrtest"
	"github.com/rengwu/chartr/internal/model"
	"github.com/rengwu/chartr/internal/registry"
)

// Scratch is present in the server-authoritative model from first run, but its
// derived shape is deliberately only an identity, a home-directory path, and
// its live shells. Keeping every repository-derived field empty is the public
// proof that a rebuild did not discover, classify, or run git against $HOME.
func TestSnapshotAlwaysCarriesAThinScratchSpace(t *testing.T) {
	h := chartrtest.Start(t)
	home, err := os.UserHomeDir()
	if err != nil {
		t.Fatalf("resolve home directory: %v", err)
	}

	snap := h.Snapshot(ctx(t))
	if len(snap.Spaces) != 1 {
		t.Fatalf("first-run snapshot has %d spaces, want only Scratch", len(snap.Spaces))
	}
	scratch := findSpace(t, snap, registry.ScratchID)
	if !scratch.Scratch {
		t.Error("Scratch space is not flagged")
	}
	if scratch.Name != "Scratch" {
		t.Errorf("Scratch name = %q, want Scratch", scratch.Name)
	}
	if scratch.Path != home {
		t.Errorf("Scratch path = %q, want home directory %q", scratch.Path, home)
	}
	if scratch.Branch != "" || scratch.Dirty || scratch.LastAgent != "" {
		t.Errorf("Scratch carries repository state: branch=%q dirty=%v lastAgent=%q",
			scratch.Branch, scratch.Dirty, scratch.LastAgent)
	}
	if len(scratch.Maps) != 0 || len(scratch.Skills) != 0 || len(scratch.Layers) != 0 {
		t.Errorf("Scratch is not thin: maps=%d skills=%d layers=%d",
			len(scratch.Maps), len(scratch.Skills), len(scratch.Layers))
	}
	if len(scratch.Terminals) != 0 {
		t.Errorf("fresh Scratch carries %d terminals, want none", len(scratch.Terminals))
	}

	if _, err := os.Stat(filepath.Join(h.ConfigDir, "spaces.toml")); !os.IsNotExist(err) {
		t.Errorf("first run wrote a registry row for Scratch: stat error = %v", err)
	}
}

// Opening through the ordinary terminal endpoint proves Scratch is an ordinary
// registry lookup to the terminal manager: the shell starts in its entry path,
// the operator's home, and multiple shells group beneath the same space.
func TestScratchShellsOpenInTheHomeDirectory(t *testing.T) {
	fakeHome := t.TempDir()
	t.Setenv("HOME", fakeHome)
	h := chartrtest.Start(t)
	home, err := os.UserHomeDir()
	if err != nil {
		t.Fatalf("resolve home directory: %v", err)
	}

	first := h.OpenTerminal(registry.ScratchID)
	tc := h.DialTerminal(ctx(t), first)
	defer tc.Close()
	tc.Send(ctx(t), "printf 'scratch-cwd<%s>\\n' \"$PWD\"\n")
	// macOS exposes /var as a symlink to /private/var; the shell reports the
	// physical spelling after chdir even though UserHomeDir returned the logical
	// one. They name the same directory.
	shellHome, err := filepath.EvalSymlinks(home)
	if err != nil {
		t.Fatalf("resolve home-directory symlinks: %v", err)
	}
	wantCWD := "scratch-cwd<" + shellHome + ">"
	out := tc.ReadUntil(ctx(t), wantCWD)
	if !strings.Contains(out, wantCWD) {
		t.Fatalf("Scratch shell did not report home as its working directory; output %q", out)
	}

	second := h.OpenTerminal(registry.ScratchID)
	scratch := findSpace(t, h.Snapshot(ctx(t)), registry.ScratchID)
	if !hasTerminal(scratch, first) || !hasTerminal(scratch, second) {
		t.Fatalf("Scratch terminals = %+v, want both %s and %s", scratch.Terminals, first, second)
	}
	if got := len(scratch.Terminals); got != 2 {
		t.Errorf("Scratch carries %d terminals, want 2", got)
	}

	// Opening shells mutates runtime state only: it neither registers $HOME nor
	// runs the registration path that could initialise a repository there.
	if _, err := os.Stat(filepath.Join(h.ConfigDir, "spaces.toml")); !os.IsNotExist(err) {
		t.Errorf("opening Scratch wrote spaces.toml: stat error = %v", err)
	}
	if _, err := os.Stat(filepath.Join(fakeHome, ".git")); !os.IsNotExist(err) {
		t.Errorf("opening Scratch ran git init in the home directory: stat error = %v", err)
	}
}

// A hidden Scratch row is absent from the list the chrome can post. The server
// puts it back at its current slot, while retaining the full-list validation for
// every registered space.
func TestReorderMayOmitOnlyScratch(t *testing.T) {
	h := chartrtest.Start(t)
	a := register(t, h, chartrtest.NewSpaceRepo(t))
	b := register(t, h, chartrtest.NewSpaceRepo(t))

	withScratch := []string{a.ID, registry.ScratchID, b.ID}
	if code, body := h.Post("/api/spaces/reorder", map[string][]string{"ids": withScratch}); code != 204 {
		t.Fatalf("seat Scratch between registered spaces = %d, body %s", code, body)
	}

	visibleOrder := []string{b.ID, a.ID}
	if code, body := h.Post("/api/spaces/reorder", map[string][]string{"ids": visibleOrder}); code != 204 {
		t.Fatalf("reorder omitting Scratch = %d, body %s", code, body)
	}
	want := []string{b.ID, registry.ScratchID, a.ID}
	if got := allSpaceIDs(h.Snapshot(ctx(t))); !equalStrings(got, want) {
		t.Fatalf("sidebar after hidden-Scratch reorder = %v, want %v", got, want)
	}

	if code, body := h.Post("/api/spaces/reorder", map[string][]string{"ids": []string{b.ID}}); code != 400 {
		t.Errorf("reorder omitting a registered space = %d, want 400 (body %s)", code, body)
	}
	if got := allSpaceIDs(h.Snapshot(ctx(t))); !equalStrings(got, want) {
		t.Errorf("sidebar after refused registered-space omission = %v, want untouched %v", got, want)
	}

	data, err := os.ReadFile(filepath.Join(h.ConfigDir, "spaces.toml"))
	if err != nil {
		t.Fatalf("read registry after reorder: %v", err)
	}
	if got := strings.Count(string(data), "[[space]]"); got != 2 {
		t.Errorf("registry carries %d space rows, want only the 2 registered spaces:\n%s", got, data)
	}
	home, err := os.UserHomeDir()
	if err != nil {
		t.Fatalf("resolve home directory: %v", err)
	}
	if strings.Contains(string(data), "path = \""+home+"\"") {
		t.Errorf("registry persisted Scratch as a space row:\n%s", data)
	}
}

// The slot is stored, not merely remembered for as long as this process lives.
// A second chartr over the same config root is the only thing that can tell the
// two apart, so the arrangement is asserted across one: the operator moves
// Scratch, restarts, and finds the sidebar exactly as they left it.
func TestScratchSlotSurvivesARestart(t *testing.T) {
	configDir := t.TempDir()
	repoA, repoB := chartrtest.NewSpaceRepo(t), chartrtest.NewSpaceRepo(t)

	first := chartrtest.Start(t, chartrtest.WithConfigDir(configDir))
	a := register(t, first, repoA)
	b := register(t, first, repoB)

	want := []string{b.ID, registry.ScratchID, a.ID}
	if code, body := first.Post("/api/spaces/reorder", map[string][]string{"ids": want}); code != 204 {
		t.Fatalf("seat Scratch between registered spaces = %d, body %s", code, body)
	}
	if got := allSpaceIDs(first.Snapshot(ctx(t))); !equalStrings(got, want) {
		t.Fatalf("sidebar after the move = %v, want %v", got, want)
	}

	second := chartrtest.Start(t, chartrtest.WithConfigDir(configDir))
	if got := allSpaceIDs(second.Snapshot(ctx(t))); !equalStrings(got, want) {
		t.Errorf("sidebar after a restart = %v, want the stored %v", got, want)
	}
}

// Refusal is a server rule, not merely an absent button: the chrome offers none
// of these on Scratch, but a stale or hand-written client must not be able to
// reach past that and act on the operator's home directory. Every endpoint that
// resolves a space before it acts answers a bad request, and the isolated home
// is still empty afterwards — the public proof that nothing was written, charted
// or initialised there.
func TestRepoScopedEndpointsRefuseScratch(t *testing.T) {
	fakeHome := t.TempDir()
	t.Setenv("HOME", fakeHome)
	chartrtest.StubAgent(t, "some-harness")
	h := chartrtest.Start(t)
	// A real, present agent, so the launch endpoints clear their own doorstep and
	// the Scratch identity is the only thing left that can refuse them.
	registerAgent(t, h, "stub", map[string]any{"adapter": "some-harness"})
	id := registry.ScratchID

	refusals := []struct {
		what string
		call func() (int, string)
	}{
		{"spawn a session", func() (int, string) {
			return h.SpawnWithAgent(id, "some-map", 1, "implement", "stub")
		}},
		{"launch an on-ramp skill", func() (int, string) {
			return h.LaunchRaw(id, "stub", "ideate", "")
		}},
		{"ideate", func() (int, string) { return h.IdeateRaw(id, "stub") }},
		{"preview a payload", func() (int, string) {
			return h.Get("/api/spaces/" + id + "/maps/some-map/tickets/1/payload?role=implement")
		}},
		{"resume a dead session", func() (int, string) { return h.Resume(id, "sess") }},
		{"respawn a dead session", func() (int, string) { return h.Respawn(id, "sess") }},
		{"release a dead session", func() (int, string) { return h.Release(id, "sess") }},
		{"release a ticket", func() (int, string) { return h.ReleaseTicket(id, "some-map", 1) }},
		{"open a config layer", func() (int, string) {
			return h.Post("/api/spaces/"+id+"/config/open", map[string]string{"layer": "user-config"})
		}},
		{"set the remembered agent", func() (int, string) {
			return h.Put("/api/spaces/"+id+"/agent", map[string]string{"agent": "stub"})
		}},
		{"deregister it", func() (int, string) { return h.Delete("/api/spaces/" + id) }},
	}
	for _, r := range refusals {
		code, body := r.call()
		if code != 400 {
			t.Errorf("%s = %d, want 400 (body %s)", r.what, code, body)
			continue
		}
		// The message has to name the reason: a 400 these endpoints would have
		// given anyway — a malformed body, an unregistered agent — is not the
		// refusal this asserts.
		if !strings.Contains(body, "Scratch") {
			t.Errorf("%s refused with %s, want a refusal naming Scratch", r.what, body)
		}
	}

	// Deregistering is refused rather than silently ignored, and the entry it
	// names is untouched: a no-op on an explicit destructive request is worse
	// than a clear answer.
	snap := h.Snapshot(ctx(t))
	scratch := findSpace(t, snap, id)
	if !scratch.Scratch {
		t.Error("Scratch left the snapshot after a refused deregistration")
	}

	// Nothing reached the home directory: no repository, no `.plan/`, no adapter,
	// no payload — the whole point of refusing at the server rather than in the
	// chrome.
	entries, err := os.ReadDir(fakeHome)
	if err != nil {
		t.Fatalf("read the isolated home: %v", err)
	}
	if len(entries) != 0 {
		names := make([]string, 0, len(entries))
		for _, e := range entries {
			names = append(names, e.Name())
		}
		t.Errorf("refused actions wrote into the home directory: %v", names)
	}
}

// The three terminal endpoints are the whole of what Scratch is for, so they
// take it like any other space: open a shell, acknowledge it, end it.
func TestTerminalEndpointsAcceptScratch(t *testing.T) {
	fakeHome := t.TempDir()
	t.Setenv("HOME", fakeHome)
	h := chartrtest.Start(t)
	id := registry.ScratchID

	termID := h.OpenTerminal(id)
	if code, body := h.Post("/api/spaces/"+id+"/terminals/"+termID+"/seen", nil); code != 204 {
		t.Errorf("acknowledge a Scratch shell = %d, want 204 (body %s)", code, body)
	}
	if code, body := h.Delete("/api/spaces/" + id + "/terminals/" + termID); code != 204 {
		t.Errorf("end a Scratch shell = %d, want 204 (body %s)", code, body)
	}
}

func allSpaceIDs(m model.Model) []string {
	out := make([]string, 0, len(m.Spaces))
	for _, s := range m.Spaces {
		out = append(out, s.ID)
	}
	return out
}
