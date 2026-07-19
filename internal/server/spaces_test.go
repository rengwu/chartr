package server_test

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"testing"
	"time"

	"github.com/rengwu/wayfinder-harness/internal/harnesstest"
	"github.com/rengwu/wayfinder-harness/internal/model"
)

// Ticket 02 at the process boundary: the registry (register with an announced
// git init, forget-not-destroy removal, a rebuildable index) and role bindings
// (three-layer field-level merge resolving user-over-workspace, a committed
// autopilot flag ignored with a warning, an absent adapter surfaced as a
// badge). Every assertion is on what the design makes public — HTTP responses,
// control-socket snapshots, the filesystem, and git — never on internals.

func ctx(t *testing.T) context.Context {
	t.Helper()
	c, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	t.Cleanup(cancel)
	return c
}

type registerResp struct {
	ID        string `json:"id"`
	Path      string `json:"path"`
	GitInited bool   `json:"gitInited"`
}

func register(t *testing.T, h *harnesstest.Harness, path string) registerResp {
	t.Helper()
	code, body := h.Post("/api/spaces", map[string]string{"path": path})
	if code != 200 {
		t.Fatalf("register %s = %d, body %s", path, code, body)
	}
	var r registerResp
	if err := json.Unmarshal([]byte(body), &r); err != nil {
		t.Fatalf("register response not JSON: %v (%q)", err, body)
	}
	return r
}

func findSpace(t *testing.T, m model.Model, id string) model.Space {
	t.Helper()
	for _, s := range m.Spaces {
		if s.ID == id {
			return s
		}
	}
	t.Fatalf("space %s not in snapshot (%d spaces)", id, len(m.Spaces))
	return model.Space{}
}

func binding(t *testing.T, s model.Space, role string) model.RoleBinding {
	t.Helper()
	for _, b := range s.Bindings {
		if b.Role == role {
			return b
		}
	}
	t.Fatalf("role %q not in space %s bindings", role, s.Name)
	return model.RoleBinding{}
}

// Registering a plain folder makes it a space and, because it was not yet a git
// repository, initialises one — reported in the action's response, never silent
// (story 2). An already-registered repo is not re-initialised.
func TestRegisterInitialisesNonRepoAnnounced(t *testing.T) {
	h := harnesstest.Start(t)

	plain := t.TempDir() // a folder, not a repo
	if _, err := os.Stat(filepath.Join(plain, ".git")); !os.IsNotExist(err) {
		t.Fatalf("precondition: %s already looks like a repo", plain)
	}

	resp := register(t, h, plain)
	if !resp.GitInited {
		t.Error("gitInited = false, want the announced git init for a non-repo folder")
	}
	if _, err := os.Stat(filepath.Join(plain, ".git")); err != nil {
		t.Errorf("no .git after registering a non-repo folder: %v", err)
	}

	snap := h.Snapshot(ctx(t))
	s := findSpace(t, snap, resp.ID)
	if s.Path != plain {
		t.Errorf("space path = %q, want %q", s.Path, plain)
	}
	if s.Name != filepath.Base(plain) {
		t.Errorf("space name = %q, want %q", s.Name, filepath.Base(plain))
	}

	// A second registration of an existing repo does not re-init.
	repo := harnesstest.NewSpaceRepo(t)
	resp2 := register(t, h, repo)
	if resp2.GitInited {
		t.Error("gitInited = true for an existing repo, want false")
	}
}

// De-registering forgets the entry and touches nothing in the repository — not
// git, not the working tree, not committed config (story 4). Registering must
// likewise write nothing into the repo: the registry lives in user config.
func TestForgetNotDestroy(t *testing.T) {
	h := harnesstest.Start(t)
	repo := harnesstest.NewSpaceRepo(t)

	harnesstest.WriteFile(t, repo, "README.md", "hello\n")
	harnesstest.Git(t, repo, "add", "-A")
	harnesstest.Git(t, repo, "commit", "-qm", "seed")
	harnesstest.WriteFile(t, repo, "dirty.txt", "uncommitted work\n")

	head := harnesstest.Git(t, repo, "rev-parse", "HEAD")
	status := harnesstest.Git(t, repo, "status", "--porcelain")
	files := worktreeFiles(t, repo)

	resp := register(t, h, repo)

	// Registering wrote nothing into the repo.
	if got := worktreeFiles(t, repo); !equalStrings(got, files) {
		t.Errorf("register changed the repo tree:\n before %v\n after  %v", files, got)
	}
	if _, err := os.Stat(filepath.Join(repo, ".wayfinder-harness.toml")); !os.IsNotExist(err) {
		t.Error("register wrote a committed config file into the repo; it must not")
	}

	// Forget it.
	if code, body := h.Delete("/api/spaces/" + resp.ID); code != 204 {
		t.Fatalf("deregister = %d, body %s", code, body)
	}

	// Nothing in the repository moved.
	if got := harnesstest.Git(t, repo, "rev-parse", "HEAD"); got != head {
		t.Errorf("HEAD changed across register/forget: %s -> %s", head, got)
	}
	if got := harnesstest.Git(t, repo, "status", "--porcelain"); got != status {
		t.Errorf("git status changed across register/forget:\n before %q\n after  %q", status, got)
	}
	if got, want := readFile(t, filepath.Join(repo, "dirty.txt")), "uncommitted work\n"; got != want {
		t.Errorf("dirty file changed: %q", got)
	}
	if got := worktreeFiles(t, repo); !equalStrings(got, files) {
		t.Errorf("forget changed the repo tree:\n before %v\n after  %v", files, got)
	}

	// The space is gone from the snapshot.
	for _, s := range h.Snapshot(ctx(t)).Spaces {
		if s.ID == resp.ID {
			t.Error("forgotten space still present in snapshot")
		}
	}
}

// The registry is a rebuildable index: deleting it costs re-adding folders,
// never work. A harness started against a data dir whose registry.toml is gone
// shows no spaces, and re-registering the untouched repo restores it.
func TestRegistryLossIsRebuildable(t *testing.T) {
	dataDir := t.TempDir()
	repo := harnesstest.NewSpaceRepo(t)
	harnesstest.WriteFile(t, repo, "keep.txt", "authoritative work lives in the repo\n")
	harnesstest.Git(t, repo, "add", "-A")
	harnesstest.Git(t, repo, "commit", "-qm", "work")
	head := harnesstest.Git(t, repo, "rev-parse", "HEAD")

	first := harnesstest.Start(t, harnesstest.WithDataDir(dataDir))
	resp := register(t, first, repo)
	if len(first.Snapshot(ctx(t)).Spaces) != 1 {
		t.Fatal("space not registered on the first harness")
	}

	// Lose the registry, then bring a fresh harness up on the same data dir.
	if err := os.Remove(filepath.Join(dataDir, "registry.toml")); err != nil {
		t.Fatalf("removing registry: %v", err)
	}
	second := harnesstest.Start(t, harnesstest.WithDataDir(dataDir))
	if got := len(second.Snapshot(ctx(t)).Spaces); got != 0 {
		t.Fatalf("after registry loss, snapshot has %d spaces, want 0", got)
	}

	// The repo — the authoritative state — is untouched, so re-adding restores.
	if got := harnesstest.Git(t, repo, "rev-parse", "HEAD"); got != head {
		t.Errorf("repo HEAD changed across registry loss: %s -> %s", head, got)
	}
	resp2 := register(t, second, repo)
	if resp2.ID != resp.ID {
		t.Errorf("re-registered id = %s, want the same stable id %s", resp2.ID, resp.ID)
	}
	if len(second.Snapshot(ctx(t)).Spaces) != 1 {
		t.Error("re-registering did not restore the space")
	}
}

// Role bindings merge three layers — built-in ‹ workspace ‹ user — field by
// field, resolving user-over-workspace (ADR 0009). A user override of one field
// inherits the rest, and the effective binding records where each field came
// from so the inheritance is visible (story 39).
func TestBindingMergeMatrix(t *testing.T) {
	h := harnesstest.Start(t)
	repo := harnesstest.NewSpaceRepo(t)

	// Committed workspace config: full bindings for two roles.
	harnesstest.WriteFile(t, repo, ".wayfinder-harness.toml", `
[roles.implement]
adapter = "claude"
model = "sonnet-ws"

[roles.review]
adapter = "codex"
model = "gpt-ws"
`)

	// Local user config, keyed by space path: override just implement.model and
	// just review.adapter — each inheriting the other field from workspace.
	harnesstest.WriteFile(t, h.DataDir, "user.toml", fmt.Sprintf(`
[spaces.%q.roles.implement]
model = "sonnet-user"

[spaces.%q.roles.review]
adapter = "opencode"
`, repo, repo))

	resp := register(t, h, repo)
	s := findSpace(t, h.Snapshot(ctx(t)), resp.ID)

	// implement: adapter inherited from workspace, model won by the user layer.
	impl := binding(t, s, "implement")
	assertField(t, "implement.adapter", impl.Adapter, "claude", impl.AdapterFrom, "workspace")
	assertField(t, "implement.model", impl.Model, "sonnet-user", impl.ModelFrom, "user")

	// review: model inherited from workspace, adapter won by the user layer.
	rev := binding(t, s, "review")
	assertField(t, "review.adapter", rev.Adapter, "opencode", rev.AdapterFrom, "user")
	assertField(t, "review.model", rev.Model, "gpt-ws", rev.ModelFrom, "workspace")

	// An untouched role falls through to the shipped built-in default.
	grill := binding(t, s, "grill")
	if grill.AdapterFrom != "built-in" || grill.ModelFrom != "built-in" {
		t.Errorf("grill resolved from %s/%s, want built-in/built-in", grill.AdapterFrom, grill.ModelFrom)
	}
	if grill.Adapter == "" || grill.Model == "" {
		t.Errorf("grill built-in binding is empty: %+v", grill)
	}
}

// A committed autopilot flag is ignored with a warning — committing "no human
// reviews this code" for everyone who clones is exactly what "cockpit, not
// autopilot" refuses; autopilot is strictly a local choice (ADR 0009).
func TestCommittedAutopilotIgnoredWithWarning(t *testing.T) {
	h := harnesstest.Start(t)
	repo := harnesstest.NewSpaceRepo(t)
	harnesstest.WriteFile(t, repo, ".wayfinder-harness.toml", "autopilot = true\n")

	resp := register(t, h, repo)
	s := findSpace(t, h.Snapshot(ctx(t)), resp.ID)

	if !hasSubstring(s.Warnings, "autopilot") {
		t.Errorf("committed autopilot produced no warning; warnings = %v", s.Warnings)
	}
}

// A binding whose adapter is not on PATH surfaces as a badge naming the binding
// and the fix, without failing anything (story 40); a binding whose adapter is
// present resolves clean.
func TestAdapterPresenceBadge(t *testing.T) {
	// A real binary the probe will find, created on PATH for this test.
	binDir := t.TempDir()
	fake := filepath.Join(binDir, "fake-agent")
	if err := os.WriteFile(fake, []byte("#!/bin/sh\n"), 0o755); err != nil {
		t.Fatalf("writing fake agent: %v", err)
	}
	t.Setenv("PATH", binDir+string(os.PathListSeparator)+os.Getenv("PATH"))

	h := harnesstest.Start(t)
	repo := harnesstest.NewSpaceRepo(t)
	harnesstest.WriteFile(t, repo, ".wayfinder-harness.toml", `
[roles.implement]
adapter = "fake-agent"

[roles.review]
adapter = "no-such-agent-xyz"
`)

	resp := register(t, h, repo)
	s := findSpace(t, h.Snapshot(ctx(t)), resp.ID)

	if impl := binding(t, s, "implement"); !impl.Present || impl.Missing != "" {
		t.Errorf("implement present=%v missing=%q, want present with no badge", impl.Present, impl.Missing)
	}

	rev := binding(t, s, "review")
	if rev.Present {
		t.Error("review bound to a missing adapter reports present")
	}
	if !strings.Contains(rev.Missing, "no-such-agent-xyz") || !strings.Contains(rev.Missing, "PATH") {
		t.Errorf("absence badge = %q, want it to name the adapter and PATH", rev.Missing)
	}
}

// Pinning reorders the sidebar: a pinned space sorts ahead of unpinned ones
// regardless of recency (story 6). The snapshot carries spaces already ordered.
func TestPinOrdersAhead(t *testing.T) {
	h := harnesstest.Start(t)
	older := register(t, h, harnesstest.NewSpaceRepo(t))
	time.Sleep(5 * time.Millisecond) // distinct recency timestamps
	newer := register(t, h, harnesstest.NewSpaceRepo(t))

	// Newest-registered sorts first by recency.
	if got := h.Snapshot(ctx(t)).Spaces[0].ID; got != newer.ID {
		t.Fatalf("first space by recency = %s, want the newer %s", got, newer.ID)
	}

	// Pin the older one; it must now lead despite being less recent.
	if code, body := h.Post("/api/spaces/"+older.ID+"/pin", map[string]bool{"pinned": true}); code != 204 {
		t.Fatalf("pin = %d, body %s", code, body)
	}
	snap := h.Snapshot(ctx(t))
	if snap.Spaces[0].ID != older.ID {
		t.Errorf("first space after pin = %s, want the pinned %s", snap.Spaces[0].ID, older.ID)
	}
	if !snap.Spaces[0].Pinned {
		t.Error("pinned space does not report pinned in the snapshot")
	}
}

// --- small local assertion helpers ---------------------------------------

func assertField(t *testing.T, name, gotVal, wantVal, gotFrom, wantFrom string) {
	t.Helper()
	if gotVal != wantVal {
		t.Errorf("%s value = %q, want %q", name, gotVal, wantVal)
	}
	if gotFrom != wantFrom {
		t.Errorf("%s resolved from %q, want %q", name, gotFrom, wantFrom)
	}
}

func hasSubstring(list []string, sub string) bool {
	for _, s := range list {
		if strings.Contains(s, sub) {
			return true
		}
	}
	return false
}

func worktreeFiles(t *testing.T, repo string) []string {
	t.Helper()
	var out []string
	err := filepath.WalkDir(repo, func(path string, d os.DirEntry, err error) error {
		if err != nil {
			return err
		}
		if d.IsDir() {
			if d.Name() == ".git" {
				return filepath.SkipDir // git's own internals are not the working tree
			}
			return nil
		}
		rel, _ := filepath.Rel(repo, path)
		out = append(out, rel)
		return nil
	})
	if err != nil {
		t.Fatalf("walking %s: %v", repo, err)
	}
	sort.Strings(out)
	return out
}

func readFile(t *testing.T, path string) string {
	t.Helper()
	b, err := os.ReadFile(path)
	if err != nil {
		t.Fatalf("reading %s: %v", path, err)
	}
	return string(b)
}

func equalStrings(a, b []string) bool {
	if len(a) != len(b) {
		return false
	}
	for i := range a {
		if a[i] != b[i] {
			return false
		}
	}
	return true
}
