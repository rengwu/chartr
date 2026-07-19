---
type: task
blocked_by: [09]
---

# Liveness and the death halt

## Question

`observe` is `{alive, dead}` read from the PTY, and the harness surfaces but never acts (planning ticket 02). Working, quiet, dead enter the snapshot: quiet is a hint shown only for AFK ticket types silent past a threshold with no `## Proposed Answer` — an idle HITL session shows nothing. A death halts to the human: the dead session stays pinned to its ticket with scrollback preserved, and the operator chooses resume (same-ticket crash recovery only — ADR 0005 as amended), respawn fresh, or release the claim back to the frontier; the stale claim stands until they act. No auto-kill, no enforced timeout, no auto-requeue. A dirty working tree — from a session or an ad-hoc shell — is a badge, never a spawn gate.

Done when: process-boundary tests with stub agents assert dead is detected on exit, the halt offers exactly the three choices and takes none itself, scrollback survives death, quiet appears only for the AFK case past threshold, and a dirtied tree badges while spawn still proceeds; the absence of autonomous action is asserted, not assumed (no state change without an operator HTTP call).
