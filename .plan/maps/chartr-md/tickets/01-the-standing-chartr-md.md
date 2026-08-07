---
type: task
claimed_by: s1d7d800233ea
claimed_at: 2026-08-07T07:37:08Z
---

# The standing CHARTR.md

## Question

Generate a `CHARTR.md` at the root of every registered space, carrying the text a
free session is told, excluded from git locally, reconciled at startup and on every
sources mutation.

### What it holds

The bytes are `prompt.ComposeFree`'s, with one problem to solve first. The free
core (`internal/prompt/assets/core-free.md`) opens:

> chartr is the cockpit that opened this terminal … and this shell was opened with
> no ticket and no role.

Both clauses are **false in the mouth of this file**. chartr did not open the
terminal of a `/new` agent, and that agent's shell was not opened by chartr at all.
The middle sentence — maps derived from `.plan/maps/`, one agent session per ticket
— is true for both readers and is the whole reason the file exists.

Recommended shape: split the asset so the two shared sentences are one part and the
free-session-only tail (*"this shell was opened with no ticket and no role"*) is
appended by `ComposeFree` alone. Then a sibling `ComposeStanding(configDir, reg)`
returns the same four parts without it, and the opening sentence is rephrased so it
is true unconditionally (chartr *drives this repository*, not *opened this
terminal*). One asset, two composers, two goldens. Take a different split if it is
smaller, but **do not ship a file whose first sentence is false to its audience** —
that is the failure this ticket exists to avoid.

Everything else about the free payload's content rules carries over unchanged and is
not reopened here: the ignore test on chartr's own voice, the sources block rendered
in resolution order with local paths and skill names, **no git URL ever printed**,
disabled sources absent, the shadowing sentence last, `preferences.md` appended
verbatim as the operator's own voice. The file carries **no live fact about the
space** — no map list, no frontier, no branch. A standing file has a strictly worse
staleness profile than a per-spawn one, so that rule binds harder here than it did
in ticket 07 of `skill-sources-impl`, not softer.

### Where it goes, and how git ignores it

`<space>/CHARTR.md`, written with `writeOwnerFile`'s mode — same reasoning as the
per-session payload, it holds the operator's preferences and it is theirs alone.

The ignore is one line, `CHARTR.md`, appended to `<space>/.git/info/exclude`:

- **Idempotent by line match.** chartr wrote the line, so a literal scan of the
  existing file is enough; do not shell out to `git check-ignore` on every space at
  every startup.
- **Create the file and its directory if absent**, preserving whatever is already
  there. Never rewrite or reorder existing lines.
- **If `.git` is not a directory, skip the exclude and still write `CHARTR.md`.** A
  worktree or submodule has `.git` as a *file*; ADR 0003 rules worktrees out, so
  this is a guard rather than a supported case. It must not fail and must not
  resolve the pointer.

### When it is written

Compose **once** — the document is identical for every space, since its only
variable parts come from the config root — then write it into each registered
space. Exactly two call sites:

1. **Startup**, after the space registry and the source list are both loaded and
   before the server serves. `firstRun` is the wrong home: it takes a `configDir`
   and knows nothing about the space registry.
2. **Every sources mutation** — the five handlers in `internal/server/sourcesapi.go`
   (`handleRegisterSource`, `handleRemoveSource`, `handleSetSourceEnabled`,
   `handleReorderSources`, `handleRefreshSource`).

**Not inside `rebuild()`** (`internal/server/spaces.go:192`). Every one of those
handlers calls it, but so does terminal churn and space registration, and the
trigger set is deliberately narrower than that. Call the reconcile explicitly.

Rules for the fan-out:

- **Skip scratch spaces** (`registry.Entry.Scratch`). A scratch space follows the
  operator's home directory; writing `CHARTR.md` into `~` is wrong, and it is not
  necessarily a git repository.
- **Write only when the bytes differ.** An unchanged space must not have its file's
  mtime touched, so a clean tree stays clean and no watcher fires.
- **A space that is unreachable is skipped silently** — a deleted directory, an
  unmounted volume. It converges on the next startup or sources change. Never fail
  startup, and never fail a settings save, because one space could not be written.

## Done when

- A `/new` agent in a registered space finds `CHARTR.md` at the repository root, and
  its first sentence is true of an agent chartr did not launch.
- `git status` in that space is clean: `CHARTR.md` is excluded through
  `.git/info/exclude`, and the space's tracked `.gitignore` is unmodified.
- Registering, removing, toggling, reordering or refreshing a source rewrites
  `CHARTR.md` in every registered non-scratch space.
- A golden file for the standing document, alongside the existing
  `internal/prompt/testdata/free-payload.golden.md`. **The golden is the guard on
  the ignore test** — chartr's own voice is a handful of sentences and nothing else
  will notice one more, so a diff on that file is the review.
- Tests covering: the exclude line appended once and not twice across two runs; an
  existing `.git/info/exclude` keeping its prior contents; a scratch space receiving
  no file; an unreachable registered space not failing a sources mutation; and no
  `https://` reaching the document.
- The per-session path is untouched — `.chartr/run/<sid>/payload.md`, its `*`
  ignore and `adapter.Opener` all behave exactly as before, with their tests
  unchanged.
- `go vet ./...`, `go test ./...`, and in `web/` the `check` and `build` scripts
  plus `vitest`, all pass.
