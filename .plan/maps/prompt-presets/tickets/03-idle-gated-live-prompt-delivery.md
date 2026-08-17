---
type: task
blocked_by: [01]
undermined_by: []
---

# Deliver one live preset through the idle-gated PTY queue

## Question

Implement the narrow live-delivery contract from [the specification](../spec.md)
inside the terminal manager and server. A request names one catalog preset and one
active terminal. A live Chartr-launched agent receives it immediately when idle;
otherwise it holds one snapshotted pending preset until the next observed idle.
The operator can cancel that item, and a second activation is refused while it is
pending.

Reuse the existing agent identity, terminal status, and typed opener behavior.
Add only enough public terminal state for the pane to know that the tab is an
eligible Chartr-launched agent and which preset is pending. Serialize a submitted
preset's text and separate carriage return against other input writes, but do not
add persistence, history, retries, provider APIs, draft inspection, or delivery to
manually launched agents and shells.

## Done when

- The server refuses unknown presets, ordinary shells, manually launched agents,
  dead tabs, foreign-space terminals, and a second request while one is pending.
- An idle eligible agent receives the snapshotted body followed by a separate
  carriage return; another input write cannot interleave between them.
- Working, running, and blocked agents receive no bytes and expose one pending
  preset until cancellation, exit, or the next observed idle transition.
- The next idle transition submits the pending preset exactly once and clears it;
  cancellation and terminal exit submit nothing.
- The pushed terminal model contains only the small eligibility and pending shape
  the pane consumes, and the queue remains in-memory runtime state.
- Focused terminal-manager tests and server process-boundary tests cover the state
  matrix, write ordering, races at the supported seam, and cleanup, and pass.

