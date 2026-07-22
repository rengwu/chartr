---
type: grilling
blocked_by: [01, 02, 03, 04]
---

# Sequencing the work

## Question

Tickets 01–04 settle four mostly-independent decisions. This ticket turns them into an order — one implementation map or several, what lands first, and what "done" means for the effort as a whole. It inherits every answer above; if any of them is shaky, say which and how it changes the sequence.

The naive order is "cut first, then build" — ticket 01's deletion shrinks everything downstream (less UI for ticket 03 to explain, less payload machinery for ticket 02 to repackage, a smaller app for ticket 04 to wrap). The counter-pressures: the webview shell is the operator's oldest want and is almost entirely additive — it could land first without touching the cut; the skills repackaging (02) and the lifecycle cut (01) interact through the payload composer and the review prompt, so doing them in the wrong order means migrating code that is about to be deleted; and every intermediate state must leave a *working* chartr, because this repo drives itself — the cockpit is how the work gets done.

Settle:

- **The order.** Which ticket's work lands first, and why — with the dependencies named explicitly rather than gestured at. Where can implementation maps run in parallel (different file surfaces), and where would they collide (composer, config, cockpit screens)?
- **The map shape.** One implementation map for the whole effort, or one per decision? The repo's convention (`to-tickets` graduating a planning map) suggests one — argue whether that holds when the four decisions are this independent.
- **Self-hosting continuity.** At every point in the sequence, the chartr must still drive this repo's own maps. Which intermediate state is the most dangerous (lifecycle half-cut? payload composer mid-repackaging?), and what is the escape hatch if the cockpit breaks underneath its own work?
- **Done.** What the operator can *do* at the end that they cannot do today — stated as a short list of concrete capabilities, not virtues. Include what happens to the dead weight that is not part of any ticket (`Probe.svelte`, stale `sessions/` archives, the no-op `make webview` target, tracked `.DS_Store` files, the stray root `node_modules`): swept by a named ticket, or left to rot?

## Answer

**One implementation map, sequenced `04, 01 → 02 → 03` — the webview shell front-loaded as the first ticket, then a strict cut-first chain.** The four decisions graduate to a single implementation map because three of them co-edit the same two documents and a single map is the only thing that serialises those writes. `01 → 02 → 03` is a forced data dependency, not a chosen convenience: 01 and 02 rewrite the same composer, and 03's frontmatter already blocks on both. 04 is collision-free and lands first — the operator's oldest want, banked early, wired against server entry points the cut leaves untouched. The most dangerous intermediate state is a half-cut parser, and the escape hatch is ticket 01's own destination: once the gate is gone, finishing a ticket needs nothing but a terminal and `git`. Dead weight goes to one deletion-only housekeeping ticket — minus `make webview`, which is 04's scaffolding, not debris.

### The order

The graph has one chain and one free node placed first:

```
04  (webview shell) ── lands first, collision-free
     ↓
01 (cut) ──→ 02 (skills) ──→ 03 (transparency surface)
```

- **`01 → 02` is forced — they rewrite the same file.** 01 deletes `internal/prompt/compose.go`'s review branch and shrinks `binding.go` (autopilot + the `review` role); 02 then rewrites what remains of `compose.go` from `<part>.{replace,append}.md` resolution to whole-skill `SKILL.md` resolution. Reverse them and 02 lovingly ports the review-composition code and `review`-role wiring into the new skill resolver — code 01 then deletes. Cutting first means 02 rewrites a *smaller, review-free* composer exactly once. This is the ticket's "wrong order means migrating code about to be deleted," named at the file.
- **`02 → 03` is forced by 03's own frontmatter** (`blocked_by: [01, 02]`) and by content: 03's settings surface has no autopilot to toggle because 01 deleted it, and 03's one new read field (`Space.Skills` with winning layer + `forked_from`) *is* the provenance 02's resolver must already compute. 03 renders 01's smaller world and 02's resolver output; it cannot precede either. **There is no parallelism inside the chain to find** — it is a true dependency, not an ordering of convenience.
- **04 lands first, and it is genuinely safe there.** The shell is `cmd/webview` reusing the exact server entry points `cmd/chartr`'s `run()` already calls — `server.New(Options{…})` → `net.Listen("127.0.0.1:0")` → `srv.Serve(ctx, ln)`. 01 removes route *registrations* and the gate/review/promote internals but leaves `New` and `Serve` intact, so the shell wires against a surface the cut does not reshape. Front-loading therefore banks the operator's oldest want and validates the in-process-server model *before* the cut churns `internal/server`, at no rework cost. The one residual risk — 01 changing `New`/`Serve`'s shape after all — is bounded to two call sites and caught the moment `cmd/webview` fails to compile.

**Collision surfaces, stated as files** (what actually forces serialisation):

| Surface | Tickets | Consequence |
|---|---|---|
| `internal/prompt/compose.go`, `internal/prompt/` | 01, 02 | **serialise 01→02** (migrate-then-delete trap) |
| `internal/config/binding.go` | 01, 03 | serialised anyway (03 blocked by 01) |
| `web/src/lib/SpacePane.svelte`, `App.svelte` | 01, 03 | serialised anyway (03 blocked by 01) |
| `CONTEXT.md` | 01, 02, 03 | one map serialises; splitting races the edits |
| `docs/adr/0009` | 01, 02, 03 | one map serialises; splitting races the edits |
| `internal/server` (`New`/`Serve`) | 04 reads, 01 amends internals | no collision — cut leaves the entry points intact |
| `cmd/webview`, `release.yml`, `.goreleaser.yaml` | 04 only | fully independent |

### The map shape

**One implementation map** — and the deciding argument is the shared-document race, not the convention. 01, 02, and 03 all amend ADR 0009 and all edit the `CONTEXT.md` vocabulary; a single internally-ordered map is the only structure that gives those edits one serialised writer. The counter-cases were put under pressure and fail:

- **Rejected — split cut (01+02) from additive (03+04):** the seam doesn't fall cleanly, because 03 depends on 02. The two maps would not be independent, and the CONTEXT/ADR edits still cross the boundary — a split that buys the collision it was meant to avoid.
- **Rejected — four maps, one per decision:** maximally honest about independence, but it multiplies the CONTEXT/ADR write-race four ways and fragments one destination ("a leaner, more open chartr") into four frontiers and four `to-spec` runs. 04's real independence is already expressed by making it an *unblocked ticket*, which needs no separate map.

The `to-tickets` convention (one planning map graduates to one implementation map) points the same way; here it is reinforced by the file-level evidence, not merely followed.

### Self-hosting continuity

Every intermediate commit must leave a chartr that **builds, derives ticket status, and can spawn** — the CLAUDE.md gates (`go vet`/`go test`, frontend `check`/`build`/`vitest`) are the per-commit tripwire, load-bearing here because a red commit is a cockpit that cannot drive its own next ticket.

- **Most dangerous state: a half-cut parser (01 mid-flight).** `internal/wayfinder/parse.go` feeds the star-map, the frontier, and spawn eligibility. A half-done `StatusProposed` removal or `Frontier()` revert makes the cockpit misread the status of every ticket in every map — including this one — and it can no longer tell what its own frontier is. Strictly worse than the runner-up.
- **Runner-up: the composer mid-repackaging (02).** A `compose.go` caught between part-resolution and skill-resolution injects a broken or empty payload — a new session spawns with no role wiring. Bad but bounded: it breaks *new spawns*, not status derivation or live sessions, so the operator sees it on the next spawn rather than having the map silently misread.
- **The escape hatch is ticket 01's own destination.** Once the gate is gone the lifecycle is "write `## Answer`, commit" — which needs no cockpit: a terminal and `git` complete any ticket. So if the cockpit breaks *underneath its own work*, the operator finishes the offending ticket the vanilla-wayfinder way and the repair lands like any other commit. The chartr's own simplification is what makes it safe to rebuild the chartr. The concrete discipline: **land 01 and 02 as small, independently-green commits** (parser change, then composer change), never a big-bang — the `01 → 02` order already guarantees each is touched once, cleanly. This is a stated discipline, not an enforced gate: ticket 01 chose to trust the operator's git flow and exit the judgment business, and re-imposing a green-commit *gate* here would contradict that — so it is a recommendation the operator owns, backed by the build/test tripwire, not a new check.
- **04 does not de-risk self-hosting** — it is additive UI; front-loading it is a momentum choice. The safety comes entirely from ordering 01 before 02 and keeping each commit green. The supported cgo-free browser binary is always available, so 03's chrome breaking never strands the operator either.

### Done

Concrete capabilities the operator gains, not virtues:

1. **Resolve a ticket by writing `## Answer` and committing** — no review gate, no `proposed` state, no approval hop; resolved blockers unblock immediately (01).
2. **Read, edit, and reuse every injected prompt as a standard `SKILL.md`** on disk — the same skills usable in an agent CLI *outside* the chartr (02).
3. **See every resolved config value with its provenance layer and file location on one settings screen**, edit role bindings inline (user layer only, comment-preserving, reversible), and open any layer file in `$EDITOR` (03).
4. **Launch the cockpit as a real mac window** — dock icon, native menu, single-instance — instead of a browser tab, Linux best-effort behind it (04).

**Dead weight — one deletion-only `housekeeping` ticket**, isolated so the sweep never hides inside a functional diff, with the ticket's premises corrected:

- `web/src/lib/Probe.svelte` — orphan debug component; `git rm`.
- stale `sessions/` archives — remove from the tree or confirm gitignored (run-time debris, not source).
- stray root `node_modules` — present on disk but **already untracked** (0 tracked entries); delete from disk and confirm the root is ignored (deps belong under `web/`).
- `.DS_Store` — the ticket calls these "tracked," but `git ls-files` finds **none**; the honest task is a `.gitignore` entry, not a `git rm`.
- **`make webview` is *not* swept** — it is a deliberate no-op (`if [ -d ./cmd/webview ]`) that ticket 04 makes live by writing `cmd/webview`. Sweeping it would delete scaffolding one ticket from being real.

The sweep ticket stays strictly deletion-and-ignore; the moment it grows anything behavioural it is no longer housekeeping and earns a functional ticket.

### Revisit trigger

- **04-first turns out to need `internal/server` reshaped** in a way that collides with 01's cut (not anticipated — the shell reuses only `New`/`Serve`) → 04 gains an explicit ordering against 01 and stops being the free first node.
- **A half-cut parser or composer actually strands the cockpit in practice** → the "small green commits" discipline hardens from recommendation toward a checked pre-commit step, reopening ticket 01's "trust the git flow" stance — a new decision, not a silent tightening here.
- **The housekeeping sweep grows past its five items** into anything behavioural → it splits out a functional ticket rather than smuggling a refactor into a deletion diff.
