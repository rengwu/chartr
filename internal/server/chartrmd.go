package server

import (
	"bytes"
	"log"
	"os"
	"path/filepath"
	"strings"

	"github.com/rengwu/chartr/internal/prompt"
)

// The standing `CHARTR.md` (chartr-md, ticket 01): the brief a free session is
// told, kept at the root of every registered space for the agents the opener
// never reaches — one spawned with `/new`, one the operator started in their own
// terminal, one that compacted its brief away and can re-read it.
//
// It is per-machine and never committed, which is what lets it carry the same
// bytes the per-session payload does — the sources block with its absolute
// config paths, and `preferences.md` verbatim. Nothing here reaches a teammate
// through a `git pull`, so the standing rule that execution config is never
// committed is not in tension with this file.
//
// Discovery is hopeful, not guaranteed. Nothing auto-reads it: `CLAUDE.md` is
// Claude Code's convention and `AGENTS.md` is the cross-harness one, and both
// belong to the operator, not to chartr. This file works on an agent's
// curiosity. That is the accepted ceiling.
const (
	// chartrDocName is the file's fixed name at the space root, and the literal
	// line written into `.git/info/exclude`.
	chartrDocName = "CHARTR.md"
	// gitExcludeRel locates git's local, uncommitted ignore file under `.git` — exactly what a
	// per-machine generated file needs. Not the space's tracked `.gitignore`:
	// that is the repository's own file, and writing to it would leave a phantom
	// diff in the operator's tree and an ignore rule in every clone for a file no
	// clone will ever have.
	gitExcludeRel = "info/exclude"
)

// reconcileChartrDoc composes the standing document once and writes it into
// every registered space. It is called from three places — startup, each of the
// five sources mutations, and a space registration — and deliberately *not* from
// rebuild(), which also fires on terminal churn and would make the trigger set
// unbounded.
//
// Registration was not a trigger as ticket 01 was charted, and became one on the
// operator's call: the gap it left was precisely the moment an operator adds a
// repository and opens an agent in it, so the document would be missing exactly
// when it is first wanted.
//
// It never returns an error. A space that cannot be written is a space that
// converges on the next startup or sources change, and neither the server coming
// up nor an operator's settings save is allowed to fail because one registered
// directory is on an unmounted volume.
func (s *Server) reconcileChartrDoc() {
	// Composed once: the document's only variable parts come from the config
	// root, so it is identical in every space.
	p, err := prompt.ComposeStanding(s.opts.ConfigDir, s.srcs)
	if err != nil {
		log.Printf("chartr: composing %s: %v", chartrDocName, err)
		return
	}
	doc := []byte(p.Markdown)

	for _, e := range s.reg.List() {
		// Scratch follows the operator's home directory. Writing CHARTR.md into `~`
		// is wrong on its face, and a home directory is not necessarily a git
		// repository either.
		if e.Scratch {
			continue
		}
		if err := writeChartrDoc(e.Path, doc); err != nil {
			log.Printf("chartr: writing %s in %s: %v", chartrDocName, e.Path, err)
		}
	}
}

// writeChartrDoc puts the document at the space root and makes git ignore it.
//
// An unreachable space — a deleted directory, an unmounted volume — is skipped
// silently rather than reported: it is a normal state for a registered space,
// not a failure of the write.
//
// The bytes are compared before writing so an unchanged space does not have its
// file's mtime touched. That keeps a clean tree clean and stops the watch from
// firing a rebuild on chartr's own write.
func writeChartrDoc(space string, doc []byte) error {
	if st, err := os.Stat(space); err != nil || !st.IsDir() {
		return nil
	}

	path := filepath.Join(space, chartrDocName)
	if existing, err := os.ReadFile(path); err == nil && bytes.Equal(existing, doc) {
		// Still ensure the exclude: the file can predate the line, or the operator
		// can have cleaned `.git/info/exclude` under a document that is current.
		return ensureGitExclude(space)
	}
	// Owner-only, for the same reason the per-session payload is: it carries the
	// operator's preferences and the paths chartr runs against, and that is no
	// other login's business.
	if err := writeOwnerFile(path, doc); err != nil {
		return err
	}
	return ensureGitExclude(space)
}

// ensureGitExclude appends the one-line `CHARTR.md` ignore to
// `<space>/.git/info/exclude`, creating the file and its directory if absent and
// preserving whatever is already there. Existing lines are never rewritten or
// reordered — the file is the operator's as much as it is chartr's.
//
// Idempotence is a literal line scan, not a `git check-ignore`: chartr wrote the
// line, so it knows exactly what it is looking for, and shelling out to git once
// per space on every startup and every settings save to learn something it
// already knows is a cost with no answer attached.
//
// If `.git` is not a directory the exclude is skipped and the document still
// written. A worktree or a submodule has `.git` as a *file* pointing elsewhere;
// ADR 0003 rules worktrees out, so this is a guard rather than a supported case —
// it must not fail, and it must not follow the pointer.
func ensureGitExclude(space string) error {
	gitDir := filepath.Join(space, ".git")
	st, err := os.Stat(gitDir)
	if err != nil || !st.IsDir() {
		return nil
	}

	path := filepath.Join(gitDir, gitExcludeRel)
	existing, err := os.ReadFile(path)
	if err != nil && !os.IsNotExist(err) {
		return err
	}
	for _, line := range strings.Split(string(existing), "\n") {
		if strings.TrimSpace(line) == chartrDocName {
			return nil
		}
	}

	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		return err
	}
	// The exclude is git's file in the operator's checkout, so it stays an
	// ordinary repository file at 0644 — the same reasoning as the `*` marker
	// beside the run directory.
	next := existing
	if len(next) > 0 && !bytes.HasSuffix(next, []byte("\n")) {
		next = append(next, '\n')
	}
	next = append(next, []byte(chartrDocName+"\n")...)
	return os.WriteFile(path, next, 0o644)
}
