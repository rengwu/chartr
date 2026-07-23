package config_test

import (
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/config"
)

// Seam 1 — the pure parse of terminal.toml into a resolved TerminalPrefs value
// plus human-readable warnings. Every assertion is on what ResolveTerminalPrefs
// makes of the file's bytes: a valid value lands, an invalid one keeps the
// default and warns, a missing file is all defaults and silent. The client resolve
// seam (tokens.ts) turns these prefs into concrete xterm options; here we only
// prove the file becomes the value.

func TestTerminalPrefsMissingFileIsAllDefaults(t *testing.T) {
	prefs, warnings := config.ResolveTerminalPrefs(nil)
	if prefs != (config.TerminalPrefs{}) {
		t.Errorf("a missing file resolved to %+v, want the zero (all-default) prefs", prefs)
	}
	if len(warnings) != 0 {
		t.Errorf("a missing file warned: %v", warnings)
	}
}

func TestTerminalPrefsReadsFontAndColours(t *testing.T) {
	prefs, warnings := config.ResolveTerminalPrefs([]byte(`
[font]
family = "IBM Plex Mono"
size = 15

[theme]
background = "#1e2530"
foreground = "#e6e6e6"
`))
	if len(warnings) != 0 {
		t.Fatalf("a valid file warned: %v", warnings)
	}
	if prefs.FontFamily != "IBM Plex Mono" {
		t.Errorf("font family = %q, want the set value", prefs.FontFamily)
	}
	if prefs.FontSize != 15 {
		t.Errorf("font size = %v, want 15", prefs.FontSize)
	}
	if prefs.Background != "#1e2530" || prefs.Foreground != "#e6e6e6" {
		t.Errorf("colours = %q/%q, want the set values", prefs.Background, prefs.Foreground)
	}
}

func TestTerminalPrefsBadValueKeepsDefaultAndWarns(t *testing.T) {
	// A negative size and a colour that is not a colour: each falls back to its
	// unset default and the operator is told what was ignored.
	prefs, warnings := config.ResolveTerminalPrefs([]byte(`
[font]
size = -3

[theme]
background = "not-a-colour"
`))
	if prefs.FontSize != 0 {
		t.Errorf("a negative size resolved to %v, want the default (unset)", prefs.FontSize)
	}
	if prefs.Background != "" {
		t.Errorf("an invalid colour resolved to %q, want the default (unset)", prefs.Background)
	}
	if !hasSub(warnings, "size") {
		t.Errorf("no warning named the bad font size: %v", warnings)
	}
	if !hasSub(warnings, "background") {
		t.Errorf("no warning named the bad colour: %v", warnings)
	}
}

func TestTerminalPrefsUnknownKeyWarns(t *testing.T) {
	prefs, warnings := config.ResolveTerminalPrefs([]byte(`
[font]
family = "IBM Plex Mono"
weight = "bold"
`))
	if prefs.FontFamily != "IBM Plex Mono" {
		t.Errorf("a known key beside an unknown one was dropped: %+v", prefs)
	}
	if !hasSub(warnings, "weight") {
		t.Errorf("an unknown key produced no warning: %v", warnings)
	}
}

func TestTerminalPrefsMalformedFileWarnsAndDefaults(t *testing.T) {
	prefs, warnings := config.ResolveTerminalPrefs([]byte("this is not = = toml"))
	if prefs != (config.TerminalPrefs{}) {
		t.Errorf("a malformed file resolved to %+v, want the zero prefs", prefs)
	}
	if len(warnings) == 0 {
		t.Errorf("a malformed file produced no warning")
	}
}

func hasSub(warnings []string, sub string) bool {
	for _, w := range warnings {
		if strings.Contains(w, sub) {
			return true
		}
	}
	return false
}
