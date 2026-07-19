---
type: task
blocked_by: [05, 07, 08]
---

# Spawn a session

## Question

The product's tracer bullet. From a frontier ticket's pane (or the sidebar's affordances), spawn wires the whole chain: the harness writes the claim commit — pathspec-limited to the ticket file, carrying trailers for layer provenance and the payload content hash (ADR 0008) — composes the payload, writes it to a gitignored path inside the space, archives it per session in harness-owned state, resolves the role binding, and the adapter launches the agent's own interactive TUI in a PTY (no headless mode) with the one-line "read this file" opener typed in. The session seats as a tab bound to exactly one ticket. An absent binary hard-blocks that one spawn with the message naming the binding, its source layer, and the local-override fix. Which roles are offered follows the map's kind; an unclassified map offers none.

Done when: with a stub agent CLI on PATH, process-boundary tests assert the claim commit (pathspec, trailers), the gitignored payload whose content matches the preview, the archived copy, and the opener arriving at the stub's stdin; spawning from the pane in the browser lands on a live TUI tab; binding the role to a missing binary blocks with the specific message and blocks nothing else.
