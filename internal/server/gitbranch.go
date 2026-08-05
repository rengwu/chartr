package server

import (
	"os"
	"path/filepath"
	"strings"

	"github.com/rengwu/chartr/internal/gitroot"
)

// gitBranch resolves the Git root from the Space path, then reports the
// working tree's current branch by reading .git/HEAD — the checked-out ref's
// short name, or a short sha for a detached HEAD. It follows the one level of
// indirection a linked worktree uses (.git as a file pointing at the real
// gitdir). Empty on anything it cannot read, so the sidebar omits the branch
// rather than surfacing an error.
func gitBranch(spacePath string) string {
	root, err := gitroot.Resolve(spacePath)
	if err != nil {
		return ""
	}
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
		if !filepath.IsAbs(p) {
			p = filepath.Join(root, p)
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
