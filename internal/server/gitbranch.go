package server

import (
	"os"
	"path/filepath"
	"strings"
)

// gitBranch reports the working tree's current branch by reading .git/HEAD — the
// checked-out ref's short name, or a short sha for a detached HEAD. It follows
// the one level of indirection a linked worktree uses (.git as a file pointing
// at the real gitdir). Empty on anything it can't read, so the sidebar simply
// omits the branch rather than surfacing an error: this is a label, not a
// guarantee, and reading a file (not shelling out to git) keeps it cheap enough
// to run on every rebuild.
func gitBranch(root string) string {
	dir := filepath.Join(root, ".git")
	info, err := os.Stat(dir)
	if err != nil {
		return ""
	}
	if !info.IsDir() {
		// A linked worktree: .git is a file "gitdir: <path>" pointing at the git
		// directory that holds this worktree's HEAD.
		b, err := os.ReadFile(dir)
		if err != nil {
			return ""
		}
		p := strings.TrimSpace(strings.TrimPrefix(strings.TrimSpace(string(b)), "gitdir:"))
		if p == "" {
			return ""
		}
		dir = p
	}
	head, err := os.ReadFile(filepath.Join(dir, "HEAD"))
	if err != nil {
		return ""
	}
	s := strings.TrimSpace(string(head))
	if ref, ok := strings.CutPrefix(s, "ref: refs/heads/"); ok {
		return ref
	}
	// Detached HEAD: a raw sha. Show a short form.
	if len(s) >= 7 && !strings.ContainsAny(s, " \t") {
		return s[:7]
	}
	return ""
}
