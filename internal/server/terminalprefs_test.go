package server_test

import (
	"path/filepath"
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/chartrtest"
)

// Seam 1, folded with snapshot assembly (spec, Testing Decisions): the parse of
// terminal.toml is tested together with the settings landing on the pushed model
// snapshot. The resolved prefs ride the snapshot globally; a bad value's warning
// surfaces on a space, beside the agent-library warnings.

func TestSnapshotCarriesTerminalPrefs(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	chartrtest.WriteFile(t, h.DataDir, "terminal.toml", `
[font]
family = "IBM Plex Mono"
size = 15

[theme]
background = "#1e2530"

[padding]
left = 12

[keys]
copyOnSelect = true
`)
	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))

	register(t, h, repo)
	snap := h.Snapshot(ctx(t))

	if snap.Terminal.FontFamily != "IBM Plex Mono" {
		t.Errorf("snapshot font family = %q, want the set value", snap.Terminal.FontFamily)
	}
	if snap.Terminal.FontSize != 15 {
		t.Errorf("snapshot font size = %v, want 15", snap.Terminal.FontSize)
	}
	if snap.Terminal.Background != "#1e2530" {
		t.Errorf("snapshot background = %q, want the set value", snap.Terminal.Background)
	}
	// The CSS-only settings and the tri-state key behaviours ride the same snapshot.
	if snap.Terminal.PaddingLeft != 12 {
		t.Errorf("snapshot padding left = %v, want 12", snap.Terminal.PaddingLeft)
	}
	if snap.Terminal.CopyOnSelect == nil || !*snap.Terminal.CopyOnSelect {
		t.Errorf("snapshot copyOnSelect = %v, want the set value", snap.Terminal.CopyOnSelect)
	}
	// An unset slot stays empty for the client to fall through to a token default.
	if snap.Terminal.Foreground != "" {
		t.Errorf("snapshot foreground = %q, want empty (unset)", snap.Terminal.Foreground)
	}
	// An unset tri-state stays nil so the client applies its own default (Shift+Enter
	// is on unless the file says otherwise).
	if snap.Terminal.ShiftEnterNewline != nil {
		t.Errorf("snapshot shiftEnterNewline = %v, want nil (unset)", *snap.Terminal.ShiftEnterNewline)
	}
}

func TestSnapshotMissingTerminalFileFallsBackToBuiltinDefault(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))

	resp := register(t, h, repo)
	snap := h.Snapshot(ctx(t))

	// With no per-machine terminal.toml, the server resolves the built-in default
	// baked into the binary rather than the bare zero-value prefs, so a fresh
	// machine gets the intended default look.
	if snap.Terminal.FontFamily == "" || snap.Terminal.PaddingTop == 0 {
		t.Errorf("a machine with no terminal.toml carried %+v, want the built-in default", snap.Terminal)
	}
	// The built-in default is valid, so it still resolves silently.
	if s := findSpace(t, snap, resp.ID); len(s.Warnings) != 0 {
		t.Errorf("the built-in terminal default warned: %v", s.Warnings)
	}
}

func TestSnapshotTerminalBadValueWarnsOnSpace(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	chartrtest.WriteFile(t, h.DataDir, "terminal.toml", `
[theme]
background = "not-a-colour"
`)
	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))

	resp := register(t, h, repo)
	snap := h.Snapshot(ctx(t))

	if snap.Terminal.Background != "" {
		t.Errorf("a bad colour rode the snapshot as %q, want unset", snap.Terminal.Background)
	}
	s := findSpace(t, snap, resp.ID)
	if !hasSubstring(s.Warnings, "background") {
		t.Errorf("a bad terminal colour did not surface on the space: %v", s.Warnings)
	}
}

// The Settings surface's half of the file (ticket 08): `terminal.toml` is a named
// layer like every other file the operator's config lives in, so the Terminal
// section can show its path and open it in the operator's own editor through the
// space-less open action — read-value-plus-open-file, never a second config store.
func TestTerminalConfigIsAnOpenableGlobalLayer(t *testing.T) {
	h := chartrtest.Start(t)
	chartrtest.WriteFile(t, h.DataDir, "terminal.toml", "[font]\nsize = 15\n")
	// Nudge a rebuild so the freshly written file is on the snapshot.
	register(t, h, chartrtest.NewSpaceRepo(t))

	snap := h.Snapshot(ctx(t))
	l := layer(t, snap.Config, "terminal-config")
	if want := filepath.Join(h.DataDir, "terminal.toml"); l.Path != want {
		t.Errorf("terminal config path = %q, want %q", l.Path, want)
	}
	if l.Holds != "terminal" || !l.Exists {
		t.Errorf("terminal config layer = %+v, want it holding terminal and existing", l)
	}

	record := stubEditor(t)
	code, body := h.Post("/api/config/open", map[string]string{"layer": "terminal-config"})
	if code != 200 {
		t.Fatalf("open terminal-config = %d, body %s", code, body)
	}
	if got := waitForFile(t, record); !strings.Contains(got, l.Path) {
		t.Errorf("the editor was handed %q, want the server-resolved %q", got, l.Path)
	}
}

// A machine with no terminal.toml still lists the layer: the surface says where
// the file *would* go, and a read-shaped action creates nothing.
func TestTerminalConfigLayerListedWhenAbsent(t *testing.T) {
	h := chartrtest.Start(t)
	register(t, h, chartrtest.NewSpaceRepo(t))

	l := layer(t, h.Snapshot(ctx(t)).Config, "terminal-config")
	if l.Exists {
		t.Errorf("terminal config layer = %+v, want it absent", l)
	}
	code, body := h.Post("/api/config/open", map[string]string{"layer": "terminal-config"})
	if code != 200 || !strings.Contains(body, `"exists":false`) {
		t.Errorf("open of an absent terminal.toml = %d %s, want it surfaced as absent", code, body)
	}
}
