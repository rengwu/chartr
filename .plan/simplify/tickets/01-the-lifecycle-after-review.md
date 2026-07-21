---
type: grilling
---

# The lifecycle after review

## Question

The operator has decided: the review pipeline goes — agent review, the human review hub, the `proposed` state, all of it. The lifecycle becomes: a session holds a ticket, writes its `## Answer`, commits its work, and the ticket is **resolved**. But "delete the gate" is not a decision, it is a direction. This ticket settles what the lifecycle actually is once the gate is gone.

The review pipeline is not a feature bolted on the side — it is load-bearing. The frontier is *defined* as stricter than wayfinder's (a blocker must be resolved **and** human-approved). `proposed` is a ticket status the parser derives and the star-map renders (ticket 09's docked moon, the gold beacon). The claim commit's trailers exist to serve the brief. `promote.go`'s two lifecycle commits are how `## Proposed Answer` becomes `## Answer`. `autopilot` is resolved in config and consumed nowhere. Cutting the pipeline means redefining all of these, not just deleting `gate.go`.

Settle:

- **The new status model.** Which statuses survive (open / blocked / claimed / resolved / out_of_scope?), what exactly `claimed → resolved` requires (an `## Answer` heading? a commit? both?), and who writes the Answer — the agent by prompt convention, or the harness mechanically at session end? The old design deliberately kept the harness's writes append-only and pathspec-limited (ADR 0008); does that discipline survive unchanged?
- **What the harness still guarantees.** With no gate, is there *any* deterministic check left — lint, "session ended without an Answer" surfacing — or does the harness fully exit the judgment business and trust the operator's git flow? Where is the line between "surfaced, not enforced" and silence?
- **The extension seam, made concrete.** The map refuses a plugin framework but promises that review could return as a consumer of conventions. Name the actual seams (fsnotify events, derived statuses, git trailers, run-dir layout) precisely enough that a future effort could build review *without* touching harness internals — and decide what the harness must keep emitting (trailers? session end records?) purely for that hypothetical consumer, versus what is YAGNI and gets cut too.
- **What dies, file by file.** The deletion list: `gate.go`, `review.go`, `promote.go`, `ReviewHub.svelte`, the `proposed` status, autopilot resolution, the `review` role, the violet counter-orbiter and gold beacon on the star-map, the review brief vocabulary in `CONTEXT.md`, ADRs 0004 and 0008. For each: deleted, amended, or retained-and-repurposed — and which ADR amendments the answer must write.
- **In-flight wreckage.** Old maps carry `proposed` tickets, `## Proposed Answer` sections, and rejected-ticket prose. Does the new parser still read them (cheap tolerance), migrate them (a one-time rewrite), or ignore them (history is history)?
