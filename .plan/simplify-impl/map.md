# simplify — implementation

## Destination

The [spec](../simplify/spec.md) implemented end to end: the review pipeline gone
and the lifecycle vanilla wayfinder (write `## Answer`, commit, resolved); every
injected prompt a standard `SKILL.md` on disk; a first-class settings route that
shows every resolved config value with its provenance layer and edits role
bindings inline; and the cockpit launchable as a real native window behind the
one supported cgo-free browser binary. Done looks like the housekeeping sweep
resolved with every slice before it demoable in the real cockpit — a leaner, more
open harness that still drives its own maps.

## Notes

**This map carries execution.** Every ticket is a `task` that delivers working
code, not a decision — all decisions were made on the
[planning map](../simplify/map.md) and synthesized into the
[spec](../simplify/spec.md), which is the single source of truth here. Do not
re-litigate a decision; if implementation exposes one as wrong, mark the
*planning* ticket undermined and raise it, rather than quietly deviating.

**Per-session reading order:** the spec, then this map, then your ticket. The spec
carries the settled seams and symbols each ticket names — prefer them to
brittle line-level paths. Vocabulary comes from `CONTEXT.md` at the repo root; the
ADRs under `docs/adr/` are binding, and this effort **amends 0004 / 0008 / 0009,
reaffirms 0002, strikes one premise of 0007, and writes new 0013 / 0014** — a
ticket that touches an ADR says so in its answer.

**Sequencing (spec → Sequencing).** The order is `01, 02 → 03 → 04 → 05` with
`06` free: the webview shell (01) is collision-free and banks first; the cut is
two tickets — **delete the review feature (02)** then **revert to the vanilla
lifecycle (03)** — because the bulk deletion and the dangerous parser/semantics
change earn separate, independently-green commits; the skills repackaging (04)
follows the cut because it rewrites the same `internal/prompt` the cut just
shrank; and the transparency surface (05) renders the result of both. Within each
ticket, **land small, independently-green commits** (02: frontend, then backend;
03: the parser/semantics revert), never a big-bang — a stated discipline the
operator owns, backed by the build/test tripwire, not a new gate (the cut exits
the judgment business).

**The review feature and its semantics are split on purpose.** Ticket 02 deletes
the gate, the review UI, and the review role, and retargets `implement.md` to
`## Answer` so the map stays livable the instant the gate is gone — but leaves the
parser's `proposed` status dormant. Ticket 03 then retires `proposed` itself and
reverts `Frontier()` in one clean commit, so the **half-cut parser** — the most
dangerous state, since it misreads every ticket's status, this map's own included
— is isolated to a small ticket a stall cannot smear across the whole cut.

**Self-hosting is the binding constraint.** This repo drives itself, so every
intermediate commit must build, derive ticket status, and spawn. The escape hatch
is the cut's own destination — once the gate is gone (02), a terminal and `git`
finish any ticket the vanilla-wayfinder way. Note the lifecycle flips *underneath*
the map: tickets resolved before 02 lands still pass through the old gate (the
running binary is rebuilt between tickets); 02 is the last ticket gated that way,
and everything after it resolves by a bare `## Answer`.

**Before commit:** run the CLAUDE.md gates — `go vet ./...` and `go test ./...`
(the embed test compiles against `web/dist/`), the frontend `check` / `build`
scripts and `vitest`, and confirm **no amber in the built CSS**. Review the diff
(the `review-code` skill if available) and drive real behaviour (`run`/`verify`)
where a "Done when" is only real at runtime. No map linter is wired in this repo.

**The wayfinder-adapter step is already done for this map:** `[maps."simplify-impl"]
kind = "implementation"` is recorded in `.wayfinder-harness/config.toml`, committed
alongside these files (see `docs/wayfinder-adapter.md`).

## Decisions so far

<!-- one line per resolved ticket: gist + link. -->

- **The cockpit launches as a real native window.** A second cgo binary,
  [`cmd/webview`](tickets/01-the-webview-shell.md), runs the same server
  in-process on `127.0.0.1:0` and points a `webview/webview` window at it; the
  cgo hides behind `//go:build webview` so the default build stays green and
  goreleaser never sees it. Single-instance is a data-dir lock file keyed by pid
  (not `webview.Window()` — a window handle does not cross a process boundary);
  a missing runtime is a hard error pointing at `harness`, never a silent browser.
  Writes **ADR 0013**, confirms **ADR 0011** unamended.

- **The review feature is gone; the lifecycle is already livable.**
  [Ticket 02](tickets/02-delete-the-review-feature.md) cut the gate, its server
  mechanics, its UI and its role in two independently-green commits (frontend,
  then backend), shrinking the harness's lifecycle writes to **claim + release**
  and retargeting `implement.md` — and `core.md` with it — to `## Answer`, so a
  session resolves and its dependents unblock with no gate in the path. The
  parser's `proposed` is left **dormant** for ticket 03; **no ADR is touched
  here**.

- **Every injected prompt is a skill.**
  [Ticket 04](tickets/04-everything-is-a-skill.md) repackaged the library as
  seven `SKILL.md` directories (the four roles, `core`, `ideate`, and a new
  `tracker-convention` carrying the glossary), swapped the `replace`/`append`
  overlay for **whole-skill shadowing** across built-in ‹ user ‹ workspace, and
  moved fork provenance to `forked_from:` frontmatter with drift measured over
  the whole directory hash. The harness still composes the payload itself
  (**ADR 0002** reaffirmed); the claim's provenance trailers re-key to `Skill:`
  lines. The user skill layer is `~/.config/wayfinder-harness/skills/` via a new
  `Options.ConfigDir` — bindings still read `<dataDir>/user.toml`, a split
  ticket 05 must render. **ADR 0012** amended: this space's design-system
  overlay is now a committed workspace `implement` skill.

- **The config layers have a face, and one edit boundary.**
  [Ticket 05](tickets/05-the-transparency-surface.md) shipped the cockpit's
  first real route — a `#/settings` hash prefix disjoint from the star deep-link
  by construction — rendering every resolved value with its layer and its file
  from the pushed model (`Space.Skills`, `Space.Layers`, a new `Model.Config`
  for the layers no space owns). It edits **only role bindings, only into the
  user layer**, through a key-level comment-preserving TOML line editor; every
  other layer gets a server-named open-in-editor hatch. The route renders *over*
  the pane rather than replacing it — the terminal and star-map are islands
  worth keeping alive — with the pane inert underneath. Ticket 04's two-homed
  user layer is rendered honestly rather than papered over. Writes **ADR 0014**;
  **ADR 0009** amended with the edit boundary.

- **The dead weight is swept; three of the four items were already clean.**
  [Ticket 06](tickets/06-sweep-dead-weight.md) deleted the orphan `Probe.svelte`
  and the `#probe` hash swap in `main.ts` that was its only consumer — the sweep's
  one real diff. `sessions/`, the root `node_modules` and `.DS_Store` were already
  ignored at HEAD and tracked nowhere, so they were confirmed rather than changed:
  `sessions/` is *live* payload-audit state and stays on disk, and only the stray
  root Vite cache was removed. `make webview` untouched; **no ADR is touched**.

## Not yet specified

<!-- Empty. Every decision is settled in the spec; this map only executes it. A ticket that exposes a genuinely new question sends it back to the planning map — it does not open fog here. -->

## Out of scope

<!-- Inherited from the spec's Out of Scope; these never graduate into tickets on this map. -->

- **A plugin system, extension API, or hook framework** — the filesystem and git conventions are the seam; review returns only when a real second consumer earns an interface.
- **A TUI or true-native frontend** — the webview shell is the native answer; anything further waits for the "still feels wrong to daily-drive" trigger.
- **Redesigning the wayfinder method** — the `tracker-convention` skill restates the format; it does not change it.
- **Migrating or rewriting the old maps** — `harness-design`, `harness-design-impl`, `reskin` stay human-readable and un-migrated.
- **An ad-hoc preferences store** — refused on the transparency surface; a genuine preference earns a layer and a home when it actually appears.
- **A binding-editing UI beyond role bindings** — only bindings are inline; everything else is read-value-plus-open-file until a second setting earns editing.
- **Dock badge, `harness://` scheme, per-adapter native skill materialisation, a lightweight skill `append`** — each a named revisit trigger, not this effort's work.
