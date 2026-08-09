package server

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/rengwu/chartr/internal/prompt"
	"github.com/rengwu/chartr/internal/sources"
)

// The first-run sequence: everything chartr writes into its own config root
// before it serves anything, in one function and one order (skill-sources ticket
// 06). Each ticket of that effort adds its step here rather than inventing its
// own startup hook, so the order is readable in one place and the dependencies
// between the steps — a role binding must not be written before the source it
// points into is on disk — are enforced by position rather than by comment.
//
// **Nothing is reported.** Every write below is quiet: the migrated rows are
// discoverable the moment the operator opens Settings, and nothing pushes them,
// or the fact that a migrated fork no longer drives its old role, at the operator
// on the run it happens.

const (
	// legacySkillsDir is the user layer of the retired three-layer model:
	// `<configDir>/skills/`. chartr owns the bytes under its own config root, so
	// this one gets an active migration.
	legacySkillsDir = "skills"
	// materializedBuiltinDir is where the retired layer model materialized the
	// shipped library. Nothing writes it any more; the migration only reads it.
	materializedBuiltinDir = "builtin-skills"

	// The names the migrated rows carry in the operator's list. `Legacy skills`
	// first, `Migrated built-in skills` second, both before the default row: the
	// old order was workspace › user › built-in, and the surviving relative order
	// carries forward.
	legacySourceName  = "Legacy skills"
	migratedBuiltinNm = "Migrated built-in skills"
)

// firstRun brings the config root up to the state this build expects and returns
// the loaded source list. It runs at every startup; each step carries its own
// idempotence, and the whole migration half fires exactly once per machine, on
// the absence of `sources.toml`.
func firstRun(configDir string) (*sources.Registry, error) {
	if err := migrateSkillLayers(configDir); err != nil {
		return nil, err
	}
	// The operator's ordered list. chartr ships no skills of its own (ADR 0017),
	// so there is nothing to seed here and no default source to reconcile: an empty
	// list is the honest first-run state, and the role bindings stay unwritten until
	// the operator registers a source and binds each role. What lands in the list
	// on a first run is only what migrateSkillLayers carried forward.
	srcs, err := sources.Load(configDir)
	if err != nil {
		return nil, err
	}
	// chartr's file-format contract, and the operator's own preferences file, both
	// under the config root (skill-sources ticket 03). Startup is the first of two
	// reconcile points; every composition is the other, so an upgrade updates the
	// contract even in a process that never restarts a preview.
	if _, err := prompt.ReconcileContract(configDir); err != nil {
		return nil, err
	}
	return srcs, nil
}

// migrateSkillLayers carries an upgrade from the three-layer skill model into the
// source list, once per machine. Who owns the bytes decides each fate: the two
// directories under chartr's config root get this active migration, and the one
// inside the operator's repo (`<space>/.chartr/skills/`) gets nothing at all —
// chartr simply stops resolving it, and the directory is left exactly where it is
// without a word, going inert as silently as it goes unread.
//
// It fires on the absence of `sources.toml` and always leaves that file behind,
// even with nothing to migrate, so it cannot fire twice.
func migrateSkillLayers(configDir string) error {
	if configDir == "" {
		return nil
	}
	switch _, err := os.Stat(sources.FilePath(configDir)); {
	case err == nil:
		return nil // the list exists: this machine has already been through here
	case !os.IsNotExist(err):
		return fmt.Errorf("reading the source list before migrating: %w", err)
	}

	// The old user layer becomes an ordinary `dir` source, but only if it holds
	// something: an empty or absent directory contributes no row rather than a
	// permanently `empty` one. That an auto-registered fork of `implement` stops
	// driving `task` tickets is real but bounded to free-session bare-name lookups
	// — qualified bindings already closed that door for every source.
	legacy := filepath.Join(configDir, legacySkillsDir)
	registerLegacy := sources.HasSkills(legacy)

	// The materialized shipped library gets the same fate as the legacy one: if it
	// holds a skill it becomes a registered `dir` source, left exactly where it is.
	//
	// **This is where the cut (ticket 09) landed on ticket 06's design.** That
	// ticket split this directory two ways — byte-identical to shipped, renamed
	// aside and unregistered; diverging anywhere, registered in place — on a
	// comparison against the embedded shipped library. The cut deletes that embed,
	// so there is no longer anything to compare against and the rename-aside branch
	// has no test it could rest on. Registering unconditionally is the branch that
	// touches nothing on disk, which is the direction ticket 06 itself argued for:
	// keeping too much is the right error to make, and nothing in this effort
	// destroys data.
	//
	// In practice nothing is lost. Ticket 07 had already changed the embedded set
	// and the core's bytes, so the comparison answered "diverging" for every
	// already-materialized library on a real machine, and the release freeze means
	// no build between 06 and 09 ever shipped — the rename-aside branch was
	// unreachable on every upgrade path an operator can actually be on.
	builtin := filepath.Join(configDir, materializedBuiltinDir)
	registerBuiltin := sources.HasSkills(builtin)

	r, err := sources.Load(configDir)
	if err != nil {
		return err
	}
	if registerLegacy {
		if _, err := r.RegisterDir(legacySourceName, legacy); err != nil {
			return fmt.Errorf("migrating %s: %w", legacy, err)
		}
	}
	if registerBuiltin {
		if _, err := r.RegisterDir(migratedBuiltinNm, builtin); err != nil {
			return fmt.Errorf("migrating %s: %w", builtin, err)
		}
	}
	// Written even when neither row applies: the file's absence is the signal this
	// migration fires on, so a machine with nothing to migrate must still leave
	// one behind.
	return r.Save()
}
