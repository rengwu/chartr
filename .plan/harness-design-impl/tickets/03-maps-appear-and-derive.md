---
type: task
blocked_by: [02]
---

# Maps appear and derive

## Question

A registered space's maps enter the snapshot, live. Port wayfinder-maps' model layer with its test suite (ADR 0001) and extend its derived-status table with `proposed` (`## Proposed Answer` present, `## Answer` absent — ADR 0004) and the harness's stricter frontier (a blocker must be resolved, which on disk means blessed). Discovery is by notice, not refresh: the filesystem watch surfaces a map created by a hosted shell, an external terminal, or a `git pull` — and it reads wherever wayfinder writes, handling the current `.plan/<slug>/` layout and tolerating `.plan/maps/<slug>/`, hard-coding neither. A malformed map renders as-is with the malformation surfaced, never refused. The sidebar nests spaces → maps, finished maps sorting last.

Done when: process-boundary tests show a fixture map dropped into a registered space from outside appearing in the snapshot without any refresh action, under both layouts; derived statuses — including `proposed` and the stricter frontier — are asserted against fixture tickets; the ported model-layer tests pass; the sidebar shows the space's maps.
