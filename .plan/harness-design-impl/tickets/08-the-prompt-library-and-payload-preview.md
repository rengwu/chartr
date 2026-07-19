---
type: task
blocked_by: [04]
---

# The prompt library and payload preview

## Question

The hackable prompt library, live before any PTY exists. Five role prompts plus the common core, vendored from the wayfinder skills and recording the upstream commit, embedded in the binary and materialized to disk as plain markdown; resolution per role walks space committed config → user config → embedded defaults with `replace` (resets base) and `append` (stacks) semantics per layer; a replacement sitting behind the shipped default is surfaced, never auto-merged. Payload composition assembles core + role prompt + context bundle (map body, ticket, blockers' answers, glossary — ADR 0005, fresh every time) into one markdown document; a preview surface shows, for a chosen ticket and role, exactly what a session would be told, with per-part layer provenance. The review payload always carries the ticket's Done-when and the spec, by assembly.

Done when: process-boundary tests cover the resolution matrix (replace/append across all three layers), behind-default surfacing, bundle assembly from fixture tickets, and the review payload provably containing Done-when and spec; the preview renders the composed payload with provenance in the browser; the materialized library is editable on disk and edits show up in the next composition.
