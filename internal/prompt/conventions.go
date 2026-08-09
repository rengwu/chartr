package prompt

import (
	_ "embed"
	"fmt"
	"os"
	"path/filepath"
)

// conventionsText is chartr's file-format contract: the whole of what a writer
// must know to produce a map chartr can read. It is embedded rather than
// shipped as a skill because it is a *parser* contract — the one document an
// operator cannot shadow, disable, or reorder away, since a source whose
// skills write maps chartr cannot read is a source whose work is invisible.
//
//go:embed assets/conventions.md
var conventionsText string

const (
	// conventionsDir is the contract's directory, relative to a space's root —
	// the same `.chartr/` a spawn's skill mirror and run directory already live
	// under.
	conventionsDir = ".chartr"
	// conventionsName is the contract's materialized file name inside
	// conventionsDir. Deliberately not `conventions.md`, the embedded asset's own
	// name: this file has a different lifecycle (per-space, git-ignored) from
	// the asset it is generated from, and a distinct name keeps the two from
	// being mistaken for each other on disk.
	conventionsName = "TRACKER-CONVENTION.md"
	// conventionsIgnoreName marks the whole of `.chartr/` as git-ignored from
	// its root, the same `*` device the skill mirror and run directory already
	// use one level down (internal/sources/mirror.go) — self-contained, so a
	// fresh clone that never had the file still ignores it the moment chartr
	// writes it, with no line needed in the repository's own tracked
	// `.gitignore`. It must ignore itself too, not just conventionsName: a
	// marker that left itself trackable would still show `.chartr/` as dirty to
	// `git status`.
	conventionsIgnoreName = ".gitignore"

	// preferencesFile is the operator's own standing instructions, appended
	// after the conventions in every payload. chartr creates it empty and never
	// writes it again — the bytes are the operator's, unranked and unmerged.
	preferencesFile = "preferences.md"
)

// ConventionsRelPath is where the contract is materialized, relative to a
// space's root — identical in every space by construction. A payload names
// this path rather than inlining the document or pointing at an absolute
// location under the operator's config root: a relative path is what a session
// sandboxed to its own working tree can actually resolve, and being the same
// string in every space is what lets the standing `CHARTR.md` be composed once
// for all of them (see internal/sources/mirror.go, which the mirror path
// follows for the identical reason).
const ConventionsRelPath = conventionsDir + "/" + conventionsName

// Conventions returns the canonical bytes of the file-format contract.
func Conventions() string { return conventionsText }

// PreferencesPath is where the operator's own preferences file lives.
func PreferencesPath(configDir string) string {
	return filepath.Join(configDir, preferencesFile)
}

// Contract is the config-root document every payload carries verbatim: the
// operator's own preferences. The conventions half of what used to be a
// two-file contract under the config root is gone from here — its bytes now
// live per-space (ReconcileSpaceConventions), and a payload points at
// ConventionsRelPath directly rather than reading it from this struct.
type Contract struct {
	// Preferences is the raw content of `preferences.md` — empty when the
	// operator has written none. It is never wrapped, labelled, or ranked.
	Preferences string
}

// ReconcileContract brings the config root's `preferences.md` into the state a
// composition requires, and returns its bytes for a payload to inline
// verbatim.
//
// The file is created empty when absent and **never rewritten or merged**
// thereafter. An existing file that cannot be read fails here rather than
// composing without it — silently dropping the operator's own instructions is
// the one outcome that is not acceptable.
func ReconcileContract(configDir string) (Contract, error) {
	if configDir == "" {
		return Contract{}, nil
	}
	if err := os.MkdirAll(configDir, 0o700); err != nil {
		return Contract{}, err
	}

	prefs := PreferencesPath(configDir)
	b, err := os.ReadFile(prefs)
	switch {
	case err == nil:
	case os.IsNotExist(err):
		// First run, or the operator deleted it: recreate it empty and treat it as
		// empty. An absent preferences file is a normal state, not a failure.
		if err := writeAtomic(prefs, nil); err != nil {
			return Contract{}, err
		}
	default:
		return Contract{}, fmt.Errorf("reading %s: %w", prefs, err)
	}

	return Contract{Preferences: string(b)}, nil
}

// ReconcileSpaceConventions brings a space's own copy of the file-format
// contract to the embedded canonical bytes — the same generated-file treatment
// `conventions.md` got under the operator's config root before it moved here:
// missing or differing bytes are replaced atomically, so a chartr upgrade
// updates every space's copy and an operator's edit lasts exactly until the
// next reconcile.
//
// It writes into `.chartr/`, not the space root, and marks itself git-ignored
// there rather than touching the repository's own tracked ignore file — the
// same self-contained device the skill mirror and run directory use. This is
// what closes, for the one document that was still pointed at by an absolute
// path outside the space, the sandbox hole the mirror already exists to close:
// an agent sandboxed to its own working tree can now resolve
// `.chartr/TRACKER-CONVENTION.md` without a path that leaves the repo.
func ReconcileSpaceConventions(spaceDir string) error {
	if spaceDir == "" {
		return nil
	}
	dir := filepath.Join(spaceDir, conventionsDir)
	if err := os.MkdirAll(dir, 0o755); err != nil {
		return fmt.Errorf("creating %s: %w", dir, err)
	}

	ignore := filepath.Join(dir, conventionsIgnoreName)
	if cur, err := os.ReadFile(ignore); err != nil || string(cur) != "*\n" {
		if err := os.WriteFile(ignore, []byte("*\n"), 0o644); err != nil {
			return fmt.Errorf("writing %s: %w", ignore, err)
		}
	}

	path := filepath.Join(dir, conventionsName)
	if cur, err := os.ReadFile(path); err == nil && string(cur) == conventionsText {
		return nil // already current — leave its mtime untouched
	}
	return writeAtomic(path, []byte(conventionsText))
}

// writeAtomic writes b to path through a temp file in the same directory, so a
// reader never sees a half-written contract and a crash mid-write leaves the
// previous bytes intact.
func writeAtomic(path string, b []byte) error {
	tmp, err := os.CreateTemp(filepath.Dir(path), filepath.Base(path)+".*")
	if err != nil {
		return err
	}
	name := tmp.Name()
	defer os.Remove(name)
	if _, err := tmp.Write(b); err != nil {
		tmp.Close()
		return err
	}
	if err := tmp.Close(); err != nil {
		return err
	}
	if err := os.Chmod(name, 0o600); err != nil {
		return err
	}
	return os.Rename(name, path)
}
