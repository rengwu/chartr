package prompt

import (
	_ "embed"
	"fmt"
	"os"
	"path/filepath"
)

// conventionsText is chartr's file-format contract: the whole of what a writer
// must know to produce a map chartr can read. It is embedded rather than shipped
// as a skill because it is a *parser* contract — the one document an operator
// cannot shadow, disable, or reorder away, since a source whose skills write maps
// chartr cannot read is a source whose work is invisible.
//
//go:embed assets/conventions.md
var conventionsText string

const (
	// conventionsFile is chartr's generated write contract under the config root.
	// It is reconciled to the embedded bytes at startup and again before every
	// composition, so an upgrade updates it and an operator edit lasts until the
	// next compose.
	conventionsFile = "conventions.md"
	// preferencesFile is the operator's own standing instructions, appended after
	// the conventions in every payload. chartr creates it empty and never writes
	// it again — the bytes are the operator's, unranked and unmerged.
	preferencesFile = "preferences.md"
)

// Conventions returns the canonical bytes of the file-format contract.
func Conventions() string { return conventionsText }

// ConventionsPath is where the contract is materialized under configDir. A
// payload names this path rather than inlining the document: a location is a
// capability, and it survives being ignored.
func ConventionsPath(configDir string) string {
	return filepath.Join(configDir, conventionsFile)
}

// PreferencesPath is where the operator's own preferences file lives.
func PreferencesPath(configDir string) string {
	return filepath.Join(configDir, preferencesFile)
}

// Contract is the pair of config-root documents every payload carries: the path
// of the generated conventions, and the operator's preference bytes verbatim.
type Contract struct {
	// ConventionsPath is the on-disk location of the reconciled contract.
	ConventionsPath string
	// Preferences is the raw content of `preferences.md` — empty when the
	// operator has written none. It is never wrapped, labelled, or ranked.
	Preferences string
}

// ReconcileContract brings the config root's two contract files into the state a
// composition requires, and returns what a payload needs from them.
//
// The conventions file is *generated*: missing or differing bytes are replaced
// with the embedded canonical ones, atomically. This is the narrow exception to
// chartr's hackability — a contract a reader can edit is a contract the parser
// does not keep — so an upgrade updates it automatically and an operator's edit
// lasts exactly until the next composition.
//
// `preferences.md` is the opposite and must not be confused with it: created
// empty when absent and **never rewritten or merged** thereafter. An existing
// file that cannot be read fails here rather than composing without it — silently
// dropping the operator's own instructions is the one outcome that is not
// acceptable.
func ReconcileContract(configDir string) (Contract, error) {
	if configDir == "" {
		return Contract{}, nil
	}
	if err := os.MkdirAll(configDir, 0o700); err != nil {
		return Contract{}, err
	}

	conv := ConventionsPath(configDir)
	if cur, err := os.ReadFile(conv); err != nil || string(cur) != conventionsText {
		if err != nil && !os.IsNotExist(err) {
			// An unreadable conventions file is still chartr's to own: overwrite it
			// rather than fail, since its content is not the operator's to keep.
			_ = os.Remove(conv)
		}
		if err := writeAtomic(conv, []byte(conventionsText)); err != nil {
			return Contract{}, err
		}
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

	return Contract{ConventionsPath: conv, Preferences: string(b)}, nil
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
