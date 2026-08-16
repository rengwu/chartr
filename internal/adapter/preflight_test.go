package adapter

import (
	"encoding/json"
	"os"
	"path/filepath"
	"testing"
)

// The vectors are kimi's own: each path below is one this host has watched
// kimi trust through its TUI, so a drift between kimiWorkDirKey and kimi's
// encodeWorkDirKey fails here before it fails a launch.
func TestKimiWorkDirKey(t *testing.T) {
	for _, tc := range []struct {
		dir  string
		want string
	}{
		{"/Users/rengwu/Desktop/Projects/chartr", "wd_chartr_328473b6eb1a"},
		{"/Users/rengwu/Desktop/Projects/macdirstat", "wd_macdirstat_3dd750ec86f5"},
		// Trailing slashes never reach the hash.
		{"/Users/rengwu/Desktop/Projects/chartr/", "wd_chartr_328473b6eb1a"},
		{"/Users/rengwu/Desktop/Projects/chartr//", "wd_chartr_328473b6eb1a"},
	} {
		if got := kimiWorkDirKey(tc.dir); got != tc.want {
			t.Errorf("kimiWorkDirKey(%q) = %q, want %q", tc.dir, got, tc.want)
		}
	}
}

func TestKimiSlug(t *testing.T) {
	for _, tc := range []struct {
		name string
		want string
	}{
		{"chartr", "chartr"},
		{"chartr-test-3", "chartr-test-3"},
		{"My Project!", "my-project"},
		{"under_score.dots", "under_score.dots"},
		{"--lead-and-trail--", "lead-and-trail"},
		{"", "workspace"},
		{".", "workspace"},
		{"..", "workspace"},
		{"!!!", "workspace"},
		{"a-very-long-directory-name-that-keeps-going-past-forty", "a-very-long-directory-name-that-keeps-go"},
	} {
		if got := kimiSlug(tc.name); got != tc.want {
			t.Errorf("kimiSlug(%q) = %q, want %q", tc.name, got, tc.want)
		}
	}
}

func TestPreflightUnknownAdapterIsANoop(t *testing.T) {
	if err := Preflight("claude", "/tmp/wherever", nil); err != nil {
		t.Errorf("Preflight(claude) = %v, want nil", err)
	}
}

func TestPreflightKimiWritesTheTrustMarker(t *testing.T) {
	home := t.TempDir()
	dir := "/Users/operator/Projects/My Map"

	if err := Preflight("kimi", dir, []string{"KIMI_CODE_HOME=" + home}); err != nil {
		t.Fatalf("Preflight: %v", err)
	}

	marker := filepath.Join(home, "workspace-trust", kimiWorkDirKey(dir))
	body, err := os.ReadFile(marker)
	if err != nil {
		t.Fatalf("no trust marker at %s: %v", marker, err)
	}
	var got struct {
		Root      string `json:"root"`
		TrustedAt int64  `json:"trustedAt"`
	}
	if err := json.Unmarshal(body, &got); err != nil {
		t.Fatalf("marker is not kimi's document shape: %v", err)
	}
	if got.Root != dir {
		t.Errorf("root = %q, want %q", got.Root, dir)
	}
	if got.TrustedAt == 0 {
		t.Error("trustedAt is zero — kimi trusts on the document's presence, but the timestamp should still be honest")
	}
	if st, _ := os.Stat(marker); st.Mode().Perm() != 0o600 {
		t.Errorf("marker mode = %o, want 600 — the directory kimi keeps it in is private", st.Mode().Perm())
	}
}

func TestPreflightKimiKeepsAnExistingMarker(t *testing.T) {
	home := t.TempDir()
	dir := "/Users/operator/Projects/chartr"

	trustDir := filepath.Join(home, "workspace-trust")
	if err := os.MkdirAll(trustDir, 0o700); err != nil {
		t.Fatal(err)
	}
	marker := filepath.Join(trustDir, kimiWorkDirKey(dir))
	original := []byte(`{"root":"/Users/operator/Projects/chartr","trustedAt":1}`)
	if err := os.WriteFile(marker, original, 0o600); err != nil {
		t.Fatal(err)
	}

	if err := Preflight("kimi", dir, []string{"KIMI_CODE_HOME=" + home}); err != nil {
		t.Fatalf("Preflight: %v", err)
	}
	body, err := os.ReadFile(marker)
	if err != nil {
		t.Fatal(err)
	}
	if string(body) != string(original) {
		t.Errorf("existing marker was rewritten to %s — trust once given keeps its original record", body)
	}
}

func TestPreflightKimiUsesTheDefaultHome(t *testing.T) {
	home := t.TempDir()
	t.Setenv("HOME", home)
	if err := os.Mkdir(filepath.Join(home, ".kimi-code"), 0o700); err != nil {
		t.Fatal(err)
	}

	dir := "/Users/operator/Projects/chartr"
	if err := Preflight("kimi", dir, nil); err != nil {
		t.Fatalf("Preflight: %v", err)
	}
	if _, err := os.Stat(filepath.Join(home, ".kimi-code", "workspace-trust", kimiWorkDirKey(dir))); err != nil {
		t.Errorf("no trust marker under the default home: %v", err)
	}
}

func TestPreflightKimiSkipsAHostWithoutKimi(t *testing.T) {
	home := t.TempDir() // exists, but has no .kimi-code inside
	t.Setenv("HOME", home)

	if err := Preflight("kimi", "/Users/operator/Projects/chartr", nil); err != nil {
		t.Fatalf("Preflight on a kimi-less host = %v, want nil — nothing to prepare", err)
	}
	if _, err := os.Stat(filepath.Join(home, ".kimi-code")); !os.IsNotExist(err) {
		t.Error("preflight created a kimi home — a half-created home is kimi's first run to make, not ours")
	}
}
