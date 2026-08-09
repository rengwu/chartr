# Handoff — skill-sources redesign (mirror + ship-none)

Repo: wayfinder-harness (chartr). Branch: `main`. Working tree clean at handoff.
Next session focus: **Task 5 — focus-triggered mirror sync + "Syncing skills…" UI.**

## What shipped (all committed, all green)

- `622e985` feat(sources): the skill **mirror** + **seed teardown**
- `a424bc8` docs: ADR 0018, glossary, getting-started
- `2516740` test: suite updated for the teardown + mirror coverage

Green at handoff: `go vet ./...`, `go test ./...`, and `web` `npm run check` / `build` / `vitest` (195 tests).

## The decisions (read the records — don't re-derive)

- `docs/adr/0018-skill-mirror-and-no-seed.md` — the live record.
- Banner atop `docs/adr/0017-skills-come-from-registered-sources.md` — what of 0017 is superseded.
- `CONTEXT.md` glossary — **Source**, **Binding**, **Mirror** (new); **Seed** removed.
- `docs/getting-started.md` §8.

In one line each: (1) every enabled source's skills are copied into a gitignored
`<space>/.chartr/skills/<source>/<skill>/` and payloads point at repo-relative
`.chartr/skills/<source>`, so a sandboxed agent can read cross-referenced skills;
(2) chartr ships **no** skills — empty source list on first run, roles unbound
until the operator registers + binds; the restore-binding feature is gone.

## Key code (reference, not restated here)

- `internal/sources/mirror.go` — `Registry.Mirror(dest)`; `MirrorDir = ".chartr/skills"`.
  In-place reconcile, regular files only (symlinks skipped), prunes what no
  enabled source accounts for, writes a `*` `.gitignore` at the mirror root.
- `internal/server/chartrmd.go` — `ensureSkillsCurrent(entry)` barrier (skips
  scratch spaces and a nil registry).
- Barriers wired in `internal/server/spawn.go` (`launchSession` → covers spawn +
  respawn) and `internal/server/terminals.go` (`launchFree`).
- `internal/prompt/compose.go` `sourcesPart` — prints the mirror-relative path.
- `internal/chartrtest/rig.go` — `Start` now auto-registers a stand-in
  `chartr-skills` dir source + binds the four roles (recreates the pre-teardown
  test environment). `WithoutSkills()` and `WithConfigDir(...)` opt out.
  `SeedSkills(t)` is exposed for `WithConfigDir` tests that need skills.

## Task 5 — what's left, and why it's shaped this way

The mirror is refreshed **only at spawn** (the barrier). It is deliberately NOT
written on registration/startup: that was tried and reverted because it breaks
the "registration writes nothing into the repo" invariant
(`internal/server/spaces_test.go` `TestForgetNotDestroy`) and collides with the
retired legacy `.chartr/skills` layer (`firstrun_test.go`). So keeping the mirror
fresh for **ad-hoc sessions** — an agent reading `CHARTR.md` that chartr did not
spawn — is the focus-sync's job, not registration's.

Design decided in the prior grilling (Q7–Q12), not yet built:

- **No "focused space" signal exists today.** Focus is only per-terminal-tab (the
  notification dot: `internal/server/terminals.go` `handleTerminalSeen`,
  `internal/terminal/manager.go` `MarkSeen`). A new frontend→backend
  focused-space signal must be built — this is the largest, most coupled piece.
- On focus, after a ~5s debounce, do a cheap local-source tree walk; reconcile
  that space's mirror only if a source changed (cheap fingerprint/mtime cache).
  Global rate-limit so rapid space-switching coalesces to one scan per source.
  Periodic rescan as a backstop for missed events.
- `ensureSkillsCurrent` stays the **correctness** mechanism; focus-sync is a
  prewarm. Remote (git) sources refresh only on explicit register/refresh — no
  generations; leave that alone.
- **"Syncing skills…"**: the barrier already blocks the spawn; surface a brief
  syncing state on the spawn button. **Read `docs/design-system.md` first**
  (required before any UI — ADR 0012, chrome/island split ADR 0010); use
  `web/src/lib/components/ui/` primitives and Phosphor icons, tokens only.

Gotchas: keep mirror paths **relative** (absolute paths would break composing
`CHARTR.md` once for all spaces); the mirror owns `.chartr/skills/` and prunes
anything not from a source.

## Suggested skills for the next agent

- **Read `docs/design-system.md`** before touching UI (project hard rule).
- `run` — launch/screenshot the app to verify the Syncing state in situ.
- `code-review` / `simplify` — before finalizing the change.
- `wayfinder` + `to-tickets` — if you want Task 5 formalized as a `.plan/` map
  first. Note chartr ships no skills; these come from a registered source
  (`github.com/rengwu/skills`, per the operator's setup). No mattpocock skills
  are installed locally — this handoff skill was fetched ad hoc.

## Conventions

Commit straight to `main` (no branch). Before committing frontend changes: `web`
check + build + vitest, plus `go vet ./...` / `go test ./...`. Vocabulary is
load-bearing (use CONTEXT.md terms); don't strip the why-comments.
