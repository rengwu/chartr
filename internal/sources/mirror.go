package sources

import (
	"bytes"
	"errors"
	"fmt"
	"io/fs"
	"os"
	"path"
	"path/filepath"
	"strings"
)

// The mirror is a repo-local copy of every enabled source's skills, written into
// a space at `.chartr/skills/<source>/<skill>/…`. It exists so an agent sandboxed
// to its own space — one that can read only inside the working tree — still
// reaches every skill a payload names. The role skill's body is still inlined
// into the payload (ADR 0002/0005, unchanged); the mirror is how the *other*
// skills a role skill sends the agent to (a `grill-me` that says "run
// `/grilling`") become readable without a path that leaves the repo.
//
// This reverses the one consequence ADR 0017 left open — "the committed workspace
// layer `<space>/.chartr/skills/` goes inert" — but not the decision under it:
// the mirror is gitignored and per-machine, the same footing as `CHARTR.md` and
// `.chartr/run`. It is never committed, so nothing here reaches a teammate through
// a `git pull`, and "this project's skills" is still not a thing the list can
// express — every enabled source is mirrored into every space (global trust).
//
// It is reconciled *in place*, not replaced wholesale like the seed: a live
// session reads these files on demand, so the directory is brought to match the
// sources file by file — a changed file rewritten, a vanished one removed — and
// the tree a running agent is reading is never renamed out from under it.

// MirrorDir is the mirror's path relative to a space's root. It is the one place
// the location is named: the server joins it onto a space to know where to write,
// and a payload's sources block prints it verbatim so the path an agent reads is
// repo-relative — identical in every space, which is what keeps the standing
// `CHARTR.md` composable once for all of them. It sits beside `.chartr/run`, the
// gitignored per-session payloads, on the same never-committed footing.
const MirrorDir = ".chartr/skills"

// mirrorIgnore is the file written at the mirror root that keeps the whole tree
// out of git. A `*` inside the directory is self-contained — it needs no entry in
// the repository's own `.gitignore`, so a fresh clone that never had the mirror
// still ignores it the moment chartr writes it — the same device `.chartr/run`
// uses beside the session payloads.
const mirrorIgnore = ".gitignore"

// MirroredSkill is one skill as it landed in the mirror: the source that yielded
// it, its name, and the absolute directory inside the mirror an agent reads it
// from. It is what a payload's sources block is rebuilt from once the paths point
// into the repo rather than out of it.
type MirroredSkill struct {
	Source string `json:"source"`
	Name   string `json:"name"`
	Dir    string `json:"dir"`
}

// Mirror brings dest into the state the enabled sources describe and returns the
// skills it wrote, in resolution order. dest is chartr's to own outright: every
// regular file below it that no enabled source accounts for is removed, so a skill
// deleted or disabled upstream actually disappears from the repo.
//
// Only regular files are copied. A symlink inside a skill is skipped — the minimal
// handling the operator asked for, and the safe one: a symlink can point outside
// the space and reopen the very sandbox hole the mirror exists to close. Each
// file keeps its source mode bits, so a bundled script stays executable.
//
// A single unreadable source file is skipped rather than raised: one bad file in
// someone else's repo must not cost a spawn its skills. A failure to *write* dest
// (a full disk, a read-only tree) is returned, because the caller that guarantees
// "current at launch" needs to know the mirror is not.
func (r *Registry) Mirror(dest string) ([]MirroredSkill, error) {
	if strings.TrimSpace(dest) == "" {
		return nil, errors.New("sources: mirror needs a destination directory")
	}

	// The desired tree, keyed by slash-separated path below dest, mapped to the
	// source file it is copied from. Built from the same walk resolution uses, so
	// the mirror carries exactly what the sources block lists — shadowed skills
	// included, since they are still reachable as `source/skill`.
	desired := map[string]string{}
	skills := []MirroredSkill{}

	for _, st := range r.States() {
		if !st.Enabled {
			continue
		}
		for _, sk := range st.Skills {
			base := path.Join(st.Name, sk.Name)
			skills = append(skills, MirroredSkill{
				Source: st.Name,
				Name:   sk.Name,
				Dir:    filepath.Join(dest, filepath.FromSlash(base)),
			})
			err := filepath.WalkDir(sk.Dir, func(p string, d fs.DirEntry, err error) error {
				if err != nil {
					return err
				}
				// A symlink — file or directory — is skipped, and a symlinked
				// directory is not descended into.
				if d.Type()&os.ModeSymlink != 0 {
					if d.IsDir() {
						return fs.SkipDir
					}
					return nil
				}
				if d.IsDir() {
					return nil
				}
				rel, err := filepath.Rel(sk.Dir, p)
				if err != nil {
					return err
				}
				desired[path.Join(base, filepath.ToSlash(rel))] = p
				return nil
			})
			if err != nil {
				return nil, fmt.Errorf("sources: reading the skill %q from %q: %w", sk.Name, st.Name, err)
			}
		}
	}

	if err := writeMirror(dest, desired); err != nil {
		return nil, err
	}
	return skills, nil
}

// writeMirror is the in-place reconcile: write or refresh every desired file,
// then remove every file the sources no longer account for, then prune the empty
// directories that leaves behind. The `.gitignore` marker at the root is written
// first and is never a prune target — it is chartr's, not a source's.
func writeMirror(dest string, desired map[string]string) error {
	if err := os.MkdirAll(dest, 0o755); err != nil {
		return fmt.Errorf("sources: creating the mirror directory: %w", err)
	}
	if err := os.WriteFile(filepath.Join(dest, mirrorIgnore), []byte("*\n"), 0o644); err != nil {
		return fmt.Errorf("sources: writing the mirror ignore marker: %w", err)
	}

	for rel, srcFile := range desired {
		info, err := os.Stat(srcFile)
		if err != nil {
			continue // vanished between the walk and the copy; skip it, don't fail the spawn
		}
		b, err := os.ReadFile(srcFile)
		if err != nil {
			continue
		}
		target := filepath.Join(dest, filepath.FromSlash(rel))
		if cur, err := os.ReadFile(target); err == nil && bytes.Equal(cur, b) {
			continue // already current — leave its mtime untouched
		}
		if err := os.MkdirAll(filepath.Dir(target), 0o755); err != nil {
			return fmt.Errorf("sources: writing the mirror: %w", err)
		}
		if err := os.WriteFile(target, b, info.Mode().Perm()); err != nil {
			return fmt.Errorf("sources: writing the mirror: %w", err)
		}
	}

	return pruneMirror(dest, desired)
}

// pruneMirror removes every regular file under dest that no source accounts for,
// then the directories left empty by that removal. The ignore marker is kept. An
// unreadable entry is left in place rather than raised: a mirror that carries one
// stale file is a smaller problem than a spawn that refuses over it.
func pruneMirror(dest string, desired map[string]string) error {
	var dirs []string
	err := filepath.WalkDir(dest, func(p string, d fs.DirEntry, err error) error {
		if err != nil {
			return nil
		}
		if p == dest {
			return nil
		}
		rel, err := filepath.Rel(dest, p)
		if err != nil {
			return nil
		}
		if d.IsDir() {
			dirs = append(dirs, p)
			return nil
		}
		slash := filepath.ToSlash(rel)
		if slash == mirrorIgnore {
			return nil
		}
		if _, ok := desired[slash]; !ok {
			_ = os.Remove(p)
		}
		return nil
	})
	if err != nil {
		return fmt.Errorf("sources: pruning the mirror: %w", err)
	}

	// Deepest first, so a directory emptied by its children's removal is itself
	// collected in the same pass. os.Remove refuses a non-empty directory, which
	// is exactly the test for "did anything survive here", so its error is ignored.
	for i := len(dirs) - 1; i >= 0; i-- {
		_ = os.Remove(dirs[i])
	}
	return nil
}
