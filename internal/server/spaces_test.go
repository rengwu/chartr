package server_test

import (
	"context"
	"encoding/json"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"testing"
	"time"

	"github.com/rengwu/chartr/internal/chartrtest"
	"github.com/rengwu/chartr/internal/model"
)

// The registry at the process boundary: register with an announced git init,
// forget-not-destroy removal, a rebuildable index, and the sidebar order — one
// authority, the stored one, which nothing but an explicit reorder moves. Every
// assertion is on what the design makes public — HTTP responses, control-socket
// snapshots, the filesystem, and git — never on internals.

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

func register(t *testing.T, h *chartrtest.Chartr, path string) registerResp {
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

// Registering a plain folder makes it a space and, because it was not yet a git
// repository, initialises one — reported in the action's response, never silent
// (story 2). An already-registered repo is not re-initialised.
func TestRegisterInitialisesNonRepoAnnounced(t *testing.T) {
	h := chartrtest.Start(t)

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
	repo := chartrtest.NewSpaceRepo(t)
	resp2 := register(t, h, repo)
	if resp2.GitInited {
		t.Error("gitInited = true for an existing repo, want false")
	}
}

// De-registering forgets the entry and touches nothing in the repository — not
// git, not the working tree, not committed config (story 4). Registering must
// likewise write nothing into the repo: the registry lives in user config.
func TestForgetNotDestroy(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)

	chartrtest.WriteFile(t, repo, "README.md", "hello\n")
	chartrtest.Git(t, repo, "add", "-A")
	chartrtest.Git(t, repo, "commit", "-qm", "seed")
	chartrtest.WriteFile(t, repo, "dirty.txt", "uncommitted work\n")

	head := chartrtest.Git(t, repo, "rev-parse", "HEAD")
	status := chartrtest.Git(t, repo, "status", "--porcelain")
	files := worktreeFiles(t, repo)

	resp := register(t, h, repo)

	// Registering wrote nothing into the repo.
	if got := worktreeFiles(t, repo); !equalStrings(got, files) {
		t.Errorf("register changed the repo tree:\n before %v\n after  %v", files, got)
	}
	if _, err := os.Stat(filepath.Join(repo, ".chartr/config.toml")); !os.IsNotExist(err) {
		t.Error("register wrote a committed config file into the repo; it must not")
	}

	// Forget it.
	if code, body := h.Delete("/api/spaces/" + resp.ID); code != 204 {
		t.Fatalf("deregister = %d, body %s", code, body)
	}

	// Nothing in the repository moved.
	if got := chartrtest.Git(t, repo, "rev-parse", "HEAD"); got != head {
		t.Errorf("HEAD changed across register/forget: %s -> %s", head, got)
	}
	if got := chartrtest.Git(t, repo, "status", "--porcelain"); got != status {
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
// never work. chartr started against a config dir whose spaces.toml is gone
// shows no spaces, and re-registering the untouched repo restores it.
func TestRegistryLossIsRebuildable(t *testing.T) {
	configDir := t.TempDir()
	repo := chartrtest.NewSpaceRepo(t)
	chartrtest.WriteFile(t, repo, "keep.txt", "authoritative work lives in the repo\n")
	chartrtest.Git(t, repo, "add", "-A")
	chartrtest.Git(t, repo, "commit", "-qm", "work")
	head := chartrtest.Git(t, repo, "rev-parse", "HEAD")

	first := chartrtest.Start(t, chartrtest.WithConfigDir(configDir))
	resp := register(t, first, repo)
	if len(first.Snapshot(ctx(t)).Spaces) != 1 {
		t.Fatal("space not registered on the first chartr")
	}

	// Lose the registry, then bring a fresh chartr up on the same config dir.
	if err := os.Remove(filepath.Join(configDir, "spaces.toml")); err != nil {
		t.Fatalf("removing registry: %v", err)
	}
	second := chartrtest.Start(t, chartrtest.WithConfigDir(configDir))
	if got := len(second.Snapshot(ctx(t)).Spaces); got != 0 {
		t.Fatalf("after registry loss, snapshot has %d spaces, want 0", got)
	}

	// The repo — the authoritative state — is untouched, so re-adding restores.
	if got := chartrtest.Git(t, repo, "rev-parse", "HEAD"); got != head {
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

// spaceIDs is the sidebar sequence as the cockpit reads it: the snapshot's
// spaces in slice order, which the client renders without re-sorting.
func spaceIDs(m model.Model) []string {
	out := make([]string, 0, len(m.Spaces))
	for _, s := range m.Spaces {
		out = append(out, s.ID)
	}
	return out
}

// A newly registered space appends. The stored order belongs to the operator, so
// adding a repo lands it at the end rather than rearranging what is already
// there (story 5).
func TestNewSpaceLandsLast(t *testing.T) {
	h := chartrtest.Start(t)

	first := register(t, h, chartrtest.NewSpaceRepo(t))
	time.Sleep(5 * time.Millisecond) // distinct recency timestamps; they must not matter
	second := register(t, h, chartrtest.NewSpaceRepo(t))

	if got, want := spaceIDs(h.Snapshot(ctx(t))), []string{first.ID, second.ID}; !equalStrings(got, want) {
		t.Fatalf("sidebar = %v, want registration order %v", got, want)
	}

	third := register(t, h, chartrtest.NewSpaceRepo(t))
	snap := h.Snapshot(ctx(t))
	if got := snap.Spaces[len(snap.Spaces)-1].ID; got != third.ID {
		t.Errorf("last space = %s, want the newly registered %s", got, third.ID)
	}
}

// Activating a space no longer moves it. Recency is still recorded, it simply
// stops sorting anything, so the row under the operator's cursor stays put
// (story 3).
func TestActivatingASpaceDoesNotReorder(t *testing.T) {
	h := chartrtest.Start(t)

	a := register(t, h, chartrtest.NewSpaceRepo(t))
	time.Sleep(5 * time.Millisecond)
	b := register(t, h, chartrtest.NewSpaceRepo(t))
	time.Sleep(5 * time.Millisecond)
	c := register(t, h, chartrtest.NewSpaceRepo(t))

	want := []string{a.ID, b.ID, c.ID}
	if got := spaceIDs(h.Snapshot(ctx(t))); !equalStrings(got, want) {
		t.Fatalf("sidebar = %v, want %v", got, want)
	}

	// Re-registering an existing path is how a space is activated: it refreshes
	// recency. Under the old rule each of these would have jumped to the top.
	for _, sp := range []registerResp{a, b} {
		time.Sleep(5 * time.Millisecond)
		register(t, h, sp.Path)
	}

	if got := spaceIDs(h.Snapshot(ctx(t))); !equalStrings(got, want) {
		t.Errorf("sidebar after activating = %v, want the unchanged %v", got, want)
	}
}

// Pinning is vestigial after the stored order: the flag is still written and
// still rides the snapshot, but it no longer moves a row. It goes away entirely
// with its route in the contract half of this work.
func TestPinNoLongerReorders(t *testing.T) {
	h := chartrtest.Start(t)
	first := register(t, h, chartrtest.NewSpaceRepo(t))
	last := register(t, h, chartrtest.NewSpaceRepo(t))

	if code, body := h.Post("/api/spaces/"+last.ID+"/pin", map[string]bool{"pinned": true}); code != 204 {
		t.Fatalf("pin = %d, body %s", code, body)
	}

	if got, want := spaceIDs(h.Snapshot(ctx(t))), []string{first.ID, last.ID}; !equalStrings(got, want) {
		t.Errorf("sidebar after pinning the last space = %v, want the unchanged %v", got, want)
	}
}

// The agent selector persists the operator's pick immediately, decoupled from a
// spawn: a PUT records it, the next snapshot reads it back as the space's
// remembered agent, and it survives without any session ever being launched.
func TestSetSpaceAgentPersistsWithoutSpawn(t *testing.T) {
	h := chartrtest.Start(t)
	sp := register(t, h, chartrtest.NewSpaceRepo(t))

	if got := findSpace(t, h.Snapshot(ctx(t)), sp.ID).LastAgent; got != "" {
		t.Fatalf("fresh space remembers %q, want nothing", got)
	}

	if code, body := h.Put("/api/spaces/"+sp.ID+"/agent", map[string]string{"agent": "opus"}); code != 204 {
		t.Fatalf("set agent = %d, body %s", code, body)
	}
	if got := findSpace(t, h.Snapshot(ctx(t)), sp.ID).LastAgent; got != "opus" {
		t.Errorf("remembered agent after set = %q, want opus", got)
	}

	// An empty agent is refused rather than silently clearing the memory.
	if code, _ := h.Put("/api/spaces/"+sp.ID+"/agent", map[string]string{"agent": ""}); code != 400 {
		t.Errorf("empty agent = %d, want 400", code)
	}
	if got := findSpace(t, h.Snapshot(ctx(t)), sp.ID).LastAgent; got != "opus" {
		t.Errorf("remembered agent after a refused empty set = %q, want opus", got)
	}
}

// --- small local assertion helpers ---------------------------------------

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
