package server_test

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/chartrtest"
	"github.com/rengwu/chartr/internal/prompt"
	"github.com/rengwu/chartr/internal/tracker"
)

// The tracker-adapter offer at the process boundary: it rides each space's
// snapshot only when there is something to act on, installing writes chartr's
// adapter and clears the offer, a foreign file is never touched until an explicit
// install replaces it, and dismissing silences the prompt for good. Assertions are
// on the snapshot, the HTTP responses, and the filesystem — never internals.

func adapterPath(repo string) string {
	return filepath.Join(repo, filepath.FromSlash(tracker.RelPath))
}

// A repo with no adapter offers to install one; installing writes the template
// and the offer clears itself on the next snapshot.
func TestTrackerAdapterAbsentThenInstall(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	resp := register(t, h, repo)

	offer := findSpace(t, h.Snapshot(ctx(t)), resp.ID).TrackerAdapter
	if offer == nil || offer.State != "absent" {
		t.Fatalf("offer = %+v, want state absent", offer)
	}

	if code, body := h.Post("/api/spaces/"+resp.ID+"/tracker-adapter", nil); code != 200 {
		t.Fatalf("install = %d, body %s", code, body)
	}

	// The template landed on disk, marker and all.
	b, err := os.ReadFile(adapterPath(repo))
	if err != nil {
		t.Fatalf("adapter not written: %v", err)
	}
	if string(b) != prompt.TrackerAdapter() {
		t.Fatal("written adapter does not match the shipped template")
	}

	// And the offer is gone — the adapter now reads up-to-date.
	if offer := findSpace(t, h.Snapshot(ctx(t)), resp.ID).TrackerAdapter; offer != nil {
		t.Fatalf("offer still present after install: %+v", offer)
	}
}

// A foreign issue-tracker.md is surfaced with its state and best-effort hint, and
// chartr does not touch it — until an explicit install replaces it.
func TestTrackerAdapterForeignNotClobberedThenReplace(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	const foreign = "# Issue tracker\n\nIssues live on GitHub — run `gh issue create`.\n"
	chartrtest.WriteFile(t, repo, tracker.RelPath, foreign)
	resp := register(t, h, repo)

	offer := findSpace(t, h.Snapshot(ctx(t)), resp.ID).TrackerAdapter
	if offer == nil || offer.State != "foreign" || offer.RemoteHint != "gh" {
		t.Fatalf("offer = %+v, want foreign with gh hint", offer)
	}

	// The foreign file is untouched by mere classification.
	if b, _ := os.ReadFile(adapterPath(repo)); string(b) != foreign {
		t.Fatal("foreign adapter was modified without an explicit install")
	}

	// An explicit install is the consented replace.
	if code, body := h.Post("/api/spaces/"+resp.ID+"/tracker-adapter", nil); code != 200 {
		t.Fatalf("replace = %d, body %s", code, body)
	}
	if b, _ := os.ReadFile(adapterPath(repo)); !strings.Contains(string(b), tracker.Marker) {
		t.Fatal("after replace, adapter is not chartr's")
	}
	if offer := findSpace(t, h.Snapshot(ctx(t)), resp.ID).TrackerAdapter; offer != nil {
		t.Fatalf("offer still present after replace: %+v", offer)
	}
}

// Dismissing the offer silences it for good and writes nothing to the repo.
func TestTrackerAdapterDismissSticks(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	resp := register(t, h, repo)

	if findSpace(t, h.Snapshot(ctx(t)), resp.ID).TrackerAdapter == nil {
		t.Fatal("expected an offer before dismissal")
	}
	if code, body := h.Post("/api/spaces/"+resp.ID+"/tracker-adapter/dismiss", nil); code != 204 {
		t.Fatalf("dismiss = %d, body %s", code, body)
	}
	if offer := findSpace(t, h.Snapshot(ctx(t)), resp.ID).TrackerAdapter; offer != nil {
		t.Fatalf("offer still present after dismiss: %+v", offer)
	}
	// Nothing was written to the repo.
	if _, err := os.Stat(adapterPath(repo)); !os.IsNotExist(err) {
		t.Fatal("dismiss must not write the adapter")
	}
}

// An up-to-date adapter makes no offer at all.
func TestTrackerAdapterUpToDateNoOffer(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	chartrtest.WriteFile(t, repo, tracker.RelPath, prompt.TrackerAdapter())
	resp := register(t, h, repo)

	if offer := findSpace(t, h.Snapshot(ctx(t)), resp.ID).TrackerAdapter; offer != nil {
		t.Fatalf("up-to-date adapter should make no offer, got %+v", offer)
	}
}
