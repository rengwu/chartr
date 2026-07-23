package server_test

import (
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
	// An unset slot stays empty for the client to fall through to a token default.
	if snap.Terminal.Foreground != "" {
		t.Errorf("snapshot foreground = %q, want empty (unset)", snap.Terminal.Foreground)
	}
}

func TestSnapshotMissingTerminalFileIsAllDefaults(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	chartrtest.WriteMap(t, repo, "widget", mapBody)
	chartrtest.WriteTicket(t, repo, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))

	resp := register(t, h, repo)
	snap := h.Snapshot(ctx(t))

	if (snap.Terminal.FontFamily != "" || snap.Terminal.FontSize != 0 ||
		snap.Terminal.Background != "" || snap.Terminal.Foreground != "") {
		t.Errorf("a machine with no terminal.toml carried %+v, want all defaults", snap.Terminal)
	}
	if s := findSpace(t, snap, resp.ID); len(s.Warnings) != 0 {
		t.Errorf("a missing terminal.toml warned: %v", s.Warnings)
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
