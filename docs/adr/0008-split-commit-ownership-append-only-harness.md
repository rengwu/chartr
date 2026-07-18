# Commit ownership splits; the harness only ever appends

The harness commits, deterministically, exactly the lifecycle writes it owns — the claim at spawn, the promotion at approval, the rejection demotion at abandonment — each a **pathspec-limited commit** touching only the ticket file, so a live session's staged work can never be swept into a gate commit. The implementing agent commits its own work plus its `## Proposed Answer`, shaped by prompt convention (message format, granularity, never push) that the harness verifies after the fact and surfaces when violated, but cannot and does not enforce.

The harness **never rewrites history and never pushes**: no amend, no automatic reset or revert, no remote. Promotion is its own commit rather than an amend of the session's — proposed-then-blessed stays visible. Undoing rejected work is the human's act; the review hub offers revert (and reset, when the rejected commits are verifiably the tip) as optional levers a human pulls.

The alternatives were rejected on the map's standing rule. Harness-commits-everything is a fiction — agents commit on their own initiative regardless, so the takeover must handle agent commits anyway. Agents-commit-everything hands the gate write, a must-always-be-true, to a nondeterministic agent.

## Consequences

- **Git is the audit trail.** Claim, work, and verdict are each commits; harness-owned commits carry structured trailers (agent, model, role, verdict). There is no event store or harness-owned history — a second history can drift and is invisible to vanilla tools.
- Approval never waits on a live session: the narrow write is safe against the shared index, and the residual race (an agent's `git commit -a` sweeping the promotion edit) degrades to an attribution smear the harness detects — its own commit comes up empty — and reports.
- The harness's linearity guarantee ends at the local repository; the remote is the operator's.
- A dirty tree is surfaced, never cleaned: uncommitted debris from a dead or abandoned session is a badge in the cockpit, and spawning into it is allowed. The contamination risk this accepts is the operator's to manage; autopilot, if it ever arrives, forces this open again.
