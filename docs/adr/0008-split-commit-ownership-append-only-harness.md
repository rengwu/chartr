# Commit ownership splits; the chartr only ever appends

The chartr commits, deterministically, exactly the lifecycle writes it owns — the claim at spawn, the promotion at approval, the rejection demotion at abandonment — each a **pathspec-limited commit** touching only the ticket file, so a live session's staged work can never be swept into a gate commit. The implementing agent commits its own work plus its `## Proposed Answer`, shaped by prompt convention (message format, granularity, never push) that the chartr verifies after the fact and surfaces when violated, but cannot and does not enforce.

The chartr **never rewrites history and never pushes**: no amend, no automatic reset or revert, no remote. Promotion is its own commit rather than an amend of the session's — proposed-then-blessed stays visible. Undoing rejected work is the human's act; the review hub offers revert (and reset, when the rejected commits are verifiably the tip) as optional levers a human pulls.

The alternatives were rejected on the map's standing rule. Chartr-commits-everything is a fiction — agents commit on their own initiative regardless, so the takeover must handle agent commits anyway. Agents-commit-everything hands the gate write, a must-always-be-true, to a nondeterministic agent.

## Consequences

- **Git is the audit trail.** Claim, work, and verdict are each commits; chartr-owned commits carry structured trailers (agent, model, role, verdict). There is no event store or chartr-owned history — a second history can drift and is invisible to vanilla tools.
- Approval never waits on a live session: the narrow write is safe against the shared index, and the residual race (an agent's `git commit -a` sweeping the promotion edit) degrades to an attribution smear the chartr detects — its own commit comes up empty — and reports.
- The chartr's linearity guarantee ends at the local repository; the remote is the operator's.
- A dirty tree is surfaced, never cleaned: uncommitted debris from a dead or abandoned session is a badge in the cockpit, and spawning into it is allowed. The contamination risk this accepts is the operator's to manage; autopilot, if it ever arrives, forces this open again.

## Amendment: the chartr's write set shrinks to claim + release (simplify, ticket 03)

The ownership split and the append-only rule are unchanged — the chartr still commits only its own lifecycle writes, each pathspec-limited and trailer-carrying, and still never amends, resets, reverts or pushes. What changes is *how many* writes it owns.

With the review feature gone there is no promotion and no rejection demotion, so the chartr's deterministic writes are **two**: the **claim** at spawn and the **release** at a death halt. The implementing agent writes its own `## Answer` — not a `## Proposed Answer` — and commits it, still by prompt convention the chartr cannot enforce.

Two consequences lapse with the writes they described: the approval-vs-live-session race (there is no approval commit left to smear) and the revert/reset levers the review hub offered (there is no hub). The dirty-tree badge stands, and so does git as the whole audit trail.
