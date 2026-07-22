---
type: grilling
---

# The lifecycle after review

## Question

The operator has decided: the review pipeline goes — agent review, the human review hub, the `proposed` state, all of it. The lifecycle becomes: a session holds a ticket, writes its `## Answer`, commits its work, and the ticket is **resolved**. But "delete the gate" is not a decision, it is a direction. This ticket settles what the lifecycle actually is once the gate is gone.

The review pipeline is not a feature bolted on the side — it is load-bearing. The frontier is *defined* as stricter than wayfinder's (a blocker must be resolved **and** human-approved). `proposed` is a ticket status the parser derives and the star-map renders (ticket 09's docked moon, the gold beacon). The claim commit's trailers exist to serve the brief. `promote.go`'s two lifecycle commits are how `## Proposed Answer` becomes `## Answer`. `autopilot` is resolved in config and consumed nowhere. Cutting the pipeline means redefining all of these, not just deleting `gate.go`.

Settle:

- **The new status model.** Which statuses survive (open / blocked / claimed / resolved / out_of_scope?), what exactly `claimed → resolved` requires (an `## Answer` heading? a commit? both?), and who writes the Answer — the agent by prompt convention, or the chartr mechanically at session end? The old design deliberately kept the chartr's writes append-only and pathspec-limited (ADR 0008); does that discipline survive unchanged?
- **What the chartr still guarantees.** With no gate, is there *any* deterministic check left — lint, "session ended without an Answer" surfacing — or does the chartr fully exit the judgment business and trust the operator's git flow? Where is the line between "surfaced, not enforced" and silence?
- **The extension seam, made concrete.** The map refuses a plugin framework but promises that review could return as a consumer of conventions. Name the actual seams (fsnotify events, derived statuses, git trailers, run-dir layout) precisely enough that a future effort could build review *without* touching chartr internals — and decide what the chartr must keep emitting (trailers? session end records?) purely for that hypothetical consumer, versus what is YAGNI and gets cut too.
- **What dies, file by file.** The deletion list: `gate.go`, `review.go`, `promote.go`, `ReviewHub.svelte`, the `proposed` status, autopilot resolution, the `review` role, the violet counter-orbiter and gold beacon on the star-map, the review brief vocabulary in `CONTEXT.md`, ADRs 0004 and 0008. For each: deleted, amended, or retained-and-repurposed — and which ADR amendments the answer must write.
- **In-flight wreckage.** Old maps carry `proposed` tickets, `## Proposed Answer` sections, and rejected-ticket prose. Does the new parser still read them (cheap tolerance), migrate them (a one-time rewrite), or ignore them (history is history)?

## Answer

**The lifecycle is vanilla wayfinder, and the chartr exits the judgment business entirely.** A session holds a ticket, writes its `## Answer`, commits its own work; the ticket resolves the instant the heading lands, and resolved blockers unblock immediately. There is no gate, no chartr-side check, and no new emission for hypothetical consumers. Old maps' `## Proposed Answer` sections become unknown headings — ignored, not tolerated, not migrated.

### The new status model

Four statuses survive: **open, claimed, resolved, out_of_scope**. `proposed` dies with the pipeline that defined it, and nothing replaces it — the chartr's "one addition" to wayfinder's derived-status table (ADR 0004) is withdrawn, and the table becomes exactly wayfinder's own.

- **What `claimed → resolved` requires:** an `## Answer` heading with prose beneath it, written **by the agent, by prompt convention** — the same convention that today produces `## Proposed Answer`, retargeted. The commit of the work is likewise the agent's act by convention (ADR 0008 already established that agents commit on their own initiative regardless). The chartr never writes the Answer, mechanically or otherwise: ADR 0004's core — *the agent writes, the chartr watches* — survives intact; only the gate at the end of the watch is gone.
- **Unblocking:** a resolved blocker unblocks its dependents **immediately**, even while the resolving session is still live and mid-commit. This knowingly gives up ADR 0004's containment consequence ("that hold is the containment") — the operator chose vanilla semantics over a `resolved AND unclaimed` frontier rule. The failure mode it accepts: a dependent can spawn against a ticket whose session hasn't finished committing. The mitigation is social, not mechanical: the claim is visible on the star-map, and the operator is present by design (cockpit, not autopilot).
- **ADR 0008's discipline survives, shrunk.** The chartr's writes stay append-only, pathspec-limited, trailer-carrying commits — but the set shrinks to two: the **claim** at spawn and the **release** at death-halt. The promotion and demotion commits die with the gate they served. `Chartr-Write: true` stays; it is cheap and it is what makes chartr commits distinguishable in the audit trail.
- **`resolved` no longer means blessed.** On disk it now means "the session said so." CONTEXT.md's definition ("resolved always means blessed") and every term built on the gate — `proposed`, `agent review`, `human review`, `review brief`, `abandon`, `autopilot` — die with this ticket. `implementing` survives (a session holding a ticket is still a fact worth naming).

### What the chartr still guarantees

**Nothing new, and nothing that blocks.** The deterministic surfaces that already exist are exactly the ones that survive:

- the **death halt** — a dead session pins to its ticket; an exit without an `## Answer` is visible as a dead session holding an unanswered ticket, and resume/respawn/release remain the operator's three answers;
- the **dirty-tree badge** — uncommitted debris is surfaced, never cleaned (ADR 0008, unchanged);
- **wayfinder lint** — untouched, it never had proposed/review logic.

Rejected: an explicit "session ended without an Answer" attention item (duplicates what the death halt already shows — a surfaced-not-enforced mechanism of exactly the kind this cut exists to remove), and any blocking lint-before-resolution check (a gate by another name, against the settled direction). The line between surfaced and silent is: **the chartr surfaces facts it already derives; it computes no new judgments.**

### The extension seam, made concrete

The seam is documented convention, and every leg of it has a live consumer today — **nothing is emitted for the hypothetical reviewer alone**:

1. **File-derived ticket status.** The parser's heading table (`## Answer`, `## Ruled out`, `claimed_by`) is the public contract. Any consumer, in-process or external, re-derives what the chartr derives from the same markdown.
2. **fsnotify on `.plan/`.** `watch.go` already watches exactly this; an external consumer subscribes to the same directory.
3. **Git trailers on lifecycle commits.** Claim commits carry `Session`, `Agent`, `Model`, `Role`, `Payload-SHA256`, the `*-From` provenance triple, and `Chartr-Write: true`; release commits carry `Session` and `Chartr-Write: true`. This is the audit trail a future review would mine — who implemented, on what model, with what payload.
4. **The run-dir layout.** `.chartr/run/<sid>/` holds the injected payload and its archive — what the agent was told — gitignored, ephemeral, on disk where a consumer can read it.
5. **Session exit** is observable in-process (the terminal manager's `onExit` drives the model push); a future in-process review hooks there. A purely external consumer gets no end-of-session signal — accepted: the map's settled decision contemplates review returning "from inside or outside," and the in-process path needs no new emission.

Rejected: a session-end record (`exit.json` or similar) written purely for an external watcher that does not exist — an emission with zero consumers is the speculative bloat this map exists to cut. If a real second consumer ever arrives, its needs earn the interface; the convention is documented, not speculatively provisioned.

Cut alongside the gate, as gate-owned vocabulary no consumer will miss: the review pointer files (`run/reviews/<slug>/<num>`), the `verdict.md` / `brief.md` contracts, and the gate trailers (`Review-Session`, `Verdict`, `Approved-Over-Rejection`, `Acknowledged-Blocking`, `Review-Recommendation`).

### What dies, file by file

**Deleted outright:**

- `internal/server/gate.go`, `review.go`, `promote.go` — the whole pipeline (approve/abandon/follow-up handlers, brief assembly, verdict parsing, promotion/demotion writes). Helpers defined here but used elsewhere (`repoRel`, `firstLine`, `shortSHA`, `sectionBody`, `stripClaim`-adjacent git helpers) relocate to their surviving callers before the files go.
- Tests: `proposed_test.go`, `review_test.go`, `gate_test.go`, `gate_edges_test.go`; review stubs in `internal/chartrtest/` (`StubProposingAgent`, the follow-up stub, the rig's review-brief/read helpers).
- `web/src/lib/ReviewHub.svelte` (whole component), its `actions.ts` functions (`readReview`, `approveTicket`, `followUp`, `abandonTicket`, `ticketDiff`) and the `Review`/`ReviewRead`/finding types in `model.ts`.
- `prompts/review.md` and `internal/prompt/assets/review.md`.

**Amended:**

- `internal/wayfinder/parse.go` — `StatusProposed`, `HasProposedAnswer`/`ProposedHeading` and their `Derive()` branch removed; `Frontier()` reverts to vanilla (resolved blockers unblock; no approval, no unclaimed condition). `doc.go` loses "the chartr's one addition."
- `internal/server/` — routes (`server.go`), the review-role seating gate (`spawn.go`), `reviewState`/`ticketProposed` and `Ticket.Review` wiring (`spaces.go`, `model.go`), the review-payload guarantee (`internal/prompt/compose.go`, `prompt.go` role list).
- `internal/config/binding.go` — `RoleReview`, its `Roles` entry, the implementation-map role pair, and the entire `autopilot` resolution (fields, warning, `Resolution.Autopilot`): confirmed resolved-and-consumed-nowhere, so it dies without replacement.
- Star-map: `session.ts` drops `proposed`/`agent-review`/`human-review` states and their grammar (docked moon, violet counter-orbiter, warm break); `theme.ts` loses the proposed palette and violet hue; `starmap.ts` loses the sealed ring, the gold warming, and the offscreen "keeps calling" beacon.
- Chrome: `attention.ts` (review-first items, review queue kind), `NeedsYouQueue.svelte`, `ActionStation.svelte`, `App.svelte`, `MapCard.svelte` (hub takeover), `DetailPane.svelte` (review action/badge/section), `MapPickerCard.svelte` and `SpacePane.svelte` (gate copy).
- Prompts: `implement.md` (write `## Answer`, not `## Proposed Answer`; drop the reviewer mention), `core.md`, `glossary.md`, `ideate.md`, `README.md` — each plus its `internal/prompt/assets/` copy.
- `CONTEXT.md` — the vocabulary deaths listed above.
- `docs/design-system.md` — the star-map review grammar.
- `user.toml` / config samples — the `autopilot` key, if present.

**ADRs:** this answer writes three amendments and retires no ADR wholesale.

- **ADR 0004 — amended.** Derived ticket state survives; the `## Proposed Answer` extension, the promotion-at-gate, and the stricter-than-wayfinder frontier are withdrawn. The containment consequence is explicitly forfeited (see revisit trigger).
- **ADR 0008 — amended.** Commit ownership split and append-only discipline survive; the lifecycle-write set shrinks to claim and release. Promotion/demotion commits, and the gate-race consequences written around them, are removed.
- **ADR 0009 — amended.** The layering rule is untouched; the autopilot bullet and its two considered options are removed (the flag itself is deleted, not merely uncommitted).
- ADR 0007 survives with one premise struck: kind no longer selects a *lifecycle* (both kinds now share the vanilla one); it still selects the role set a map offers. ADR 0011 is not this ticket's business — the webview-shell tier question belongs to ticket 04.

### In-flight wreckage

**Ignore.** `## Proposed Answer` becomes an unknown heading; tickets carrying it derive open (or claimed, if a claim marker survives). `## Rejected attempts` prose is already inert — `###` subsections never matched the closing-heading scan. The old maps (`chartr-design`, `chartr-design-impl`, `reskin`) are declared history: they stay human-readable, and no tool migrates them.

Rejected: *tolerate-as-answer* (~5 parser lines) — it would silently bless answers no human approved, and it keeps a dead state alive in every future reader's head, the exact ghost-code this map exists to remove. *One-time migration* — code written for data nobody works; the map already declared the old maps a non-rescue-mission.

### Revisit trigger

Two tripwires, either sufficient:

- **Containment bites.** If dependents seeded by still-live, mid-commit resolutions produce real wreckage in practice — not in anticipation — the `resolved AND unclaimed` frontier rule returns as a parser-level fix. That is a new ticket, not a resurrection of the gate.
- **A real second consumer appears.** If review (or anything else) is actually rebuilt against these seams, its concrete needs earn an interface — and only then does the chartr emit anything for it.
