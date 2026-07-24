package tracker_test

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/prompt"
	"github.com/rengwu/chartr/internal/tracker"
)

// write drops content at the adapter path under a fresh repo and returns the root.
func write(t *testing.T, content string) string {
	t.Helper()
	repo := t.TempDir()
	p := filepath.Join(repo, filepath.FromSlash(tracker.RelPath))
	if err := os.MkdirAll(filepath.Dir(p), 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(p, []byte(content), 0o644); err != nil {
		t.Fatal(err)
	}
	return repo
}

// The shipped template must carry the marker, or classification of chartr's own
// file silently degrades to Foreign. This guards the one coupling between the
// template (in prompt's assets) and this package's Marker.
func TestShippedTemplateCarriesMarker(t *testing.T) {
	if got := prompt.TrackerAdapter(); !strings.Contains(got, tracker.Marker) {
		t.Fatalf("shipped tracker adapter is missing marker %q", tracker.Marker)
	}
}

func TestClassifyAbsent(t *testing.T) {
	res, err := tracker.Classify(t.TempDir(), prompt.TrackerAdapter())
	if err != nil {
		t.Fatal(err)
	}
	if res.State != tracker.Absent {
		t.Fatalf("state = %v, want absent", res.State)
	}
	if res.Path == "" {
		t.Fatal("path should be set even when the file is absent")
	}
}

func TestClassifyUpToDate(t *testing.T) {
	tmpl := prompt.TrackerAdapter()
	res, err := tracker.Classify(write(t, tmpl), tmpl)
	if err != nil {
		t.Fatal(err)
	}
	if res.State != tracker.UpToDate {
		t.Fatalf("state = %v, want up-to-date", res.State)
	}
}

func TestClassifyStale(t *testing.T) {
	// chartr's marker present, but the body drifted (an operator edit or an old
	// version) — a refresh candidate, not a foreign file.
	edited := prompt.TrackerAdapter() + "\n<!-- an operator note -->\n"
	res, err := tracker.Classify(write(t, edited), prompt.TrackerAdapter())
	if err != nil {
		t.Fatal(err)
	}
	if res.State != tracker.Stale {
		t.Fatalf("state = %v, want stale", res.State)
	}
}

func TestClassifyForeignWithRemoteHint(t *testing.T) {
	cases := map[string]struct {
		content string
		hint    string
	}{
		"github":       {"# Issue tracker\n\nRun `gh issue create` for new issues.", "gh"},
		"gitlab":       {"# Tracker\n\nUse glab to open issues.", "glab"},
		"linear":       {"# Tracker\n\nIssues live in Linear (linear.app/acme).", "linear"},
		"local-nohint": {"# Tracker\n\nIssues are markdown under .scratch/<feature>/.", ""},
	}
	for name, tc := range cases {
		t.Run(name, func(t *testing.T) {
			res, err := tracker.Classify(write(t, tc.content), prompt.TrackerAdapter())
			if err != nil {
				t.Fatal(err)
			}
			if res.State != tracker.Foreign {
				t.Fatalf("state = %v, want foreign", res.State)
			}
			if res.RemoteHint != tc.hint {
				t.Fatalf("hint = %q, want %q", res.RemoteHint, tc.hint)
			}
		})
	}
}

func TestInstallCreatesDirAndRoundTrips(t *testing.T) {
	repo := t.TempDir()
	tmpl := prompt.TrackerAdapter()

	p, err := tracker.Install(repo, tmpl)
	if err != nil {
		t.Fatal(err)
	}
	if want := filepath.Join(repo, filepath.FromSlash(tracker.RelPath)); p != want {
		t.Fatalf("install path = %q, want %q", p, want)
	}

	// A freshly installed adapter classifies as up-to-date on the next pass.
	res, err := tracker.Classify(repo, tmpl)
	if err != nil {
		t.Fatal(err)
	}
	if res.State != tracker.UpToDate {
		t.Fatalf("after install, state = %v, want up-to-date", res.State)
	}
}

func TestInstallOverwritesForeign(t *testing.T) {
	// Install is the write for both refresh and consented replace; it overwrites
	// whatever is there. (Consent is the caller's gate, not Install's.)
	repo := write(t, "# someone else's tracker\n")
	tmpl := prompt.TrackerAdapter()
	if _, err := tracker.Install(repo, tmpl); err != nil {
		t.Fatal(err)
	}
	res, err := tracker.Classify(repo, tmpl)
	if err != nil {
		t.Fatal(err)
	}
	if res.State != tracker.UpToDate {
		t.Fatalf("state = %v, want up-to-date", res.State)
	}
}
