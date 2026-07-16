---
type: grilling
blocked_by: [01]
---

# Knowing a session finished, hung, or died

## Question

The harness watches `.plan/` and derives resolution from the files (ADR 0004), so it never has to ask an agent whether it is done. That covers the happy path and nothing else. A session can also exit cleanly having achieved nothing, exit non-zero, hang forever waiting on input nobody will give it, loop while burning tokens, or die outright while its ticket still reads `implementing`.

Given what ticket 01 finds each CLI actually exposes, decide how the harness tells **working** from **stuck** from **dead**, and what it does about each. The candidate signals all have holes worth naming: process exit is weak (an agent that exits without writing `## Proposed Answer` has failed *silently*); a wall-clock timeout punishes slow-but-honest work; output-silence heuristics are guesses, and a grilling session is *supposed* to sit idle waiting for a human.

Settle also:

- What the harness owes the human when a session dies mid-ticket. A stale `claimed_by` is litter the markdown adapter already tolerates and can reason about; a half-finished commit in the working tree is not (ticket 06 owns the git side).
- Whether a dead session's ticket returns to the frontier **automatically or only through a human**. Remember this project has **no retry loops** — every rejection halts to a human, on the argument that looping burns tokens — and the same argument applies to respawning the dead.
- Whether HITL and AFK sessions need different answers here. They almost certainly do: idleness means opposite things.
