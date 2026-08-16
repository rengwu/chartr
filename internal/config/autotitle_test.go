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

func TestResolveAutoTitleNativeOnlyDefaultsOff(t *testing.T) {
	prefs, warnings := ResolveAutoTitlePrefs([]byte("enabled = true\n"))
	if prefs.NativeOnly {
		t.Fatal("an absent native_only should default to off")
	}
	if len(warnings) != 0 {
		t.Fatalf("no warnings expected, got %v", warnings)
	}
}

func TestResolveAutoTitleNativeOnlyExplicitOn(t *testing.T) {
	prefs, warnings := ResolveAutoTitlePrefs([]byte("native_only = true\n"))
	if !prefs.NativeOnly {
		t.Fatal("native_only = true should turn it on")
	}
	if !prefs.Enabled {
		t.Fatal("native_only says nothing about enabled, which stays on by default")
	}
	if len(warnings) != 0 {
		t.Fatalf("no warnings expected, got %v", warnings)
	}
}

func TestResolveAutoTitleNativeOnlyWrongTypeWarnsAndDefaults(t *testing.T) {
	prefs, warnings := ResolveAutoTitlePrefs([]byte(`native_only = "yes"` + "\n"))
	if prefs.NativeOnly {
		t.Fatal("a wrong type must keep the default off")
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
