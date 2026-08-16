package sources

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"

	"github.com/rengwu/chartr/internal/env"
)

// RegisterGit clones a repository of skills and appends it to the list. The
// clone is shallow and single-branch, lands in a temp directory and is renamed
// into place, and **the row is written only after that rename** — so nothing
// half-cloned is ever a source, and a failed clone leaves neither a row nor a
// directory behind.
//
// There is no confirm gate: pasting the URL is the deliberate act, and it is
// also the only assertion of trust in the source's whole lifetime. `git` absent
// from PATH refuses the registration at the gate, naming why; existing checkouts
// keep resolving, because resolution reads directories and never runs git.
func (r *Registry) RegisterGit(name, url, ref string) (Source, error) {
	name = strings.TrimSpace(name)
	url = strings.TrimSpace(url)
	ref = strings.TrimSpace(ref)
	if !validName(name) {
		return Source{}, fmt.Errorf("%w (got %q)", ErrBadName, name)
	}
	if url == "" {
		return Source{}, fmt.Errorf("sources: a %q source needs a url", KindGit)
	}
	if err := requireGit(); err != nil {
		return Source{}, err
	}

	r.mu.Lock()
	err := r.freeNameLocked(name)
	r.mu.Unlock()
	if err != nil {
		return Source{}, err
	}

	dest := r.checkoutPath(url)
	commit, ref, err := cloneInto(dest, url, ref)
	if err != nil {
		return Source{}, err
	}

	s := Source{
		Name:    name,
		Kind:    KindGit,
		Path:    dest,
		URL:     url,
		Ref:     ref,
		Commit:  commit,
		Fetched: time.Now().UTC(),
		Enabled: true,
	}

	r.mu.Lock()
	defer r.mu.Unlock()
	if err := r.freeNameLocked(name); err != nil {
		return Source{}, err
	}
	r.rows = append(r.rows, s)
	if err := r.saveLocked(); err != nil {
		r.rows = r.rows[:len(r.rows)-1]
		return Source{}, err
	}
	return s, nil
}

// Refresh moves a git source to the tip of its ref: fetch, hard-reset, record
// the commit and the timestamp. It is explicit and quiet — nothing here ever
// fetches unattended, and the only report is the new short sha on the returned
// row. The checkout is chartr's, not a workspace: a refresh discards local edits
// inside it, and the answer to "I want to edit this" is a `dir` source.
func (r *Registry) Refresh(name string) (Source, error) {
	s, ok := r.Get(name)
	if !ok {
		return Source{}, fmt.Errorf("%w: no source named %q", ErrNotFound, name)
	}
	if s.Kind != KindGit {
		return Source{}, fmt.Errorf("sources: %q is a %s source; there is nothing to fetch", s.Name, s.Kind)
	}
	if err := requireGit(); err != nil {
		return Source{}, err
	}

	commit, err := fetchInto(s.Path, s.Name, s.Ref)
	if err != nil {
		return Source{}, err
	}

	r.mu.Lock()
	defer r.mu.Unlock()
	for i := range r.rows {
		if !strings.EqualFold(r.rows[i].Name, s.Name) {
			continue
		}
		r.rows[i].Commit = commit
		r.rows[i].Fetched = time.Now().UTC()
		if err := r.saveLocked(); err != nil {
			return Source{}, err
		}
		return r.rows[i], nil
	}
	return Source{}, fmt.Errorf("%w: no source named %q", ErrNotFound, name)
}

// cloneInto clones url at ref into dest, shallow and single-branch, through a
// temp directory beside dest that is renamed into place — so nothing half-cloned
// is ever readable at dest, and a clone that fails partway leaves neither a
// directory nor anything for a caller to record. It returns the checked-out
// commit and the ref it landed on (the remote's own default branch, when the
// caller named none, so a later refresh fetches the same branch rather than
// whatever HEAD has become).
func cloneInto(dest, url, ref string) (string, string, error) {
	if err := os.MkdirAll(filepath.Dir(dest), 0o700); err != nil {
		return "", "", fmt.Errorf("sources: creating the checkouts directory: %w", err)
	}
	tmp, err := os.MkdirTemp(filepath.Dir(dest), ".clone-")
	if err != nil {
		return "", "", fmt.Errorf("sources: creating a temp directory to clone into: %w", err)
	}
	// One cleanup covers every failure below: a clone that fails partway leaves
	// nothing, and a successful one has already moved out of the temp directory.
	defer os.RemoveAll(tmp)

	into := filepath.Join(tmp, "checkout")
	args := []string{"clone", "--depth", "1", "--single-branch"}
	if ref != "" {
		args = append(args, "--branch", ref)
	}
	if out, err := git("", append(args, url, into)...); err != nil {
		return "", "", fmt.Errorf("sources: cloning %s failed: %w\n%s", url, err, out)
	}

	commit, err := head(into)
	if err != nil {
		return "", "", err
	}
	if ref == "" {
		ref, _ = branch(into)
	}

	if err := os.RemoveAll(dest); err != nil {
		return "", "", fmt.Errorf("sources: clearing %s: %w", dest, err)
	}
	if err := os.Rename(into, dest); err != nil {
		return "", "", fmt.Errorf("sources: moving the clone into place: %w", err)
	}
	return commit, ref, nil
}

// fetchInto moves an existing checkout to the tip of its ref and returns the new
// commit. A hard reset, because the checkout is chartr's and not a workspace.
func fetchInto(dir, name, ref string) (string, error) {
	if ref == "" {
		ref = "HEAD"
	}
	if out, err := git(dir, "fetch", "--depth", "1", "origin", ref); err != nil {
		return "", fmt.Errorf("sources: fetching %s failed: %w\n%s", name, err, out)
	}
	if out, err := git(dir, "reset", "--hard", "FETCH_HEAD"); err != nil {
		return "", fmt.Errorf("sources: resetting %s failed: %w\n%s", name, err, out)
	}
	return head(dir)
}

// requireGit is the gate. It is checked at registration and at refresh, and
// nowhere else: a checkout already on disk resolves like any other directory.
func requireGit() error {
	if _, err := exec.LookPath("git"); err != nil {
		return fmt.Errorf("%w, so a git source cannot be cloned or refreshed; register the folder as a %q source instead", ErrGitMissing, KindDir)
	}
	return nil
}

// git runs one git command in dir and returns its combined output, so a failure
// carries git's own error verbatim rather than a paraphrase of it.
func git(dir string, args ...string) (string, error) {
	cmd := exec.Command("git", args...)
	cmd.Dir = dir
	// git and whatever it shells out to (ssh, pagers, hooks) are host
	// binaries; they need the operator's environment, not the AppImage
	// bundle's loader paths.
	cmd.Env = env.HostEnviron()
	out, err := cmd.CombinedOutput()
	return strings.TrimSpace(string(out)), err
}

// head is the short sha a row records and a refresh reports.
func head(dir string) (string, error) {
	out, err := git(dir, "rev-parse", "--short=12", "HEAD")
	if err != nil {
		return "", fmt.Errorf("sources: reading the checked-out commit: %w\n%s", err, out)
	}
	return out, nil
}

// branch is the ref a clone landed on when the operator named none, recorded so
// a later refresh fetches the same branch rather than whatever the remote's HEAD
// has become.
func branch(dir string) (string, error) {
	out, err := git(dir, "rev-parse", "--abbrev-ref", "HEAD")
	if err != nil || out == "HEAD" {
		return "", err
	}
	return out, nil
}
