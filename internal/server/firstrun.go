package server

import (
	"fmt"
	"log"
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

	// DefaultSkillSourceName and DefaultSkillSourceURL are the git source a fresh
	// install pre-registers so it can spawn from the first run — chartr's own
	// skills repo. It is an ordinary `git` source, cloned like any other and the
	// operator's to remove, refresh or reorder; chartr still ships no skills of its
	// own inside the binary. The real entrypoints set the URL on server.Options;
	// tests leave it empty (see Options.DefaultSourceURL) so the suite never clones
	// over the network.
	DefaultSkillSourceName = "chartr-skills"
	DefaultSkillSourceURL  = "https://github.com/rengwu/chartr-skills"
)

// firstRun brings the config root up to the state this build expects and returns
// the loaded source list. It runs at every startup; each step carries its own
// idempotence, and the whole migration half fires exactly once per machine, on
// the absence of `sources.toml`.
func firstRun(configDir, defaultSourceURL string) (*sources.Registry, error) {
	fresh, err := migrateSkillLayers(configDir)
	if err != nil {
		return nil, err
	}
	// The operator's ordered list. chartr ships no skills inside the binary (ADR
	// 0017), but a fresh install pre-registers chartr's own skills repo as an
	// ordinary `git` source so the very first run has something to spawn from —
	// theirs to remove, refresh or reorder like any other. What else lands in the
	// list on a first run is only what migrateSkillLayers carried forward.
	srcs, err := sources.Load(configDir)
	if err != nil {
		return nil, err
	}
	// Only on the run that first writes `sources.toml` (the same signal the layer
	// migration fires on), and only when an entrypoint asked for it — tests leave
	// the URL empty so the suite never clones. Best-effort: a fresh install with no
	// network or no `git` still comes up, just with an empty list the operator
	// fills in from Settings, rather than failing to start.
	if fresh && defaultSourceURL != "" {
		if _, err := srcs.RegisterGit(DefaultSkillSourceName, defaultSourceURL, ""); err != nil {
			log.Printf("chartr: could not pre-register the default skill source %q (%v); "+
				"register a source in Settings to spawn", defaultSourceURL, err)
		}
	}
	// The operator's own preferences file lives under the config root
	// (skill-sources ticket 03), but chartr no longer stamps it: like the two
	// cores it is absent until the operator creates it from Settings, and a
	// composition reads it verbatim when present or composes empty when not. This
	// call stays only to fail startup fast on a preferences.md that exists but
	// cannot be read, never to create anything. The file-format contract that used
	// to be reconciled here too now lives per-space instead —
	// ensureConventionsCurrent reconciles it, at the same call sites that keep the
	// skill mirror current.
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
// even with nothing to migrate, so it cannot fire twice. The returned bool is
// that same "this is the first run on this machine" signal, so firstRun can
// pre-register the default source exactly when the migration itself fired.
func migrateSkillLayers(configDir string) (bool, error) {
	if configDir == "" {
		return false, nil
	}
	switch _, err := os.Stat(sources.FilePath(configDir)); {
	case err == nil:
		return false, nil // the list exists: this machine has already been through here
	case !os.IsNotExist(err):
		return false, fmt.Errorf("reading the source list before migrating: %w", err)
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
		return false, err
	}
	if registerLegacy {
		if _, err := r.RegisterDir(legacySourceName, legacy); err != nil {
			return false, fmt.Errorf("migrating %s: %w", legacy, err)
		}
	}
	if registerBuiltin {
		if _, err := r.RegisterDir(migratedBuiltinNm, builtin); err != nil {
			return false, fmt.Errorf("migrating %s: %w", builtin, err)
		}
	}
	// Written even when neither row applies: the file's absence is the signal this
	// migration fires on, so a machine with nothing to migrate must still leave
	// one behind.
	if err := r.Save(); err != nil {
		return false, err
	}
	return true, nil
}
