# chartr — spec

Synthesized from the `chartr-design` planning map (14 resolved tickets), the ADRs in `docs/adr/`, and the `CONTEXT.md` glossary, whose vocabulary this spec uses throughout. Where this spec and an ADR appear to disagree, the ADR wins and this spec has a bug.

## Problem Statement

An operator driving wayfinder maps today does it by hand: they open a terminal per project, remember which effort is at what state, re-read `map.md` to find the frontier, paste role prompts into agent CLIs themselves, watch each session by eye, and gate nothing — an agent that writes an `## Answer` has resolved a ticket whether or not anyone checked the work. Driving several projects at once multiplies the tabs, the pasted prompts, and the places attention leaks. There is no cockpit: no one surface that shows which spaces are cooking, which tickets are at the frontier, which proposed work is waiting on a human, and no deterministic machinery making the review gate — the one containment against a wrong ticket seeding its dependents — actually hold.

## Solution

**chartr**: a cross-platform, agent-agnostic cockpit that drives wayfinder maps to completion. The operator registers project spaces and switches between them; each space's maps render as a star-map; frontier tickets spawn prompt-injected agent sessions in real interactive TUIs; and on implementation maps, work passes through agent review and a human review hub before it resolves. chartr is a **cockpit, not an autopilot**: a human drives, deterministic code makes the driving safe, and an agent sits only where judgment is the product. Everything chartr injects and reasons with — prompts above all — is **hackable**: plain markdown on disk, visible and editable, never sealed in the binary.

Concretely, it is one Go binary serving a browser frontend: a Svelte chrome around two imperative islands (xterm.js terminals, a canvas star-map), a spaces sidebar, a per-map action station, and a review brief — all fed by server-pushed state derived live from each space's `.plan/` and git history.

## User Stories

### Spaces and the registry

1. As an operator, I want to register a folder as a space, so that chartr can drive its maps.
2. As an operator, I want registering a non-repo folder to run `git init` announced and never silently, so that an empty folder is no obstacle and I always know when history begins.
3. As an operator, I want the registry to be a rebuildable index rather than a source of truth, so that losing it costs me re-adding folders, never work.
4. As an operator, I want to de-register a space without chartr touching anything in the repository, so that removal is forget, not destroy.
5. As an operator, I want de-registering a space with a live session to reclaim only the chartr-owned process and leave every byte it wrote, so that chartr owns the process and I own the tree.
6. As an operator, I want pinned spaces first and the rest ordered by recency in a flat list, so that five to twenty spaces stay legible without sections.
7. As an operator, I want an always-present filter box over the space list, so that the list scales past what a flat list carries.
8. As an operator, I want an actionable signal to flag a row but never re-sort it, so that muscle memory over the sidebar holds.
9. As a first-run user, I want the empty registry to be the first-run screen with a single "register your first space" affordance, so that onboarding is the product's own front door and not a wizard.
10. As an operator, I want switching spaces to swap the view and never pause, kill, or checkpoint a session, so that the space I leave keeps cooking.

### Maps and discovery

11. As an operator, I want chartr to notice a new map under a space's `.plan/` — from a hosted shell, an external terminal, or a `git pull` — without a refresh button, so that every map enters by the same adoption path.
12. As an operator, I want map discovery to read wherever wayfinder writes — today `.plan/<slug>/`, tolerating the move to `.plan/maps/<slug>/` — so that chartr follows the convention rather than hard-coding a layout.
13. *Struck by the `kind-cut` map, which removed map kind entirely. This story asked for a newly discovered map to be inert until classified, so that no lifecycle ever ran on a heuristic. There is no lifecycle to gate — the two kinds resolved identically once review was cut — and a session's role now comes from the ticket's own `type:`, which states it exactly rather than by map-uniform approximation. A discovered map is live: readable, rendered, and spawnable the moment it is found.*
14. *Struck by the `kind-cut` map. This story asked for the kind guess pre-filled from convention (the `-impl` suffix, all-`task` tickets) for a one-keystroke confirm. With nothing to declare there is nothing to guess, and the conventions it read are gone from the code with `GuessKind`.*
15. *Struck by the `kind-cut` map. This story asked for map kind in committed workspace config so a teammate's chartr and mine agreed without re-classifying. The agreement it bought was over which maps gate review, and review is gone; the per-ticket `type:` that replaced kind already rides the map itself, so a clone inherits it with no chartr config at all.*
16. As an operator, I want chartr to notice when `to-tickets` produces an implementation map, so that graduation is observed rather than orchestrated. *(Restated by the `kind-cut` map, which struck this story's other half — surfacing the new map for classification. Noticing is the whole of it now: the map appears and is live.)*
17. As an operator, I want a malformed map (dangling `blocked_by`, unparseable ticket) rendered as-is with the malformation surfaced where it bites, so that adoption is never gated on lint.
18. As a vanilla-wayfinder user, I want chartr to leave the map format untouched apart from the one `## Proposed Answer` heading, so that my tools read the same map unchanged.

### The cockpit and the star-map

19. As an operator, I want a full-width terminal column with the space's live session and past sessions as tabs, so that the surface where the hours go owns the screen.
20. As an operator, I want the star-map as a floating card summoned over the terminal (with an operator-toggled terminal-priority split as the alternative), so that consulting the map never reflows my terminal by default.
21. As an operator, I want map visibility to change only on my explicit acts — never on switching spaces or maps — so that the cockpit doesn't rearrange itself under me.
22. As an operator, I want clicking a star to open the full ticket — question, Done-when, blockers with answers inline, session history — as a responsive pane (right dock, bottom dock when narrow) with the camera easing the star into the free space, so that reading a ticket is one click from seeing it.
23. As an operator, I want a "Next up" action station on the map card — reviews waiting first, then spawnable frontier tickets ranked by unblock count — so that the most consequential next act is never hidden.
24. As an operator, I want the tucked-away map's handle to carry the action-station badge, so that hiding the map costs ambience, never awareness.
25. As an operator, I want a session rendered as a moon orbiting its star — working orbits, quiet crawls, dead freezes grey — so that liveness reads at a glance without spending the six-state palette.
26. As an operator, I want proposed work shown as the moon docked at the star's rim and agent review as a violet counter-orbiter, so that the pipeline reads as the moon's story.
27. As an operator, I want human review to break the orbital grammar — a gold beacon with pings, and an edge chevron when the star is offscreen — so that the one state that is a call to action cannot be missed.
28. As an operator, I want live state changes to never move a star — one flare and one fading ticker line only — so that spatial memory and calm survive the map going live.
29. As an operator, I want a mapless space to still offer ad-hoc shells in its working tree, so that chartr is usable as a plain multiplexer.
30. As a keyboard-driven operator, I want keys for summoning the map and switching spaces, so that no path in a cockpit I live in all day is mouse-only.
31. As an operator with atypical color vision, I want every liveness and attention state to carry a non-color channel — motion, shape, or label — so that no state is color-only.

### Sessions, roles, and spawning

32. As an operator, I want to spawn a session on a frontier ticket in one click, with the ticket claimed by a chartr-owned commit, so that starting work is cheap and recorded.
33. As an operator, I want every session to run the agent's own interactive TUI in a PTY, so that I can type into a drifting session and intervene losslessly instead of killing it.
34. As an operator, I want chartr to surface working / quiet / dead and never act on a heuristic — no auto-kill, no timeout, no auto-nudge — so that hints stay hints.
35. As an operator, I want an idle grilling session to show no "quiet" badge, so that a session waiting on me is not dressed as a problem.
36. As an operator, I want a dead session to halt to me — resume (same-ticket crash recovery only), respawn fresh, or abandon — so that nothing requeues or retries on its own.
37. As an operator, I want a dead session's scrollback preserved and pinned to its ticket, so that the next spawn doesn't walk in blind.
38. As an operator, I want roles bound to `{adapter, model, args?}` resolved user-over-workspace with field-level merge, so that committed defaults work everywhere except where my machine and wallet differ.
39. As an operator, I want the effective resolved binding always rendered, so that field-level inheritance never surprises me.
40. As an operator, I want a spawn whose bound agent is absent from my machine hard-blocked with a specific message naming the binding, its source layer, and the local-override fix, so that the ordinary case of a missing CLI is a doorstep diagnosis, not a mid-drive failure.
41. As a project maintainer, I want committed bindings restricted to adapters, models, and portable args — no machine-specific paths — so that what lands in the repo works for whoever clones it.
42. As an operator, I want a committed autopilot flag ignored with a warning, so that no repo can turn off human review for everyone who clones it.
43. As an operator, I want the "ideate" button to spawn a ticketless, un-reviewed session from a hackable starter prompt that suggests — never triggers — escalation to `/wayfinder`, so that the nudge toward charting stays advice.
44. As an operator, I want charting to be my own `/wayfinder` flow in an ordinary shell, with the skill owning the slug and chartr only noticing the folder, so that chartr stays a cockpit over wayfinder, not a wrapper around its skills.

### Prompts and context

45. As an operator, I want the prompt library — five role prompts plus a common core — on disk as plain markdown, so that I can read and edit exactly what my sessions are told.
46. As a project maintainer, I want per-role `replace` and `append` files in committed workspace config, so that house rules ride the repo without forking the shipped prompts.
47. As an operator, I want prompt resolution to walk space → user → built-in with the cockpit surfacing when my replacement is behind the shipped default, so that forks are owned, never auto-merged.
48. As an operator, I want each session's payload — core + role prompt + context bundle — written to a gitignored file in the space and injected as a one-line "read this" opener, so that injection is one path for every TUI and the payload is inspectable.
49. As an operator, I want the exact payload each session received archived and the claim commit to carry layer-provenance and content-hash trailers, so that "what was this session told" is answerable word for word.
50. As an operator, I want context assembled fresh at every spawn — map body, ticket, blockers' answers, glossary — and never accumulated between sessions, so that no ungated "learning" becomes gospel.

### Review and the gate

51. As an operator, I want an implementing session's ticket to read `proposed` — derived from `## Proposed Answer` on disk, surviving a chartr crash — and never `resolved`, so that on disk, resolved always means blessed.
52. As an operator, I want agent review to run on a different configured model than the implementer, with the *observed* models surfaced in the brief, so that marking-your-own-homework is defended at the gate rather than falsely enforced in config.
53. As an operator, I want the review payload to always carry the ticket's Done-when and the spec by assembly, so that the reviewer cannot silently degrade into a style check.
54. As an operator, I want the hub to lead with a one-screen brief — the `## Proposed Answer` verbatim, one line of verdict plus the blocking finding, a mechanically derived recommendation — with the full verdict and diff behind expanders, so that the gate protects by anchoring, not ceremony.
55. As an operator, I want findings allowed to block only by citing the Done-when clause they break — unanchored findings advisory by rule — so that I can tell a real bug from a nitpick structurally.
56. As an operator, I want approve to be one click naming its outcome, plus exactly one "I've read the blocking finding" tick when the reviewer rejected, so that the gate is neither accidental nor exhausting.
57. As an operator, I want an agent-review rejection to halt to me and never loop, so that no tokens burn on unwatched retries.
58. As an operator, I want "take it further" to stack follow-up sessions on the same still-proposed ticket, with the `## Proposed Answer` rewritten in place and diff scopes of all / since verdict / since last read, so that iteration accumulates without new machinery.
59. As an operator, I want send-back to brief the fix-up session — bundle plus blocking finding always, advisories opt-in, my optional note riding the payload and never the ticket file — so that live steering and the ticket's permanent record stay separate.
60. As an operator, I want abandon to demand one thing — a rejection reason addressed to the next attempt, demoted into the ticket as `### Rejected` prose — and destroy nothing, with revert offered as an unticked lever, so that a failed attempt informs the next instead of vanishing.
61. As an operator, I want the post-approve strip to suggest the next best frontier ticket with a spawn button that can never inherit the approve click, so that momentum is offered and never shoved.
62. As a TUI-preferring operator, I want the review brief assembled as plain markdown on disk that the GUI merely renders with buttons, so that a pure-CLI review flow stays open.
63. As an operator, I want the sidebar's per-space wants-you flag to be the jump — one click lands on that space's halted ticket — so that surfacing from an hour heads-down is one click, not a tour of every space. *(Restated by the `needs-you-cut` map, which struck this story's original form: a summonable cross-space queue, gate-level signals only, reviews first, strictly pull. Reviews were cut on the `simplify` effort, and at this cockpit's scale the queue only ever restated the sidebar flag; jump-to, the one thing it added, moved onto the flag.)*

### Git, commits, and the audit trail

64. As an operator, I want chartr to commit exactly its lifecycle writes — claim, promotion, demotion — as pathspec-limited commits with structured trailers, so that approval never waits on a live session and gate commits can never sweep an agent's staged work.
65. As an operator, I want promotion to be its own commit, never an amend, so that proposed-then-blessed stays visible and no SHA is ever rewritten.
66. As an operator, I want agents to commit their own work under prompt convention — message format, granularity, never push — with violations surfaced rather than enforced, so that chartr claims only what it can guarantee.
67. As an operator, I want chartr to never push, so that the remote stays strictly my business.
68. As an operator, I want a dirty tree surfaced as a badge, never a spawn gate, so that I decide whether debris is harmless — accepting that contamination is mine to prevent.
69. As an auditor of past work, I want linear history plus the map to answer who ran what, when, on which model, and how it ended — with no second event store to drift — so that git is the whole audit trail.

### Shipping and platforms

70. As a user on macOS, Linux, or Windows, I want one supported artifact — the pure-Go browser-serving binary with the frontend embedded — cross-compiled from one cgo-free CI job, so that "chartr" means the same thing everywhere.
71. As a user who prefers a native window, I want per-platform webview shells as best-effort extras that never block a release, so that the tier boundary follows the cgo asymmetry.
72. As a user, I want checksummed GitHub releases as the only channel, so that distribution stays honest and simple until releases exist to hang more on.
73. As a cold-start user with zero agent CLIs installed, I want registry, maps, star-map, and ad-hoc shells all working — only spawn blocked, with the block message doubling as my to-do list — so that the first run is un-dramatic.
74. As a Windows user, I want native Windows best-effort by decision — ConPTY-capable PTY layer from day one, CI smoke-tested, WSL2 documented as the sure path — so that the platform's status is stated, not implied.

## Implementation Decisions

### Architecture

- One Go backend per operator machine: model layer reused from wayfinder-maps (load, layers, frontier, derived status, lint — ADR 0001), PTY ownership, `.plan/` watching, process supervision per space; serving a browser frontend (ADR 0006).
- The PTY layer is built on a cross-platform, ConPTY-capable library from day one (ADR 0006 as amended) — the session core never ossifies unix-only.
- Frontend: Svelte 5 over Vite, plain SPA, TypeScript, no SvelteKit (ADR 0010). The framework owns only the chrome; xterm.js terminals and the star-map are imperative islands it never reaches inside.
- The star-map renderer is reimplemented cleanly in TypeScript behind the narrow island seam (mount, receive model, emit selection) — never decomposed into components. Tuned constants and algorithms (easing, zoom coupling, parallax, dpr handling) are cribbed from the wayfinder-maps renderer to prevent feel-drift; deterministic layout, camera easing, and spatial memory are the red lines from the star-map design record.
- Transport: two hand-rolled socket kinds (ADR 0010). A JSON control socket per browser pushes the whole derived model as a snapshot on every change; reconnect is resend-snapshot. A binary terminal socket per attached terminal carries raw PTY bytes down and keystrokes up, with server-side scrollback replayed on attach. Operator actions (spawn, approve, abandon, register) are plain HTTP request/response, so a failed action surfaces as a response.
- Delivery: the Vite build output is embedded in the Go binary; distribution is one self-contained file per platform (ADR 0011). Dev loop is Vite's dev server with HMR proxying websockets to the Go backend; no live-dir hatch.

### State model

- Ticket state is derived from `.plan/` markdown by the reused model layer; session state (agent, PTY, alive/dead) is chartr's own and lives nowhere near the map (ADR 0004). `proposed` is itself derived (`## Proposed Answer` present, `## Answer` absent) and survives a chartr crash.
- chartr's frontier is stricter than wayfinder's: a blocker must be resolved *and* human-approved. That hold is the containment.
- One session per space at a time; parallelism is many spaces (ADR 0003). No worktrees, no branches, linear history. Sessions are space-scoped because the working tree is.
- `session ↔ ticket` is a hard invariant. Ad-hoc shells and the ideate on-ramp are deliberately *not* sessions: ticketless, live, un-reviewed, ended only by the human, sharing only the adapter's spawn primitive.
- A ticket's `type:` selects the role its session spawns as — `grilling`→grill, `prototype`→prototype, `research`→research, `task`→implement — and every ticket offers all four, with the operator picking at the spawn gate (ADR 0015, superseding 0007). A discovered map is live: it renders and spawns the moment it is found, on no chartr config at all. There is no map-level kind, no classification step, and nothing about a map is declared outside the map.
- Map discovery follows wherever wayfinder writes — it must handle the current `.plan/<slug>/` layout and tolerate the move to `.plan/maps/<slug>/`, never hard-coding either. Discovery is by notice (filesystem watch), not refresh.

### Sessions and adapters

- The adapter contract (ADR 0002, as amended by the interactive-spawn decision): `spawn(cwd, model, promptText)` launching the agent's own interactive TUI in a PTY; `observe → {alive, dead}` read from the PTY; `stop` by signal. Exit codes carry no meaning beyond death; *finished* is always derived from the ticket, never asked of the agent.
- No headless mode. The intervention channel — typing into a live TUI — is load-bearing; a two-mode split would foreclose it for exactly the AFK sessions most likely to drift.
- Token telemetry is optional, out-of-band, per-adapter, after-the-fact. No chartr-enforced budget caps. Role wiring travels in the prompt body uniformly — no reliance on any agent's system-prompt flag.
- Working / quiet / dead are surfaced, never acted on. Quiet applies only to AFK ticket types past a silence threshold with no `## Proposed Answer`; a HITL session idle is simply waiting. No auto-kill, no enforced timeout.
- A death halts to the human: resume (same-ticket crash recovery only — ADR 0005 as amended: never across tickets, never instead of a fresh spawn), respawn fresh, or abandon. The stale claim stands until the human acts. No auto-requeue — the no-retry-loops rule.
- Resume across units of work is excluded by design (ADR 0005): context is assembled fresh per spawn; there is no store agents write learnings into.

### Prompts and payload

- The prompt library is vendored from the wayfinder skills, adapted away from Claude-Code-specific conventions, owned and versioned by chartr, recording the upstream commit per sync. Five role prompts (grill, prototype, research, implement, review) plus a common core injected first; the role set is closed — no "charter" role, and the ideate starter prompt is filed as a non-role on-ramp.
- Resolution per role walks space committed config → user config → embedded defaults, with `replace` (resets base) and `append` (stacks) semantics per layer. Embedded defaults materialize to disk as plain markdown. A replacement behind the shipped default is surfaced, never auto-merged.
- At spawn chartr composes core + role prompt + context bundle into one markdown payload, writes it gitignored inside the space, and types a one-line "read this file" opener into the TUI. One assembly path for every agent. An agent that skips the read is visible in its pane — surfaced, not enforced.
- The claim commit carries trailers for layer provenance and a payload content hash; the composed payload is archived per session in chartr-owned state outside git.
- The review payload always includes the ticket's Done-when and the spec, composed by assembly — the reviewer cannot be handed only a diff.

### Configuration

- Role bindings are `{adapter, model, args?}` in TOML, three layers (built-in ‹ committed workspace ‹ local user), field-level merge, resolving user-over-workspace; prompts resolve space-over-user. The reconciling rule (ADR 0009): content the project ships wins; execution choices the operator makes win. The effective resolved binding is always rendered.
- The `args` hatch reaches flags the adapter doesn't model and knowingly forfeits introspection on that binding. Bare command strings and raw argv are rejected forms.
- Committed workspace config holds space-global role bindings. Legal to commit: adapters, models, portable args. Not legal: machine-specific paths, autopilot (a committed autopilot flag is ignored with a warning; autopilot is strictly local-user).
- Heterogeneity (implement ≠ review model) is never guarded at config time — it cannot be verified — and is surfaced as an observed-model line in the review brief instead. Always allow; judgment lives at the gate.
- Absent agent: hard-block that one spawn with a message naming the binding, its source, and the local-override fix. Pre-flight surfacing is the registry badge — there is no doctor command (ADR 0011).
- User config lives under the operator's config directory keyed by space, holding the registry (registered paths, pin, recency — a rebuildable index, not a source of truth), local binding overrides, and autopilot.

### Git and the gate

- Commit ownership splits (ADR 0008). Chartr commits, deterministically: claim at spawn, promotion at approval, rejection demotion at abandonment — each pathspec-limited to the one ticket file, each carrying structured trailers (agent, model, role, verdict). Agents commit their own work plus `## Proposed Answer` under prompt convention; violations (including a push) are verified after the fact and surfaced.
- Promotion is its own commit, never an amend; chartr never rewrites a SHA and never pushes. Approval proceeds during a live session — the narrow write is safe against the shared index; the residual attribution smear (an agent's `commit -a` sweeping the promotion edit) is detected by chartr's own commit coming up empty, and reported.
- Abandonment demotes `## Proposed Answer` to a dated `### Rejected` subsection with the human's reason — the ticket derives open again and the record rides the next bundle. Undoing commits is the human's, with one-click revert (and reset when the commits are verifiably the tip) as optional levers.
- A dirty tree is a badge, not a spawn gate. Git is the audit trail; there is no event store.

### The interface

- Three surfaces (the "Helm" layout, as amended): collapsible spaces→maps sidebar; full-width terminal column with the space's one live session plus history as tabs and a "+" for ad-hoc shells; the star-map summoned as a floating card over the terminal, with an operator-toggled terminal-priority split as the alternative. Map visibility changes only on explicit operator acts.
- Ticket detail is a responsive pane — right dock, re-docking to bottom (capped at half the map panel) when narrow — with the camera easing the selected star into the space the pane leaves free. Deep-links name a star.
- The action station is a numbered badge on the map card toggling a drawer: reviews first, then frontier tickets by unblock count; it rides the map's handle when the map is tucked away.
- Star-map session grammar: amber moon orbiting = session; motion = liveness (working orbits, quiet crawls dimmed, dead freezes grey); docked at rim = proposed; violet counter-orbiter = agent review (the one new hue); gold beacon + pings + offscreen edge chevron = human review, the deliberate break in the orbital grammar. Layout is computed once per map and a state change never moves a star; a live change is one flare plus one fading ticker line.
- Cross-space attention is ambient on sidebar rows and nowhere else: liveness dot, wants-you flag, beacon echo. The wants-you flag is also the jump — clicking it selects the space and lands on its halted ticket — so a gate-level signal has one surface, not two.
- Two binding interface constraints: keyboard-first navigation (map summon and space switch both have keys) and no color-only state (every state carries motion, shape, or label).
- The review hub takes over the map card: brief-first (proposed answer verbatim, one-line verdict plus blocking finding, mechanically derived recommendation — no agent free text at the gate), per-clause Done-when check and diff behind expanders, findings blocking only by clause citation, one-click approve plus a single acknowledgement tick over a rejection, forced-arrival banner on agent-review rejection, take-it-further with three diff scopes, send-back briefing dialog, abandon dialog requiring a reason addressed to the next attempt, and a post-approve suggestion strip whose spawn button enables only after a short delay. The brief is plain markdown on disk; the GUI adds buttons and nothing else.

### Shipping

- One supported artifact (ADR 0011): the pure-Go browser-serving binary per platform, from a single cgo-free CI job. Native webview shells are best-effort extras that never block a release; Windows native is best-effort by decision, smoke-tested in CI, with WSL2 the documented sure path.
- GitHub releases only, goreleaser-built and checksummed. Declined: `go install`, Homebrew (cheap later), and any agent's plugin marketplace.

## Testing Decisions

- **A good test observes external behaviour at the seam and never implementation details.** It drives chartr exactly as an operator would and asserts only on what the design already makes public: HTTP responses, control-socket snapshots, the files in `.plan/`, and git history. No test reaches into Go packages, chartr memory, or private state — if a behaviour matters, the design has already put it on the outside (derived state on disk, lifecycle as commits, the model as a pushed snapshot), and a behaviour observable nowhere outside is a design smell to fix, not a reason for an internal test.
- **The seam is the chartr process boundary, and it is deliberately the only one.** A test starts the real chartr against a temporary fixture space — a git repository with a real `.plan/` map — acts through the operator surface (register, spawn, approve, send back, abandon), and asserts on snapshot + filesystem + git. One seam, tested end to end: derivation, the gate, commit ownership, discovery-by-notice, and the push model all fall out of the same style of test.
- **Agent CLIs are stubbed at the PATH boundary the adapters already probe** — a fake agent script that reads its injected payload, writes a `## Proposed Answer`, commits, hangs, or dies on cue. This exercises spawn, injection, liveness, death-halts-to-human, and the review pipeline deterministically without adding any test-only interface: the PATH probe is a boundary the product already has.
- **The star-map island's seam (mount, receive model, emit selection) is the one frontend test point**, reserved for the renderer's binding guarantees: deterministic layout from ticket data, and zero star movement across the full lifecycle — the property the prototype verified and the design record protects.
- **What gets tested**: the lifecycle end to end (planning tickets resolve directly; implementation tickets walk implementing → proposed → agent review → human review → resolved); the gate's commit discipline (claim/promotion/demotion pathspec-limited, trailers present, promotion during a live session, the empty-commit smear detection); abandonment's demotion and re-derivation; discovery (notice, both `.plan/` layouts); config resolution (field-level merge, user-over-workspace bindings, space-over-user prompts, ignored committed autopilot, absent-agent hard block); payload assembly (layer provenance, review payload always carrying Done-when + spec); and registry semantics (rebuildability, forget-not-destroy).
- **Prior art**: wayfinder-maps' model-layer test suite travels with the reused model layer (ADR 0001) and remains the pattern for exercising derived status against fixture markdown; chartr's own tests extend that fixture-driven style up to the process boundary.

## Out of Scope

- **Cost and token visibility** — declined, not designed. Per-session figures live in each agent's own TUI; the global total is a lagging shadow of the liveness signals already surfaced; cost control is the human watching. Reopen triggers: autopilot, or explicit demand for money-legible spend. The re-entry material is banked in the ticket.
- **Concurrent-session resource limits** — chartr has no governor. It never blocks a spawn for machine load, provider limits, or spend; the operator owns the machine, the rate limits, and the wallet. Same reopen trigger: autopilot.
- **The network and environment sandbox** — a leaf agent still hits live APIs and spends money; containing that is a separate, orthogonal layer. chartr assumes nothing and documents the boundary: the operator owns sandboxing.
- **Redesigning the wayfinder method** — chartr drives maps and extends the markdown adapter by exactly one non-resolving heading. The method, its skills, and its storage shapes are not this project's to change.
- **Charting as a chartr capability** — charting is the user's own `/wayfinder` flow in a shell; chartr injects nothing for it and owns no part of it.
- **Autopilot** — named, strictly-local, non-default, and not designed here; it is the recorded reopen trigger for caps, clean-tree-at-spawn, machine stuck-detection, and cost visibility.
- **tmux as session substrate; retry loops and auto-requeue; an event store; a doctor command; `go install`/Homebrew/marketplace distribution; per-ticket worktrees or branches; SvelteKit; sync libraries** — each considered and declined in the tickets/ADRs cited above.

## Further Notes

- **The honest ceiling** (map Notes): this design makes *orchestration* correct, reliable, and reversible. It cannot make the *work* correct — residual risk lands on leaf-implementation quality and human diligence at the one gate. The gate design (anchored findings, mechanical recommendation, one-tick acknowledgement) is the mitigation, not a solution.
- **Two standing preferences bind every future decision**: *cockpit, not autopilot* — anything that must always be true belongs in deterministic code; an agent sits only where judgment is the product — and *the client is hackable* — prompts, payloads, briefs, and config are plain, editable text on disk.
- **Surface, never enforce** is the recurring stance at every trust boundary: commit conventions, heterogeneity, dirty trees, map lint, an agent skipping its payload read. chartr tells the human; only lifecycle writes are enforced, in code.
- **Named revisit triggers** recorded in the tickets, so future re-opens are decisions, not rediscoveries: autopilot (caps, cost, clean-tree-at-spawn, headless observation), cross-ticket contamination in real driving (clean-tree-at-spawn), a required capability landing in codex's blind spots (ADR 0002 support question), and demand for money-legible spend (cost visibility).
- **Vendored prompt sync** is a standing maintenance duty: each sync from the upstream wayfinder skills records the source commit, and the diff surface is small enough (~hundreds of lines of markdown) to actually review.
- The design map's prototype assets (cockpit layout, star-map states, review hub, registry) hold the rejected variants and the canonical ones; the prototypes are throwaway, but they are the visual reference for the layout, grammar, and hub decisions above.
