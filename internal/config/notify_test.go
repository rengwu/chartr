package config_test

import (
	"strings"
	"testing"
	"time"

	"github.com/rengwu/chartr/internal/config"
)

func TestNotifyPrefsMissingFileUsesDocumentedDefaults(t *testing.T) {
	prefs, warnings := config.ResolveNotifyPrefs(nil)
	want := config.NotifyPrefs{
		After:   60 * time.Second,
		Settle:  10 * time.Second,
		Enabled: true,
	}
	if prefs != want {
		t.Errorf("missing notify.toml resolved to %+v, want %+v", prefs, want)
	}
	if len(warnings) != 0 {
		t.Errorf("missing notify.toml warned: %v", warnings)
	}
}

func TestNotifyPrefsReadsEveryValue(t *testing.T) {
	prefs, warnings := config.ResolveNotifyPrefs([]byte(`
after = "2m30s"
settle = "4s"
enabled = false
`))
	want := config.NotifyPrefs{
		After:   150 * time.Second,
		Settle:  4 * time.Second,
		Enabled: false,
	}
	if prefs != want {
		t.Errorf("notify prefs = %+v, want %+v", prefs, want)
	}
	if len(warnings) != 0 {
		t.Errorf("valid notify.toml warned: %v", warnings)
	}
}

func TestNotifyPrefsBadValuesDefaultWithOneActionableWarning(t *testing.T) {
	for _, tc := range []struct {
		name string
		file string
		key  string
	}{
		{name: "negative duration", file: `after = "-2s"`, key: "after"},
		{name: "unparseable duration", file: `settle = "later"`, key: "settle"},
		{name: "duration wrong type", file: `after = 60`, key: "after"},
		{name: "enabled wrong type", file: `enabled = "no"`, key: "enabled"},
	} {
		t.Run(tc.name, func(t *testing.T) {
			prefs, warnings := config.ResolveNotifyPrefs([]byte(tc.file))
			if prefs != (config.NotifyPrefs{
				After:   config.DefaultNotifyAfter,
				Settle:  config.DefaultNotifySettle,
				Enabled: true,
			}) {
				t.Errorf("bad %s resolved to %+v, want all defaults", tc.key, prefs)
			}
			if len(warnings) != 1 {
				t.Fatalf("bad %s warnings = %v, want exactly one", tc.key, warnings)
			}
			if !strings.Contains(warnings[0], "notify.toml") || !strings.Contains(warnings[0], tc.key) {
				t.Errorf("warning %q does not name notify.toml and key %q", warnings[0], tc.key)
			}
		})
	}
}

func TestNotifyPrefsKeepsValidValuesBesideBadOnes(t *testing.T) {
	prefs, warnings := config.ResolveNotifyPrefs([]byte(`
after = "90s"
settle = "-1s"
enabled = false
`))
	if prefs.After != 90*time.Second || prefs.Settle != config.DefaultNotifySettle || prefs.Enabled {
		t.Errorf("mixed notify.toml resolved to %+v", prefs)
	}
	if len(warnings) != 1 || !strings.Contains(warnings[0], "settle") {
		t.Errorf("mixed notify.toml warnings = %v, want one naming settle", warnings)
	}
}

func TestNotifyScaffoldIsInertAndExplainsItsOwnership(t *testing.T) {
	prefs, warnings := config.ResolveNotifyPrefs(config.ScaffoldNotifyTOML)
	if prefs != (config.NotifyPrefs{
		After:   config.DefaultNotifyAfter,
		Settle:  config.DefaultNotifySettle,
		Enabled: true,
	}) {
		t.Errorf("notify scaffold resolves to %+v, want defaults", prefs)
	}
	if len(warnings) != 0 {
		t.Errorf("notify scaffold warned: %v", warnings)
	}
	text := string(config.ScaffoldNotifyTOML)
	for _, want := range []string{
		"terminal.toml",
		"user.toml",
		`# after = "60s"`,
		`# settle = "10s"`,
		"# enabled = true",
	} {
		if !strings.Contains(text, want) {
			t.Errorf("notify scaffold does not contain %q:\n%s", want, text)
		}
	}
}
