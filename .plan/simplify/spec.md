# simplify — a leaner, more open chartr (spec)

_Synthesised from the settled `simplify` planning map (tickets 01–05). This spec
graduates to one implementation map. It describes the chartr **after** the cut;
several CONTEXT.md terms die here (see Implementation Decisions → Vocabulary)._

## Problem Statement

The operator is driving real work through a chartr that has grown heavier than
the job needs, and less open than the project's own principles demand:

- **The review pipeline is load-bearing dead weight.** Every implementation
  ticket must pass through an agent-review session and a human-review hub before
  it resolves. For a single operator present by design — a cockpit, not an
  autopilot — the gate adds a `proposed` state, a review brief, promotion and
  demotion commits, and a whole vocabulary that stands between "I answered this"
  and "it's resolved." The operator wants to finish a ticket by writing the
  answer and committing, the way vanilla wayfinder already works.
- **The prompts are hackable in principle but not in the open standard.** Role
  prompts live as vendored `<part>.md` files with a bespoke
  `.replace.md`/`.append.md` layering convention invented before `SKILL.md` was a
  standard. The operator can't reuse them in an agent CLI outside the chartr,
  and the bespoke layering is one more thing to learn.
- **The configuration is opaque.** Role bindings resolve through three layers
  nobody can see; prompts materialise into a gitignored directory; map kinds live
  in a committed TOML file; there is no one place that answers "why is this value
  what it is, and where do I change it."
- **The cockpit lives in a browser tab.** It has no dock icon, no real window, no
  native menu — it doesn't feel like an app the operator can daily-drive.

## Solution

Cut the chartr down to its working core and open it up, in four moves that ship
as one sequenced implementation effort:

1. **Delete the review pipeline entirely.** The lifecycle becomes vanilla
   wayfinder: a session holds a ticket, writes its `## Answer`, commits its own
   work, and the ticket is **resolved** the instant the heading lands. Resolved
   blockers unblock their dependents immediately. The chartr exits the judgment
   business — no gate, no new deterministic check — and trusts the operator's git
   flow, keeping only the facts it already derives (death halt, dirty-tree badge,
   wayfinder lint).

2. **Repackage every injected prompt as a standard `SKILL.md`.** Role prompts,
   the core "how to use this chartr" prompt, and the tracker convention become
   vendored skill directories on disk — readable, editable, and reusable in any
   agent CLI that reads the standard. The chartr **keeps composing** the payload
   itself; only the source format and the layering change (whole-skill shadowing
   in place of bespoke replace/append).

3. **Ship a transparency surface.** A first-class global settings route — the
   cockpit's first real route — renders every space's resolved config with its
   provenance layer and file location, edits the one high-churn thing (role
   bindings) inline against the correct layer, and opens any other layer file in
   the operator's editor. Legibility first; it never becomes a second config
   store.

4. **Wrap the cockpit in a native webview shell.** A separate `cmd/webview`
   binary starts the existing server in-process on a random loopback port and
   points a `webview/webview` window at it — a real mac-first window with a
   minimal native menu and single-instance focus, best-effort behind the one
   supported cgo-free browser binary.

The four land in one implementation map, sequenced `04, 01 → 02 → 03`: the
additive shell first (collision-free), then a strict cut-first chain because the
lifecycle cut and the skill repackaging rewrite the same composer and the
transparency surface renders the result of both.

## User Stories

### The lifecycle after review (ticket 01)

1. As an operator, I want to resolve a ticket by writing its `## Answer` and
   committing, so that finishing work needs nothing but a terminal and git.
2. As an operator, I want a ticket to become resolved the instant its `## Answer`
   heading lands, so that there is no approval hop between answering and done.
3. As an operator, I want a resolved blocker to unblock its dependents
   immediately — even while the resolving session is still committing — so that
   the frontier advances at the speed of the work.
4. As an operator, I want the four statuses open / claimed / resolved /
   out_of_scope and nothing else, so that the status model is exactly wayfinder's
   with no chartr-specific `proposed` state to reason about.
5. As an operator, I want the agent to write the `## Answer` by prompt convention
   (never the chartr mechanically), so that the discipline "the agent writes,
   the chartr watches" survives the cut.
6. As an operator, I want the chartr's own commits to stay append-only,
   pathspec-limited, and trailer-carrying, shrunk to just the claim (at spawn)
   and the release (at death-halt), so that the audit trail stays honest without
   the promotion/demotion writes the gate needed.
7. As an operator, I want a dead session that exited without an `## Answer` to
   remain visibly pinned to its ticket, so that an unfinished ticket is surfaced
   by the death halt without any new "you didn't answer" check.
8. As an operator, I want uncommitted debris still surfaced as a dirty-tree badge
   and never auto-cleaned, so that the chartr keeps showing facts without
   enforcing them.
9. As an operator, I want no blocking lint-before-resolution check of any kind,
   so that no gate returns under another name.
10. As a future review feature, I want to rebuild against documented conventions
    — file-derived ticket status, fsnotify on `.plan/`, git trailers on lifecycle
    commits, the run-dir layout, in-process session-exit — so that review can
    return as a consumer without touching chartr internals.
11. As an operator, I want the chartr to emit nothing purely for a hypothetical
    reviewer that does not exist, so that no zero-consumer artifact is carried as
    speculative bloat.
12. As an operator, I want old maps' `## Proposed Answer` sections treated as
    unknown headings — ignored, not tolerated as answers, not migrated — so that
    a dead state does not live on in every future reader's head.
13. As an operator, I want the review vocabulary (proposed, agent review, human
    review, review brief, abandon, autopilot) removed from CONTEXT.md and the
    ADRs, so that the language matches the system.
14. As an operator, I want the containment consequence knowingly forfeited with a
    named revisit trigger, so that if a still-live resolution seeds real wreckage
    in practice, the `resolved AND unclaimed` frontier rule returns as a new
    ticket rather than a silent tightening.

### Everything is a skill (ticket 02)

15. As an operator, I want each role prompt as a standard `SKILL.md` directory on
    disk, so that I can read, edit, and reuse it in any agent CLI outside the
    chartr.
16. As an operator, I want seven skills to ship — `grill`, `prototype`,
    `research`, `implement`, `ideate`, plus `core` and `tracker-convention` — so
    that the whole injected library is the open standard.
17. As an operator, I want the glossary to live inside the `tracker-convention`
    skill as a supporting file, so that the method vocabulary sits with the
    convention it defines rather than fragmented across skills.
18. As an operator, I want the chartr to keep composing the payload — reading the
    resolved `core` and role `SKILL.md` bodies and assembling them with a
    freshly-built context bundle into one gitignored payload file — so that every
    session is deterministically wired to its role and context regardless of what
    the agent does next.
19. As an operator, I want the one-line opener into the PTY unchanged (read the
    payload file and proceed), so that the injection path stays identical across
    every agent adapter.
20. As an operator, I want skill layers to combine by whole-skill shadowing — the
    most-specific layer defining a skill wins its entire directory — so that
    resolution is simple and there is no per-file merge to reason about.
21. As an operator, I want the bespoke `.replace.md`/`.append.md` convention
    dropped, so that pre-standard machinery is gone.
22. As an operator, I want fork provenance recorded as a `forked_from:` field in a
    shadowing skill's frontmatter, so that provenance rides the standard rather
    than an HTML comment the composer must peel.
23. As an operator, I want a stale-fork warning when my shadowing skill's
    `forked_from` hash differs from the built-in's current content hash, so that
    the cockpit tells me my copy has drifted behind the shipped default — never
    auto-merging it.
24. As an operator, I want a skill's content hash computed over its `SKILL.md`
    plus supporting files in a stable order, so that a change to a supporting file
    is not invisible to drift detection.
25. As an operator, I want the context bundle (map body, ticket, blockers'
    answers, glossary) to stay a composed payload section, not a skill, so that
    the ticket I was handed is never mistaken for durable skill content.
26. As an operator, I want a skill's supporting files left on disk (not inlined
    into the payload), so that the agent can zoom into them on demand and I can
    reuse them outside the chartr at no payload cost.
27. As an operator, I want `SKILL.md` frontmatter (name / description /
    forked_from) stripped before the body reaches the payload, so that metadata
    drives the cockpit listing and drift detection but never leaks into what the
    agent is told.
28. As an operator, I want the skills kept vendored from the upstream skills repo
    with the upstream commit recorded per sync, so that the hackable on-disk
    surface exists after first run and drift is visible.
29. As an operator, I want the `ideate` skill to keep its special injection
    (composed alone, no core, no context bundle), so that a ticketless, mapless
    ideate session is unaffected by becoming a skill directory.
30. As an operator, I want provenance trailers on the claim commit re-keyed from
    parts to skills — recording which layer won each composed part and the content
    hash — so that the audit trail survives the format change.

### The transparency surface (ticket 03)

31. As an operator, I want a single global settings screen listing every space
    and the one global user file, so that config has one home rather than a drawer
    bolted to each space.
32. As an operator, I want the settings screen reached by a `#/settings` hash
    route (with `/s=<spaceId>` and `/user` sub-paths), a ⚙ button in the sidebar
    header, and the `,` key — and dismissed with Esc or by selecting a space — so
    that it is a real route without a routing dependency.
33. As an operator, I want each role's effective `{adapter, model, args?}` shown
    with per-field provenance (built-in / workspace / user) and PATH-probe status,
    so that I can see exactly where each binding value came from and whether the
    binary is present.
34. As an operator, I want each resolved skill shown with the layer that won its
    whole directory and its stale-fork state, so that the positive "your `grill`
    resolves from: user" is visible, not just the warnings.
35. As an operator, I want map kinds shown read-only with a link to the classify
    action, so that kind stays edited only through the deliberate,
    human-confirmed committed path.
36. As an operator, I want the concrete file path of each participating layer
    shown, so that legibility includes where each layer lives and the open hatch
    has the path it needs.
37. As an operator, I want the resolver's warnings (malformed file, unknown role,
    unrecognised kind) surfaced on the screen, so that config problems are visible
    where I read config.
38. As an operator, I want the surface to link to the existing payload preview for
    a prospective (role, ticket), so that I can see the assembly without the
    screen rebuilding it.
39. As an operator, I want to edit a role binding inline, so that I can change the
    one high-churn setting without opening a file.
40. As an operator, I want a binding edit written only to the user layer
    (`~/.config/chartr/config.toml`), never to committed workspace
    config, so that a local UI cannot silently override shared content it labels
    "workspace."
41. As an operator, I want binding writes to be surgical and comment-preserving —
    setting or clearing the specific adapter/model/args key within the target role
    table and leaving every surrounding byte intact — so that my formatting and
    comments survive edits.
42. As an operator, I want clearing a field's override to reveal the layer beneath
    it, with the provenance badge flipping back, so that editing is reversible and
    never a one-way ratchet.
43. As an operator, I want the model to re-derive after a write (the same rebuild
    the classify handler triggers), so that the edited value and its new
    provenance reflect straight back with no optimistic client state.
44. As an operator, I want an "open the file" action on each layer row that
    launches `$VISUAL`/`$EDITOR`, falling back to the OS opener and finally to
    showing the absolute path, so that anything not inline-editable still has an
    escape hatch.
45. As an operator, I want the open action to resolve the named layer file
    server-side (workspace / user / a named skill dir), never a client-supplied
    path, so that a local server never opens arbitrary paths on request.
46. As an operator, I want provenance badges to be the explanation in place, plus
    a one-line layering caption and "how resolution works →" links into the
    `core`/`tracker-convention` skills and ADR 0009, so that the surface explains
    the *why* without restating the whole method.
47. As an operator, I want no invented ad-hoc preferences table, so that the
    settings pane does not grow a second config system; the surface shows only
    what the three documented layers resolve.
48. As an operator, I want runtime session state and secrets deliberately absent
    from the surface, so that config stays config and no credentials box becomes a
    new attack surface.
49. As an operator, I want the existing per-space bindings Sheet folded into this
    route (the "bindings" button navigates to `#/settings/s=<space>`), so that
    there is one config home, not two.

### The webview shell (ticket 04)

50. As an operator, I want to launch the cockpit as a real mac window with a dock
    icon, so that it feels like an app rather than a browser tab.
51. As an operator, I want the shell as a separate `cmd/webview` binary built with
    cgo, so that the supported `chartr` binary stays pure-Go and cgo-free.
52. As an operator, I want the shell to start the existing server in-process on a
    random `127.0.0.1:0` loopback port and point the window at it, so that there
    is one process, no fixed port to collide on, and the server dies with the
    window.
53. As an operator, I want a second launch to focus the existing window rather
    than open a duplicate, via a data-dir lock file recording the live instance's
    URL, so that the one-window invariant holds.
54. As an operator, I want the focus to degrade to a "shell already running at
    <url>" refuse-with-message where raising is flaky, so that one-window is
    honoured without pretending a raise worked.
55. As an operator, I want distinct `--data-dir` roots treated as distinct
    instances, so that the lock is keyed to the data dir by construction.
56. As an operator, I want a minimal native menu — Quit (⌘Q), Reload (⌘R), and the
    standard edit items — so that a bare webview window regains the OS affordances
    a browser tab gave for free.
57. As an operator, I want no dock badge and no `chartr://` URL scheme in the
    shell, so that bespoke per-platform integration for signals that already live
    in the chrome (or producers that don't exist) is not carried; each returns
    only on a concrete need.
58. As an operator, I want a missing native runtime to produce a hard error naming
    exactly what is missing and pointing at the supported browser binary, so that
    a missing dependency is never papered over with a silent browser launch.
59. As an operator, I want no `--browser`/`--shell` force flag, so that the two
    binaries express the choice (`chartr` for the browser, `webview` for the
    shell) without impersonating each other.
60. As a release engineer, I want the shell built off the same tag with the same
    version/commit stamp but kept out of the supported `checksums.txt` (a per-asset
    `.sha256` sidecar instead), so that a best-effort asset never mutates the
    supported manifest.
61. As a release engineer, I want the shell's cgo behind `//go:build webview` with
    a cgo-free stub for the default build, so that `go vet`/`go test`/`go build`
    at `CGO_ENABLED=0` and the embed test stay green and goreleaser (building only
    `./cmd/chartr`) never sees it.
62. As a release engineer, I want the `shells` CI matrix to run after the supported
    release with `continue-on-error` and `fail-fast: false`, so that any shell
    build failing never fails the supported release.

### Sequencing, self-hosting, and housekeeping (ticket 05)

63. As an operator, I want the four decisions delivered as one implementation map,
    so that the shared CONTEXT.md and ADR-0009 edits have one serialised writer.
64. As an operator, I want the order `04, 01 → 02 → 03`, so that the additive
    shell banks early and the cut precedes the composer repackaging it shrinks.
65. As an operator, I want 01 landed before 02, so that 02 rewrites a smaller,
    review-free composer once instead of porting review code 01 then deletes.
66. As an operator, I want each cut commit small and independently green (parser
    change, then composer change) with the CLAUDE.md build/test gates as the
    per-commit tripwire, so that every intermediate state leaves a chartr that
    builds, derives status, and can spawn.
67. As an operator, I want the green-commit discipline to stay a recommendation I
    own (not a new enforced gate), so that it does not contradict ticket 01's
    choice to trust the git flow.
68. As an operator, I want the escape hatch to be ticket 01's own destination — if
    the cockpit breaks under its own work, I finish the offending ticket with a
    terminal and git, vanilla-wayfinder style — so that the simplification is what
    makes it safe to rebuild the chartr.
69. As an operator, I want dead weight swept in one deletion-only housekeeping
    ticket (`Probe.svelte`, stale `sessions/` archives, the stray root
    `node_modules`, a `.DS_Store` gitignore entry), isolated so the sweep never
    hides inside a functional diff.
70. As an operator, I want `make webview` left in place (not swept), so that the
    scaffolding ticket 04 makes live is not deleted one ticket from being real.
71. As an operator, I want the housekeeping ticket to stay strictly
    deletion-and-ignore, so that the moment it grows anything behavioural it earns
    a functional ticket instead.

## Implementation Decisions

### Vocabulary (CONTEXT.md)

- **Terms that die** (ticket 01): `proposed`, `agent review`, `human review`,
  `review brief`, `abandon`, `autopilot`, and the stricter-than-wayfinder
  `frontier` definition. `resolved` is redefined from "on disk, resolved always
  means blessed" to "the session said so." `implementing` survives.
- **Prompt library → Skill library** (ticket 02): the term is renamed; its
  `_Avoid_: skills` line inverts (it now *is* skills). Workspace/User config
  entries' "prompts" content-half references become "skills."
- **New term: Effective config surface** (ticket 03): the global screen showing
  every resolved value with its provenance and file location.
- **Frontier** reverts to wayfinder's own: open, unblocked, unclaimed; a blocker
  need only be resolved (no approval, no unclaimed condition).

### Ticket 01 — the lifecycle cut

- **Status model**: four statuses — open, claimed, resolved, out_of_scope. The
  parser's derived-status table becomes exactly wayfinder's; the chartr's "one
  addition" is withdrawn.
- **`claimed → resolved`** requires an `## Answer` heading with prose, written by
  the agent by prompt convention. The chartr never writes the Answer. The commit
  is the agent's act by convention.
- **Unblocking** is immediate on resolution, even mid-commit. Mitigation is
  social (visible claim on the star-map, operator present), not mechanical.
- **Chartr writes** stay append-only, pathspec-limited, trailer-carrying, shrunk
  to two: claim (spawn) and release (death-halt). `Chartr-Write: true` stays.
- **No new deterministic check.** Surviving surfaces: death halt, dirty-tree
  badge, wayfinder lint. Rejected: an explicit "ended without an Answer" item
  (duplicates the death halt) and any blocking pre-resolution lint (a gate by
  another name).
- **Extension seam** = five documented conventions, each with a live consumer
  today: file-derived ticket status, fsnotify on `.plan/`, git trailers on
  lifecycle commits, the run-dir layout, in-process session exit. Nothing new
  emitted for a hypothetical external reviewer (rejected: an `exit.json`
  session-end record).
- **In-flight wreckage**: ignored. `## Proposed Answer` becomes an unknown heading
  (derives open, or claimed if a claim marker survives); no migration.

### Ticket 02 — everything is a skill

- **Seven skills**: `grill`, `prototype`, `research`, `implement`, `ideate`,
  `core`, `tracker-convention`. The glossary is a supporting file inside
  `tracker-convention`. `review` is deleted (ticket 01), not kept as an example.
- **Composition retained**: the chartr reads resolved `core` + role `SKILL.md`
  bodies, assembles them with the context bundle into one payload in `run/<sid>/`,
  opener unchanged. Reaffirms ADR 0002 (the chartr leans on no agent's skill
  mechanism). Rejected: materialising into each CLI's native skills path
  (per-adapter, non-deterministic, reverses ADR 0002); point-and-read.
- **Three layers, whole-skill shadowing**: built-in (`<dataDir>/skills/`) ‹ user
  (`~/.config/chartr/skills/`) ‹ workspace
  (`<space>/.chartr/skills/`). Most-specific layer defining a skill
  wins its whole directory. The `.replace.md`/`.append.md` convention is dropped;
  the append affordance is knowingly given up (revisit: a single per-skill
  house-rules file if forking-for-one-line churn bites in practice).
- **Fork provenance** moves to a `forked_from:` frontmatter field; the content
  hash covers `SKILL.md` + supporting files in a stable order; the stale-fork
  warning survives, never auto-merged.
- **Frontmatter** (name / description / forked_from) is stripped before the body
  reaches the payload. Supporting files stay on disk (not inlined).
- **Keep vendoring** from the upstream skills repo, recording the upstream commit
  per sync. Rejected: pointing at an external checkout; shipping only the
  convention (a bare install would spawn sessions with no role wiring).

### Ticket 03 — the transparency surface

- **A global settings route** — the cockpit's first real route — via a `#/settings`
  hash prefix in `App.svelte` (`#/settings/s=<spaceId>`, `#/settings/user`),
  ~15 lines, no routing library. The bare star deep-link (`#s=…`, no `/` prefix)
  is untouched, so the schemes never collide. Entry: ⚙ sidebar button + `,` key;
  exit: Esc or selecting a space. The per-space bindings Sheet's button rewires to
  navigate here. Rejected: both per-space and global; a routing library.
- **Read path** rides the existing per-space model push. `Server.deriveSpace`
  already folds `Bindings` (full provenance + PATH probe), `Kinds`, and
  `Warnings` into `model.Space`. One new derived field: `Space.Skills
  []ResolvedSkill` (name → winning `Layer`, plus `forked_from`/stale), computed in
  `deriveSpace` next to the bindings loop from ticket 02's resolver. The payload
  preview stays a separate on-demand fetch.
- **Edit boundary**: only role bindings are inline-editable, and only into the
  user layer (`[spaces."<path>".roles.<role>]`) — because bindings resolve
  user-over-workspace (ADR 0009), the user layer *is* the correct home. The write
  is key-level and comment-preserving (harder than `DeclareMapKind`, which only
  appends to an absent slug): set/clear the specific adapter/model/args key,
  create the table if absent in `DeclareMapKind`'s style, leave surrounding bytes
  intact. Clearing an override reveals the layer beneath (reversible). After a
  write, the model re-derives.
- **Never written by the UI**: committed workspace config for execution values;
  map kind (stays classify-only, ADR 0007); no autopilot toggle (deleted by 01).
- **Open-in-editor hatch**: `POST …/config/open` launches `$VISUAL`/`$EDITOR` →
  OS opener → surface the absolute path; resolves a *named* layer file
  server-side, never a client path. Works identically under the webview shell
  (local server either way).
- **Explanation in place**: provenance badges + a one-line layering caption +
  "how resolution works →" links into the skills and ADR 0009. Rejected: an
  in-app diagram/tutorial (a docs problem that rots against the canonical skills).
- **No preferences store.** With autopilot gone there is no ad-hoc preference
  left; inventing a KV table is the "second config system" the ticket refuses.

### Ticket 04 — the webview shell

- **Separate `cmd/webview` binary**, built `CGO_ENABLED=1 -tags webview`, where
  the dead `make webview` target already points. Rejected: a `chartr --webview`
  flag (would put cgo in the supported binary).
- **In-process server**: `cmd/webview` imports `internal/server`, does what
  `cmd/chartr`'s `run()` does but binds `127.0.0.1:0`, reads the OS-assigned port
  off `ln.Addr()`, hands `http://<that>` to the webview. The server dies with the
  window (closing it cancels the same context `signal.NotifyContext` cancels
  today). Rejected: spawning the supported binary as a child.
- **Single-instance** via a data-dir lock file (`.chartr/shell.lock`)
  recording the live instance's loopback URL. A second launch focuses the running
  window through the native handle `webview.Window()` exposes; degrades to
  refuse-with-message where raising is flaky. Keyed to the data dir.
- **Library**: `webview/webview` (zserge) — WKWebView (mac) / WebKitGTK (Linux) /
  cgo-free go-webview2 (Windows), matching ADR 0011's named backends. Rejected:
  Wails (a second IPC layer for a UI that already talks HTTP/websockets).
- **Native integration in scope**: a real window + dock icon; a minimal native
  menu (Quit ⌘Q, Reload ⌘R, standard edit items); single-instance focus.
  **Declined**: a dock badge for the "Needs you" queue; a `chartr://` scheme —
  each returns only on a concrete trigger.
- **Release**: the supported lane (`.goreleaser.yaml`, `./cmd/chartr`,
  `CGO_ENABLED=0`, owns `checksums.txt`) is untouched. The shell rides a
  `continue-on-error`, `needs: release`, `fail-fast: false` `shells` matrix that
  runs after the supported release and uploads a per-asset `.sha256` sidecar.
  Same version/commit stamp, separate checksum. `cmd/webview` ships
  `main_webview.go` (`//go:build webview`) and `main_stub.go`
  (`//go:build !webview`, prints a message and exits non-zero) so the cgo-free
  wildcard stays green.
- **Fallback**: hard error naming the missing runtime and pointing at the
  supported browser binary. Rejected: auto browser fallback; a force flag.
- **Writes ADR 0013**; confirms ADR 0011 unamended.

### Ticket 05 — sequencing

- **One implementation map**, ordered `04, 01 → 02 → 03`. `01 → 02` is forced
  (both rewrite `internal/prompt/compose.go`); `02 → 03` is forced by 03's
  frontmatter and by 03 rendering 02's resolver output. 04 is collision-free
  (reuses `server.New`/`Serve`, which the cut leaves intact) and lands first.
  Rejected: split maps; four maps.
- **Dead weight**: one deletion-only `housekeeping` ticket — `Probe.svelte`
  (`git rm`), stale `sessions/` archives (remove or confirm gitignored), the
  stray root `node_modules` (already untracked — delete from disk, confirm
  ignored), `.DS_Store` (a `.gitignore` entry — none are tracked). `make webview`
  is **not** swept (04's scaffolding).

### ADR changes

- **Amended**: 0004 (derived state survives; the `## Proposed Answer` extension,
  promotion-at-gate, and stricter frontier withdrawn; containment forfeited),
  0008 (write set shrinks to claim + release; promotion/demotion removed), 0009
  (mechanism untouched; autopilot bullets gone via 01; the transparency surface
  named as its surface, and the edit-boundary rule recorded).
- **Reaffirmed**: 0002 (agent-agnostic composition survives 02).
- **Survives with one premise struck**: 0007 (kind no longer selects a lifecycle
  — both kinds now share the vanilla one — but still selects the role set).
- **New**: 0013 (webview shell architecture), 0014 (the transparency surface).
- **Confirmed unamended**: 0011 (tiering).

### Deletions (file-level, from ticket 01 unless noted)

- `internal/server/gate.go`, `review.go`, `promote.go` (relocating still-used
  helpers to surviving callers first); the review/gate/proposed tests
  (`proposed_test.go`, `review_test.go`, `gate_test.go`, `gate_edges_test.go`);
  review stubs in `internal/chartrtest/`.
- `web/src/lib/ReviewHub.svelte` and its `actions.ts`/`model.ts` review surface.
- `prompts/review.md` and `internal/prompt/assets/review.md`.
- Amendments across `internal/wayfinder/parse.go` (drop `StatusProposed`,
  proposed-answer derivation; revert `Frontier()`), `internal/config/binding.go`
  (drop `RoleReview`, autopilot resolution), the star-map (`session.ts`,
  `theme.ts`, `starmap.ts`), the chrome (`attention.ts`, `NeedsYouQueue.svelte`,
  `ActionStation.svelte`, `App.svelte`, `MapCard.svelte`, `DetailPane.svelte`,
  `MapPickerCard.svelte`, `SpacePane.svelte`), the prompts, `CONTEXT.md`,
  `docs/design-system.md`, and config samples.

## Testing Decisions

**What makes a good test here**: assert observable behaviour at a public seam —
derived status/frontier from markdown, the resolved winning layer and composed
payload body from layered skill dirs, the pushed model's config fields, the bytes
a binding write leaves in a TOML file — never the internal shape of a parser
struct or a Svelte component's private state. Prefer the existing seams over new
ones; each subsystem gets one seam at its highest observable point.

- **Ticket 01 — `internal/wayfinder` parser.** Feed map/ticket markdown to
  `ParseMap`/`ParseTicket`, assert `Ticket.Derive()` yields only open / claimed /
  resolved / out_of_scope, that `## Proposed Answer` derives as an unknown heading
  (open/claimed, never resolved), and that `Effort.Frontier()` unblocks a
  dependent the moment its blocker is resolved. Prior art: `wayfinder_test.go`
  (the deleted `proposed_test.go` is the inverse of what to assert now — resolved
  never means "proposed"). Server-side, assert the lifecycle-write set shrinks to
  claim + release via the chartrtest rig's commit inspection (prior art:
  `spawn_test.go`, `halt_test.go`, `claim.go`'s trailer tests).

- **Ticket 02 — `internal/prompt` composer.** Feed layered skill directories
  (built-in / user / workspace) and assert whole-skill shadowing picks the
  most-specific layer's whole directory; assert the composed payload contains the
  resolved `core` + role bodies with frontmatter stripped and the context bundle
  appended; assert `forked_from` drift is detected over the directory hash
  (SKILL.md + supporting files, stable order). Prior art: composition is exercised
  today through the server payload preview (`payload_test.go`); this ticket adds a
  focused `internal/prompt` unit seam for resolution + assembly.

- **Ticket 03 — read via `Server.deriveSpace`, write via `internal/config`.**
  Read: through the chartrtest rig, assert the pushed `model.Space` carries
  bindings with per-field provenance + PATH probe, the new `Space.Skills` with
  winning layer + stale-fork state, kinds, and warnings (prior art:
  `spaces_test.go`, `maps_test.go`). Write: unit-test the user-layer binding
  writer directly — set/clear a key, assert the specific value changes while
  comments, ordering, and unrelated tables are byte-preserved; assert clearing an
  override reveals the layer beneath; assert it never writes workspace config
  (prior art: `config/kinds.go` + `classify_test.go`, whose comment-preserving
  append is the closest existing pattern). The editor-launch handler is tested for
  server-side path resolution (named layer only; a client-supplied path is
  refused).

- **Ticket 04 — `cmd/webview`.** Unit-test the single-instance lockfile logic
  (write/read the loopback URL, detect a held lock, key by data dir) without a
  real window. Assert the `//go:build !webview` stub compiles and the package
  builds green at `CGO_ENABLED=0` (the embed test already guards `dist/`). The
  in-process server reuse needs no new test — it calls the same
  `server.New`/`Serve` the existing server tests cover. The cgo shell itself and
  native menu are best-effort tier: verified by building the tagged binary in the
  `shells` CI matrix, not by unit tests.

- **Cross-cutting**: the CLAUDE.md gates — `go vet ./...` / `go test ./...`,
  frontend `check` / `build` / `vitest`, and "no amber in the built CSS" — are the
  per-commit tripwire that keeps every intermediate state a working,
  self-hosting chartr.

## Out of Scope

- **A plugin system, extension API, or hook framework.** The filesystem and git
  conventions are the seam; a future review feature earns an interface only when a
  real second consumer arrives.
- **A TUI or true-native frontend.** The webview shell is the native answer;
  anything further waits for the "still feels wrong to daily-drive" trigger.
- **Redesigning the wayfinder method.** The `tracker-convention` skill restates
  the format; it does not change it.
- **Migrating or rewriting the old maps.** `chartr-design`,
  `chartr-design-impl`, and `reskin` are history — human-readable, un-migrated.
- **An ad-hoc preferences store.** Refused in ticket 03; a genuine preference
  earns a layer and a home when it actually appears.
- **A real binding *editing* UI beyond role bindings.** Only bindings are inline;
  everything else is read-value-plus-open-file until a second setting earns
  editing.
- **Dock badge, `chartr://` scheme, per-adapter native skill materialisation,
  and a lightweight skill `append` affordance.** Each is a named revisit trigger,
  not this effort's work.

## Further Notes

- **The map produced decisions, not code; this spec is their synthesis.** Every
  decision above is settled in `.plan/simplify/` tickets 01–05 with named revisit
  triggers; nothing here reopens them.
- **Self-hosting is the binding constraint on sequencing.** This repo drives
  itself, so every intermediate commit must build, derive ticket status, and
  spawn. The most dangerous state is a half-cut parser (misreads every ticket's
  status, including this map's own); the runner-up is a composer mid-repackaging
  (breaks new spawns only). The escape hatch is ticket 01's destination: once the
  gate is gone, a terminal and git finish any ticket.
- **Revisit triggers to carry into the implementation map**: containment biting
  (→ `resolved AND unclaimed` frontier rule returns); a real review consumer
  appearing (→ earns an interface); the skill `append` loss biting (→ a single
  house-rules file); a second editable setting or the global route felt as a
  detour (→ inline editor / read-only hover-card); the shell still feeling wrong
  (→ a TUI companion); single-instance raise unreliable (→ refuse-with-message +
  polish ticket); a dock badge missed or a `chartr://` producer appearing.
- **The wayfinder-adapter step still applies** (CLAUDE.md): when `to-tickets`
  charts the implementation map, record its kind (`implementation`) in
  `.chartr/config.toml` keyed by slug, and commit it with the map.
