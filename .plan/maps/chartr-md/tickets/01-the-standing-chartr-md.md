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

## Answer

`CHARTR.md` now stands at the root of every registered non-scratch space,
carrying the free session's brief minus the one clause that is only true of a
shell chartr opened, excluded through `.git/info/exclude`, and reconciled at
startup and on each of the five sources mutations.

### The split, and the sentence that was false

`internal/prompt/assets/core-free.md` became `assets/core-space.md` and lost its
tail. The opening was rephrased from *"the cockpit that opened this terminal"* to
*"the cockpit that drives this repository"*, which is true whether chartr
launched the reader or the reader wandered in; the two sentences that follow —
maps derived from `.plan/maps/`, one agent session per ticket — were already true
of both readers and are untouched. The free-only clause is now
`freeCoreTail` (*"This shell is one chartr opened with no ticket and no role."*),
a const beside the embed, appended by `ComposeFree` alone.

`ComposeFree` and the new `ComposeStanding(configDir, reg)` are two one-line
wrappers over a shared `composeSpace(configDir, reg, tail)`, so the four parts,
their order and every content rule are the same code and cannot drift apart. The
recommended shape, taken as recommended.

### Where it goes

`internal/server/chartrmd.go`: `reconcileChartrDoc` composes once — the document
has no per-space input at all — and fans the bytes out over `s.reg.List()`,
skipping `Scratch`. Each space gets `writeChartrDoc`: skip silently if the
directory is not there, compare bytes and write only on a difference (via
`writeOwnerFile`, 0600, same reasoning as the per-session payload), then
`ensureGitExclude`. The exclude is a literal line scan over
`.git/info/exclude`, appended once, existing bytes preserved verbatim and a
missing trailing newline repaired before the append; `.git` that is not a
directory means skip the exclude and still write the document. Nothing returns an
error to a caller — a failed space is logged and converges on the next trigger.

Call sites: `server.New` after the registry and `firstRun` are both loaded and
before `Serve` (not `firstRun`, which takes a config root and knows no registry),
and one explicit call in each of the five handlers in `sourcesapi.go` after their
`rebuild()`. Not inside `rebuild()`. `handleRestoreRoleBinding` is deliberately
*not* a trigger: bindings do not appear in the document.

### Done-when, clause by clause

- **A `/new` agent finds it, and the first sentence is true of them.** The
  document opens *"chartr is the cockpit that drives this repository"*;
  `TestStandingDocLandsInARegisteredSpaceAndGitIgnoresIt` asserts that prefix and
  asserts the absence of both *"opened this terminal"* and *"This shell"*.
  `TestStandingPayloadGolden` asserts the same at the composer, and that the free
  payload still carries the clause the standing one drops.
- **`git status` clean, tracked `.gitignore` unmodified.** Same test: the space
  is a real repository with a committed `.gitignore`, and after the reconcile
  `git status --porcelain` is empty and `git show HEAD:.gitignore` is unchanged.
- **All five source mutations rewrite it.**
  `TestSourcesMutationRewritesTheStandingDoc` drives register and toggle through
  HTTP and reads the resulting document out of the space; reorder is exercised as
  the no-op case below. Remove and refresh share the identical two-line call site
  and are not separately driven — refresh needs a git remote, which this test
  would have to build to assert a line of code already asserted four times.
- **A golden.** `internal/prompt/testdata/standing-payload.golden.md`, beside the
  free one, written by the same `-update` flag and `checkGolden`. The free golden
  moved by exactly the split (opening rephrased, tail its own paragraph).
- **The five tests named.** Exclude appended once across two runs (a second
  `Start` on the same config root); an existing exclude keeping its bytes,
  including the no-trailing-newline case; scratch receiving nothing (`HOME`
  pointed at a temp dir, so the assertion is real rather than a guess about the
  developer's home); an unreachable space not failing a sources mutation (a
  registered repo deleted before the register call, which still returns 200); and
  no `https://` in the document, asserted in both the server and prompt tests.
- **The per-session path untouched.** `writeSessionPayload`, the `*` ignore and
  `adapter.Opener` are not in the diff, and `spawn_test.go` / `payload_test.go`
  are unchanged.
- **Checks.** `go vet ./...` and `go test ./...` pass; in `web/`, `check` (0
  errors, 0 warnings), `vitest` (194 passed) and `build` pass. No UI changed.

### Deliberately not done, and one thing worth watching

- **Registration is still not a trigger**, so a just-registered space has no
  `CHARTR.md` until the next restart or sources change. The map holds this open on
  purpose; the test asserts the gap exists rather than papering over it, so
  closing it later will fail that assertion loudly.
- **`mine`'s no-op reorder** is how the write-only-on-difference rule is tested:
  the mtime must not move. Worth knowing that `Reorder` takes only the
  *non-default* names — the seeded source is held outside the reorderable rows —
  which is not obvious from the handler.
- **Nothing routes an agent to this file.** Discovery remains hopeful, as charted.
  The only new lever a future ticket has is the operator saying "read CHARTR.md",
  and `CLAUDE.md`/`AGENTS.md` stay out of scope.
