package sources

import (
	"bytes"
	"embed"
	"fmt"
	"io/fs"
	"os"
	"path"
	"path/filepath"
	"strings"
)

// The seed: chartr's own `chartr-skills` repo, vendored as a checked-in copy of a
// pinned ref and materialized under the config root so a first run can spawn a
// role-typed session with no network at all.
//
// It is a checked-in copy rather than a build step that clones, because a
// release's contents must be a function of the commit and a fetching build would
// make the test suite need network; and rather than a submodule, because
// `go install` and source tarballs carry none, so the binary would ship with an
// empty default source and a first run would have no role skills. `make
// vendor-skills` is the whole refresh procedure — see its comment in the Makefile.
//
// The default source's directory is in exactly one of two states, read off the
// filesystem rather than recorded anywhere:
//
//	seeded — no `.git`     — chartr owns the bytes, reconciled at every startup
//	pinned — `.git` present — the operator owns them, chartr never writes there
//
// The seed records nothing about itself on disk. Its identity is SeedCommit,
// compiled in: an on-disk marker would be a second source of truth whose only
// distinctive behaviour is going stale. Deleting the directory is therefore the
// whole reset story — it covers reset, repair, and a directory an operator
// wrecked, and needs no action anywhere in the UI.

// SeedCommit is the ref the vendored copy under assets/ was taken at. `make
// vendor-skills` rewrites this line; nothing else may.
const SeedCommit = "ece028dfd19a21d6b4b990c96d0efe2fa5c83a49"

// SeedURL is where a refresh of the default source fetches from — the upstream of
// the vendored copy. It is a var rather than a const only so a test can point the
// seeded→pinned conversion at a local repository instead of the network; nothing
// in chartr reassigns it.
var SeedURL = "https://github.com/rengwu/chartr-skills.git"

//go:embed assets/chartr-skills
var seedFS embed.FS

const seedRoot = "assets/chartr-skills"

// Reconcile brings the default source's directory into the state the compiled
// seed describes, and is called at every startup.
//
// A `.git` inside it means the operator has pinned it with a fetch, and chartr
// never writes there again: an upgrade must not silently revert an ownership the
// operator asserted. Otherwise the whole directory is compared — file set and
// bytes — against the embedded copy and **replaced wholesale** when anything
// differs, temp-then-rename. Wholesale rather than per-file so a skill deleted
// upstream actually disappears; an operator who edited a seeded skill loses that
// edit at the next startup, which is the same stance the generated conventions
// take, and the answer to "I want to edit these" is a `dir` source of their own.
func Reconcile(configDir string) error {
	if configDir == "" {
		return nil
	}
	dest := DefaultPath(configDir)
	if Pinned(dest) {
		return nil
	}

	want, err := seedFiles()
	if err != nil {
		return err
	}
	have, err := readTree(dest)
	if err != nil {
		return err
	}
	if sameTree(want, have) {
		return nil
	}
	return replaceTree(dest, want)
}

// Pinned reports whether the operator owns the bytes at dir: a `.git` inside it
// is the whole test, and it is asked of the filesystem every time rather than
// remembered.
func Pinned(dir string) bool {
	_, err := os.Stat(filepath.Join(dir, ".git"))
	return err == nil
}

// seedFiles reads the vendored copy out of the binary, keyed by slash-separated
// path below the skills root.
func seedFiles() (map[string][]byte, error) {
	files := map[string][]byte{}
	err := fs.WalkDir(seedFS, seedRoot, func(p string, d fs.DirEntry, err error) error {
		if err != nil {
			return err
		}
		if d.IsDir() {
			return nil
		}
		b, err := seedFS.ReadFile(p)
		if err != nil {
			return err
		}
		rel, err := filepath.Rel(seedRoot, p)
		if err != nil {
			return err
		}
		files[filepath.ToSlash(rel)] = b
		return nil
	})
	if err != nil {
		return nil, fmt.Errorf("sources: reading the embedded seed: %w", err)
	}
	if len(files) == 0 {
		return nil, fmt.Errorf("sources: the embedded seed is empty; the binary was built without %s", seedRoot)
	}
	return files, nil
}

// readTree reads a directory's whole file set off disk, keyed the same way. An
// absent directory reads as an empty set — the first-run state, which simply
// differs from the seed and is written.
func readTree(dir string) (map[string][]byte, error) {
	files := map[string][]byte{}
	err := filepath.WalkDir(dir, func(p string, d fs.DirEntry, err error) error {
		if err != nil {
			if os.IsNotExist(err) {
				return nil
			}
			return err
		}
		if d.IsDir() {
			return nil
		}
		b, err := os.ReadFile(p)
		if err != nil {
			return err
		}
		rel, err := filepath.Rel(dir, p)
		if err != nil {
			return err
		}
		files[filepath.ToSlash(rel)] = b
		return nil
	})
	if err != nil && !os.IsNotExist(err) {
		return nil, fmt.Errorf("sources: reading %s: %w", dir, err)
	}
	return files, nil
}

func sameTree(want, have map[string][]byte) bool {
	if len(want) != len(have) {
		return false
	}
	for name, b := range want {
		if !bytes.Equal(b, have[name]) {
			return false
		}
	}
	return true
}

// replaceTree writes the whole tree to a temp directory beside dest and renames
// it into place, so a crash or a full disk mid-write can never leave the default
// source half-written: either the old directory stands or the new one does.
func replaceTree(dest string, files map[string][]byte) error {
	if err := os.MkdirAll(filepath.Dir(dest), 0o700); err != nil {
		return fmt.Errorf("sources: creating the sources directory: %w", err)
	}
	tmp, err := os.MkdirTemp(filepath.Dir(dest), ".seed-")
	if err != nil {
		return fmt.Errorf("sources: creating a temp directory for the seed: %w", err)
	}
	defer os.RemoveAll(tmp)

	for rel, b := range files {
		p := filepath.Join(tmp, filepath.FromSlash(rel))
		if err := os.MkdirAll(filepath.Dir(p), 0o700); err != nil {
			return fmt.Errorf("sources: writing the seed: %w", err)
		}
		if err := os.WriteFile(p, b, 0o600); err != nil {
			return fmt.Errorf("sources: writing the seed: %w", err)
		}
	}
	if err := os.RemoveAll(dest); err != nil {
		return fmt.Errorf("sources: clearing %s: %w", dest, err)
	}
	if err := os.Rename(tmp, dest); err != nil {
		return fmt.Errorf("sources: moving the seed into place: %w", err)
	}
	return nil
}

// SeedSkillNames lists the skills the vendored copy carries, in sorted order. It
// reads the embedded tree rather than a hard-coded list, so re-vendoring a repo
// that added or dropped a skill needs no second edit here.
func SeedSkillNames() []string {
	entries, err := seedFS.ReadDir(seedRoot)
	if err != nil {
		return nil
	}
	var out []string
	for _, e := range entries {
		if !e.IsDir() {
			continue
		}
		if _, err := fs.Stat(seedFS, path.Join(seedRoot, e.Name(), skillFile)); err != nil {
			continue
		}
		out = append(out, e.Name())
	}
	return out
}

// DefaultBinding is the qualified reference a seeded role binding points at:
// always `chartr-skills/<skill>`, never a bare name. A bare name resolved through
// source order would reintroduce exactly the implicit capture the binding table
// exists to prevent.
func DefaultBinding(skill string) string {
	return DefaultName + "/" + strings.TrimSpace(skill)
}
