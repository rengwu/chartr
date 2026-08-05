package server_test

import (
	"context"
	"encoding/json"
	"net/http"
	"os"
	"path/filepath"
	"runtime"
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
	if strings.Contains(body, "gitRoot") || strings.Contains(body, "git_root") {
		t.Fatalf("register response exposes the internal Git root: %s", body)
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

func TestRegisterSubdirectoryUsesRootGitStateAndSpaceFiles(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	firstPath := filepath.Join(repo, "first")
	secondPath := filepath.Join(repo, "second")

	chartrtest.WriteMap(t, firstPath, "first-map", mapBody)
	chartrtest.WriteTicket(t, firstPath, "first-map", "01-first.md", ticket(1, "First", "[]", "task", ""))
	writeWorkspaceSkill(t, firstPath, "implement", "", "First skill")
	chartrtest.WriteMap(t, secondPath, "second-map", mapBody)
	chartrtest.WriteTicket(t, secondPath, "second-map", "01-second.md", ticket(1, "Second", "[]", "task", ""))
	writeWorkspaceSkill(t, secondPath, "research", "", "Second skill")
	chartrtest.Git(t, repo, "add", "--all")
	chartrtest.Git(t, repo, "commit", "-qm", "baseline")

	// This change is outside both selected Space paths. A root-wide Git action
	// must report it for both Spaces.
	chartrtest.WriteFile(t, repo, "outside.txt", "changed outside both Spaces")

	first := register(t, h, firstPath)
	second := register(t, h, secondPath)

	if first.GitInited || second.GitInited {
		t.Fatalf("child registration reported git init: first=%v second=%v", first.GitInited, second.GitInited)
	}
	if first.Path != firstPath || second.Path != secondPath {
		t.Fatalf("registration paths = %q and %q, want %q and %q", first.Path, second.Path, firstPath, secondPath)
	}
	if first.ID == second.ID {
		t.Fatalf("two Space paths share identity %q", first.ID)
	}
	for _, path := range []string{firstPath, secondPath} {
		if _, err := os.Stat(filepath.Join(path, ".git")); !os.IsNotExist(err) {
			t.Errorf("registration created %s/.git: %v", path, err)
		}
	}

	snapshot := h.Snapshot(ctx(t))
	firstSpace := findSpace(t, snapshot, first.ID)
	secondSpace := findSpace(t, snapshot, second.ID)
	paths := map[string]string{"first": firstPath, "second": secondPath}
	spaces := map[string]model.Space{"first": firstSpace, "second": secondSpace}
	for name, space := range spaces {
		if space.Path != paths[name] {
			t.Errorf("%s Space path = %q, want %q", name, space.Path, paths[name])
		}
		if space.Branch != "main" {
			t.Errorf("%s branch = %q, want main from the Git root", name, space.Branch)
		}
		if !space.Dirty {
			t.Errorf("%s is clean, want the change outside the selected Space path", name)
		}
	}
	if len(firstSpace.Maps) != 1 || firstSpace.Maps[0].Slug != "first-map" {
		t.Errorf("first Space maps = %+v, want only first-map", firstSpace.Maps)
	}
	if len(secondSpace.Maps) != 1 || secondSpace.Maps[0].Slug != "second-map" {
		t.Errorf("second Space maps = %+v, want only second-map", secondSpace.Maps)
	}
	if !hasWorkspaceSkill(firstSpace, "implement") || hasWorkspaceSkill(firstSpace, "research") {
		t.Errorf("first Space skills are not scoped: %+v", firstSpace.Skills)
	}
	if !hasWorkspaceSkill(secondSpace, "research") || hasWorkspaceSkill(secondSpace, "implement") {
		t.Errorf("second Space skills are not scoped: %+v", secondSpace.Skills)
	}
}

func hasWorkspaceSkill(space model.Space, name string) bool {
	for _, skill := range space.Skills {
		if skill.Name == name && skill.Layer == "workspace" {
			return true
		}
	}
	return false
}

func registerFailure(t *testing.T, h *chartrtest.Chartr, path string, wantStatus int) {
	t.Helper()
	code, body := h.Post("/api/spaces", map[string]string{"path": path})
	if code != wantStatus {
		t.Fatalf("register %s = %d, want %d, body %s", path, code, wantStatus, body)
	}
	var response map[string]any
	if err := json.Unmarshal([]byte(body), &response); err != nil {
		t.Fatalf("registration error is not JSON: %v (%q)", err, body)
	}
	if message, ok := response["error"].(string); !ok || strings.TrimSpace(message) == "" {
		t.Fatalf("registration error has no message: %s", body)
	}
	for _, field := range []string{"id", "path", "gitInited"} {
		if _, ok := response[field]; ok {
			t.Errorf("registration error exposes success field %q: %s", field, body)
		}
	}
}

func assertNotRegistered(t *testing.T, h *chartrtest.Chartr, path string) {
	t.Helper()
	if registeredPath(t, h, path) {
		t.Errorf("failed registration added %s to the registry", path)
	}
}

func assertNoGitEntry(t *testing.T, path string) {
	t.Helper()
	if _, err := os.Stat(filepath.Join(path, ".git")); !os.IsNotExist(err) {
		t.Errorf("failed registration created %s/.git: %v", path, err)
	}
}

func TestRegisterLinkedWorktreeUsesLinkedRoot(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	chartrtest.Git(t, repo, "commit", "--allow-empty", "-qm", "baseline")
	linked := filepath.Join(t.TempDir(), "linked")
	chartrtest.Git(t, repo, "worktree", "add", "-q", "-b", "linked-main", linked)
	child := filepath.Join(linked, "child")
	if err := os.MkdirAll(child, 0o755); err != nil {
		t.Fatalf("creating linked worktree child: %v", err)
	}
	chartrtest.WriteFile(t, linked, "outside-child.txt", "dirty in the linked worktree\n")

	dotGit := filepath.Join(linked, ".git")
	dotGitBefore, err := os.ReadFile(dotGit)
	if err != nil {
		t.Fatalf("reading linked worktree .git file: %v", err)
	}
	if info, err := os.Stat(dotGit); err != nil || !info.Mode().IsRegular() {
		t.Fatalf("linked worktree .git is not a file: %v", err)
	}

	root := register(t, h, linked)
	spaceChild := register(t, h, child)
	if root.GitInited || spaceChild.GitInited {
		t.Fatalf("linked worktree registration ran git init: root=%v child=%v", root.GitInited, spaceChild.GitInited)
	}
	if root.Path != linked || spaceChild.Path != child {
		t.Fatalf("registration paths = %q and %q, want %q and %q", root.Path, spaceChild.Path, linked, child)
	}
	assertNoGitEntry(t, child)
	if dotGitAfter, err := os.ReadFile(dotGit); err != nil || string(dotGitAfter) != string(dotGitBefore) {
		t.Errorf("linked worktree .git file changed: before=%q after=%q err=%v", dotGitBefore, dotGitAfter, err)
	}

	snapshot := h.Snapshot(ctx(t))
	for _, space := range []model.Space{
		findSpace(t, snapshot, root.ID),
		findSpace(t, snapshot, spaceChild.ID),
	} {
		if space.Branch != "linked-main" {
			t.Errorf("linked worktree branch = %q, want linked-main", space.Branch)
		}
		if !space.Dirty {
			t.Errorf("linked worktree Space is clean, want the dirty change in its Git root")
		}
	}
}

func TestRegisterNestedRepositoryAndSubmoduleUseInnermostRoots(t *testing.T) {
	h := chartrtest.Start(t)
	outer := chartrtest.NewSpaceRepo(t)
	chartrtest.WriteFile(t, outer, "outer.txt", "outer\n")
	chartrtest.Git(t, outer, "add", "--all")
	chartrtest.Git(t, outer, "commit", "-qm", "outer baseline")
	outerHead := chartrtest.Git(t, outer, "rev-parse", "HEAD")

	nested := filepath.Join(outer, "nested")
	if err := os.MkdirAll(nested, 0o755); err != nil {
		t.Fatalf("creating nested repository: %v", err)
	}
	chartrtest.Git(t, nested, "init", "-q", "-b", "nested-main")
	chartrtest.Git(t, nested, "config", "user.email", "chartr-test@example.com")
	chartrtest.Git(t, nested, "config", "user.name", "Chartr Test")
	chartrtest.Git(t, nested, "commit", "--allow-empty", "-qm", "nested baseline")
	nestedGitHeadBefore := readFile(t, filepath.Join(nested, ".git", "HEAD"))
	chartrtest.WriteFile(t, nested, "nested-dirty.txt", "nested change\n")

	source := chartrtest.NewSpaceRepo(t)
	chartrtest.WriteFile(t, source, "source.txt", "source\n")
	chartrtest.Git(t, source, "add", "--all")
	chartrtest.Git(t, source, "commit", "-qm", "source baseline")
	submodule := filepath.Join(outer, "submodule")
	chartrtest.Git(t, outer, "-c", "protocol.file.allow=always", "submodule", "add", "-q", source, "submodule")
	chartrtest.Git(t, submodule, "checkout", "-q", "-b", "submodule-main")
	submoduleGitBefore := readFile(t, filepath.Join(submodule, ".git"))
	chartrtest.WriteFile(t, submodule, "submodule-dirty.txt", "submodule change\n")

	nestedResp := register(t, h, nested)
	submoduleResp := register(t, h, submodule)
	if nestedResp.GitInited || submoduleResp.GitInited {
		t.Fatalf("nested repository registration ran git init: nested=%v submodule=%v", nestedResp.GitInited, submoduleResp.GitInited)
	}
	if nestedResp.Path != nested || submoduleResp.Path != submodule {
		t.Fatalf("registration paths = %q and %q, want %q and %q", nestedResp.Path, submoduleResp.Path, nested, submodule)
	}
	if got := chartrtest.Git(t, outer, "rev-parse", "HEAD"); got != outerHead {
		t.Errorf("outer repository HEAD changed from %s to %s", outerHead, got)
	}
	if got := readFile(t, filepath.Join(nested, ".git", "HEAD")); got != nestedGitHeadBefore {
		t.Errorf("nested repository .git/HEAD changed from %q to %q", nestedGitHeadBefore, got)
	}
	if got := readFile(t, filepath.Join(submodule, ".git")); got != submoduleGitBefore {
		t.Errorf("submodule .git file changed from %q to %q", submoduleGitBefore, got)
	}

	snapshot := h.Snapshot(ctx(t))
	nestedSpace := findSpace(t, snapshot, nestedResp.ID)
	submoduleSpace := findSpace(t, snapshot, submoduleResp.ID)
	if nestedSpace.Branch != "nested-main" || !nestedSpace.Dirty {
		t.Errorf("nested Space Git state = branch %q, dirty %v; want nested-main and dirty", nestedSpace.Branch, nestedSpace.Dirty)
	}
	if submoduleSpace.Branch != "submodule-main" || !submoduleSpace.Dirty {
		t.Errorf("submodule Space Git state = branch %q, dirty %v; want submodule-main and dirty", submoduleSpace.Branch, submoduleSpace.Dirty)
	}
}

func TestRegisterFailuresDoNotInitOrPersist(t *testing.T) {
	h := chartrtest.Start(t)

	missing := filepath.Join(t.TempDir(), "missing")
	registerFailure(t, h, missing, http.StatusBadRequest)
	assertNotRegistered(t, h, missing)

	regularFile := filepath.Join(t.TempDir(), "file.txt")
	if err := os.WriteFile(regularFile, []byte("not a Space\n"), 0o644); err != nil {
		t.Fatalf("creating regular file: %v", err)
	}
	registerFailure(t, h, regularFile, http.StatusBadRequest)
	assertNotRegistered(t, h, regularFile)

	if runtime.GOOS != "windows" {
		inaccessible := filepath.Join(t.TempDir(), "inaccessible")
		if err := os.Mkdir(inaccessible, 0o755); err != nil {
			t.Fatalf("creating inaccessible directory: %v", err)
		}
		if err := os.Chmod(inaccessible, 0); err != nil {
			t.Fatalf("removing directory permissions: %v", err)
		}
		t.Cleanup(func() { _ = os.Chmod(inaccessible, 0o755) })
		probe, err := os.Open(inaccessible)
		if err == nil {
			_ = probe.Close()
			t.Log("skipping inaccessible-directory case because this process can open mode-000 directories")
		} else {
			registerFailure(t, h, inaccessible, http.StatusBadRequest)
			assertNotRegistered(t, h, inaccessible)
			if err := os.Chmod(inaccessible, 0o755); err != nil {
				t.Fatalf("restoring inaccessible directory permissions: %v", err)
			}
			assertNoGitEntry(t, inaccessible)
		}
	}

	broken := t.TempDir()
	brokenGit := filepath.Join(broken, ".git")
	brokenGitBefore := []byte("gitdir: /path/that/does/not/exist\n")
	if err := os.WriteFile(brokenGit, brokenGitBefore, 0o644); err != nil {
		t.Fatalf("creating broken Git metadata: %v", err)
	}
	registerFailure(t, h, broken, http.StatusInternalServerError)
	assertNotRegistered(t, h, broken)
	if got, err := os.ReadFile(brokenGit); err != nil || string(got) != string(brokenGitBefore) {
		t.Errorf("broken .git marker changed: before=%q after=%q err=%v", brokenGitBefore, got, err)
	}

	bare := t.TempDir()
	chartrtest.Git(t, bare, "init", "-q", "--bare")
	registerFailure(t, h, bare, http.StatusInternalServerError)
	assertNotRegistered(t, h, bare)
	assertNoGitEntry(t, bare)

	missingGit := t.TempDir()
	t.Setenv("PATH", t.TempDir())
	registerFailure(t, h, missingGit, http.StatusInternalServerError)
	assertNotRegistered(t, h, missingGit)
	assertNoGitEntry(t, missingGit)
}

func TestRegisterRegistryFailureIsServerErrorAndKeepsInit(t *testing.T) {
	h := chartrtest.Start(t)
	plain := t.TempDir()

	if err := os.Chmod(h.ConfigDir, 0o500); err != nil {
		t.Fatalf("making config directory read-only: %v", err)
	}
	t.Cleanup(func() { _ = os.Chmod(h.ConfigDir, 0o700) })
	probe := filepath.Join(h.ConfigDir, "write-probe")
	if err := os.WriteFile(probe, []byte("probe"), 0o600); err == nil {
		_ = os.Remove(probe)
		t.Skip("process can write a mode-0500 config directory")
	}

	registerFailure(t, h, plain, http.StatusInternalServerError)
	if _, err := os.Stat(filepath.Join(plain, ".git")); err != nil {
		t.Errorf("registry failure removed the successful git init: %v", err)
	}
	assertNotRegistered(t, h, plain)
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
	if len(spaceIDs(first.Snapshot(ctx(t)))) != 1 {
		t.Fatal("space not registered on the first chartr")
	}

	// Lose the registry, then bring a fresh chartr up on the same config dir.
	if err := os.Remove(filepath.Join(configDir, "spaces.toml")); err != nil {
		t.Fatalf("removing registry: %v", err)
	}
	second := chartrtest.Start(t, chartrtest.WithConfigDir(configDir))
	if got := len(spaceIDs(second.Snapshot(ctx(t)))); got != 0 {
		t.Fatalf("after registry loss, snapshot has %d registered spaces, want 0", got)
	}

	// The repo — the authoritative state — is untouched, so re-adding restores.
	if got := chartrtest.Git(t, repo, "rev-parse", "HEAD"); got != head {
		t.Errorf("repo HEAD changed across registry loss: %s -> %s", head, got)
	}
	resp2 := register(t, second, repo)
	if resp2.ID != resp.ID {
		t.Errorf("re-registered id = %s, want the same stable id %s", resp2.ID, resp.ID)
	}
	if len(spaceIDs(second.Snapshot(ctx(t)))) != 1 {
		t.Error("re-registering did not restore the space")
	}
}

// spaceIDs is the sidebar sequence as the cockpit reads it: the snapshot's
// spaces in slice order, which the client renders without re-sorting.
func spaceIDs(m model.Model) []string {
	out := make([]string, 0, len(m.Spaces))
	for _, s := range m.Spaces {
		if s.Scratch {
			continue
		}
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

// The reorder the operator's drag emits: one endpoint takes the complete ordered
// list and the sidebar follows it, through the registry and back into the model
// snapshot the cockpit renders (story 1). It is the operator's arrangement, so it
// outlives the process that received it — a fresh chartr on the same config dir
// reads it straight back off disk (story 2).
func TestReorderSetsTheSidebarAndSurvivesARestart(t *testing.T) {
	configDir := t.TempDir()
	h := chartrtest.Start(t, chartrtest.WithConfigDir(configDir))

	a := register(t, h, chartrtest.NewSpaceRepo(t))
	b := register(t, h, chartrtest.NewSpaceRepo(t))
	c := register(t, h, chartrtest.NewSpaceRepo(t))

	if got, want := spaceIDs(h.Snapshot(ctx(t))), []string{a.ID, b.ID, c.ID}; !equalStrings(got, want) {
		t.Fatalf("sidebar = %v, want registration order %v", got, want)
	}

	// Drag the last space to the top.
	want := []string{c.ID, a.ID, b.ID}
	if code, body := h.Post("/api/spaces/reorder", map[string][]string{"ids": want}); code != 204 {
		t.Fatalf("reorder = %d, body %s", code, body)
	}
	if got := spaceIDs(h.Snapshot(ctx(t))); !equalStrings(got, want) {
		t.Fatalf("sidebar after reorder = %v, want %v", got, want)
	}

	// Sending the same list again changes nothing: the write is idempotent, which
	// is what makes a full-list write safe to emit on every drag.
	if code, body := h.Post("/api/spaces/reorder", map[string][]string{"ids": want}); code != 204 {
		t.Fatalf("repeat reorder = %d, body %s", code, body)
	}
	if got := spaceIDs(h.Snapshot(ctx(t))); !equalStrings(got, want) {
		t.Errorf("sidebar after a repeated reorder = %v, want the unchanged %v", got, want)
	}

	// The arrangement is on disk, not in this process: a fresh chartr over the
	// same registry shows the same sidebar.
	second := chartrtest.Start(t, chartrtest.WithConfigDir(configDir))
	if got := spaceIDs(second.Snapshot(ctx(t))); !equalStrings(got, want) {
		t.Errorf("sidebar after a restart = %v, want the arrangement %v", got, want)
	}
}

// A reorder republishes the snapshot permuted rather than deriving a new one —
// the sequence is the only thing it changes, and rebuilding costs a `git status`
// and a `.plan/` scan per space, which the operator watches the sidebar wait for.
// The risk that shortcut carries is a snapshot that keeps the order but loses
// what hangs off each space, so this asserts the derived halves survive the move:
// a space's discovered maps and its git branch arrive intact, in the new order.
func TestReorderKeepsEachSpacesDerivedState(t *testing.T) {
	h := chartrtest.Start(t)

	plain := chartrtest.NewSpaceRepo(t)
	mapped := chartrtest.NewSpaceRepo(t)
	chartrtest.WriteMap(t, mapped, "widget", mapBody)
	chartrtest.WriteTicket(t, mapped, "widget", "01-first.md", ticket(1, "First", "[]", "task", ""))

	a := register(t, h, plain)
	b := register(t, h, mapped)

	before := findSpace(t, h.Snapshot(ctx(t)), b.ID)
	if len(before.Maps) != 1 {
		t.Fatalf("maps before reorder = %d, want the one written", len(before.Maps))
	}
	if before.Branch == "" {
		t.Fatal("branch before reorder is empty; the fixture is on main")
	}

	want := []string{b.ID, a.ID}
	if code, body := h.Post("/api/spaces/reorder", map[string][]string{"ids": want}); code != 204 {
		t.Fatalf("reorder = %d, body %s", code, body)
	}

	m := h.Snapshot(ctx(t))
	if got := spaceIDs(m); !equalStrings(got, want) {
		t.Fatalf("sidebar after reorder = %v, want %v", got, want)
	}
	after := findSpace(t, m, b.ID)
	if len(after.Maps) != len(before.Maps) {
		t.Errorf("maps after reorder = %d, want the %d it had", len(after.Maps), len(before.Maps))
	}
	if after.Branch != before.Branch {
		t.Errorf("branch after reorder = %q, want the %q it had", after.Branch, before.Branch)
	}
	if after.Path != before.Path {
		t.Errorf("path after reorder = %q, want %q", after.Path, before.Path)
	}
}

// A reorder is the whole list or nothing. A request that omits a registered
// space, names an id the registry does not know, or repeats one is a client bug
// — refused with a 400 that leaves the previous arrangement exactly as it was,
// rather than applied as a partial reorder.
func TestReorderRejectsAListThatIsNotTheWholeRegistry(t *testing.T) {
	h := chartrtest.Start(t)

	a := register(t, h, chartrtest.NewSpaceRepo(t))
	b := register(t, h, chartrtest.NewSpaceRepo(t))
	c := register(t, h, chartrtest.NewSpaceRepo(t))

	// Put the sidebar in a known non-default arrangement first, so a rejected
	// request that silently reset the order would show up.
	want := []string{b.ID, c.ID, a.ID}
	if code, body := h.Post("/api/spaces/reorder", map[string][]string{"ids": want}); code != 204 {
		t.Fatalf("reorder = %d, body %s", code, body)
	}

	for _, tc := range []struct {
		name string
		ids  []string
	}{
		{"a list omitting a registered space", []string{b.ID, c.ID}},
		{"a list naming an unknown space", []string{b.ID, c.ID, "deadbeefcafe"}},
		{"a list repeating a space", []string{b.ID, c.ID, c.ID}},
		{"an empty list", []string{}},
		{"a list longer than the registry", []string{b.ID, c.ID, a.ID, a.ID}},
	} {
		t.Run(tc.name, func(t *testing.T) {
			if code, body := h.Post("/api/spaces/reorder", map[string][]string{"ids": tc.ids}); code != 400 {
				t.Errorf("reorder with %s = %d, want 400 (body %s)", tc.name, code, body)
			}
			if got := spaceIDs(h.Snapshot(ctx(t))); !equalStrings(got, want) {
				t.Errorf("sidebar after a refused reorder = %v, want the untouched %v", got, want)
			}
		})
	}
}

// Pin is gone, route and all. It was the ordering authority the stored order
// replaced, and it is the whole of what this half deletes: the ordering
// assertions above (activation does not move a row, a registration appends, a
// reorder is the one way a row moves) are what stand in its place. The sidebar is
// untouched by the call, because there is nothing there to call.
func TestPinRouteIsGone(t *testing.T) {
	h := chartrtest.Start(t)
	first := register(t, h, chartrtest.NewSpaceRepo(t))
	last := register(t, h, chartrtest.NewSpaceRepo(t))

	if code, body := h.Post("/api/spaces/"+last.ID+"/pin", map[string]bool{"pinned": true}); code != 404 {
		t.Errorf("post to the deleted pin route = %d, want 404 (body %s)", code, body)
	}

	if got, want := spaceIDs(h.Snapshot(ctx(t))), []string{first.ID, last.ID}; !equalStrings(got, want) {
		t.Errorf("sidebar after a post to the deleted pin route = %v, want the unchanged %v", got, want)
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
