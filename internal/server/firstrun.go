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
	// materializedBuiltinDir is where the shipped library is materialized. It
	// mirrors prompt's own unexported name; both go in ticket 09.
	materializedBuiltinDir = "builtin-skills"
	// migratedBuiltinDir is where an untouched materialized library is renamed
	// aside to, for the operator to remove or ignore. It is registered nowhere.
	migratedBuiltinDir = "builtin-skills.migrated"

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
	// The operator's ordered list, then chartr's own default source brought into
	// the state the compiled seed describes, then the four role bindings — in that
	// order, because a binding is seeded pointing into the default source and must
	// not be written before that source is on disk. Both writes are quiet and both
	// are once-only: the seed reconciles against a `.git` the operator's own fetch
	// left behind, and the bindings seed only on a startup that finds no `[roles]`
	// table at all (skill-sources ticket 05).
	srcs, err := sources.Load(configDir)
	if err != nil {
		return nil, err
	}
	if err := sources.Reconcile(configDir); err != nil {
		return nil, err
	}
	if err := seedRoleBindings(configDir); err != nil {
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

	// The materialized shipped library is compared once against the copy this
	// build embeds. Byte-identical, empty or absent: it is chartr's own bytes and
	// nothing is lost by moving it out of the way. Diverging anywhere: an
	// operator edited it, that edit lives nowhere else, and it survives in place
	// as a registered source.
	//
	// The untouched case is deliberately a *rename* and not a delete. It is the
	// only irreversible operation in this migration, it runs once, it is silent,
	// and it rests entirely on the comparison above being right — a stray
	// directory nobody cleans up and a wrong delete of someone's only copy are not
	// comparable.
	builtin := filepath.Join(configDir, materializedBuiltinDir)
	builtinExists := false
	if _, err := os.Stat(builtin); err == nil {
		builtinExists = true
	} else if !os.IsNotExist(err) {
		return fmt.Errorf("reading the built-in skill library before migrating: %w", err)
	}
	registerBuiltin := builtinExists && !prompt.MatchesShipped(builtin)
	if builtinExists && !registerBuiltin {
		aside := filepath.Join(configDir, migratedBuiltinDir)
		// A destination that already exists is left alone rather than merged into
		// or replaced: the directory being moved is byte-identical to what chartr
		// ships, so leaving it where it is costs nothing and clobbering something
		// the operator may have kept costs everything.
		if _, err := os.Stat(aside); os.IsNotExist(err) {
			if err := os.Rename(builtin, aside); err != nil {
				return fmt.Errorf("moving the built-in skill library aside: %w", err)
			}
		}
	}

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
