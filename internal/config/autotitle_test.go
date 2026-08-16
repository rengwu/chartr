package config

import "testing"

func TestResolveAutoTitleDefaultsOn(t *testing.T) {
	prefs, warnings := ResolveAutoTitlePrefs(nil)
	if !prefs.Enabled {
		t.Fatal("absent config should default to enabled")
	}
	if len(warnings) != 0 {
		t.Fatalf("no warnings expected, got %v", warnings)
	}
}

func TestResolveAutoTitleExplicitOff(t *testing.T) {
	prefs, warnings := ResolveAutoTitlePrefs([]byte("enabled = false\n"))
	if prefs.Enabled {
		t.Fatal("enabled = false should turn it off")
	}
	if len(warnings) != 0 {
		t.Fatalf("no warnings expected, got %v", warnings)
	}
}

func TestResolveAutoTitleWrongTypeWarnsAndDefaults(t *testing.T) {
	prefs, warnings := ResolveAutoTitlePrefs([]byte(`enabled = "yes"` + "\n"))
	if !prefs.Enabled {
		t.Fatal("a wrong type must keep the default on")
	}
	if len(warnings) != 1 {
		t.Fatalf("want one warning, got %v", warnings)
	}
}

func TestResolveAutoTitleUnknownKeyWarns(t *testing.T) {
	_, warnings := ResolveAutoTitlePrefs([]byte("nope = true\n"))
	if len(warnings) != 1 {
		t.Fatalf("want one unknown-key warning, got %v", warnings)
	}
}
