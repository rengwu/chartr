---
type: task
blocked_by: [09]
claimed_by: s31f0ce460375
claimed_at: 2026-07-21T04:31:01Z
---

# Propose and agent review

## Question

The pipeline from work landing to a readable verdict. An implementing session committing its work and its `## Proposed Answer` derives the ticket `proposed` (already in the model layer — here it must flow through snapshot and UI as the state a review hangs on). Spawning the review role composes the payload that provably carries the ticket's Done-when and the spec; the review prompt defines the clause-anchored verdict format — every Done-when clause assessed met/unmet, a finding able to block only by citing the clause it breaks, unanchored findings advisory by rule. From the verdict the harness assembles the review brief as plain markdown on disk: the `## Proposed Answer` verbatim, the one-line verdict plus the blocking finding, a recommendation derived mechanically from the verdict (never agent free text), and the observed-model heterogeneity line.

Done when: process-boundary tests walk a stub implementer to `proposed`, spawn a stub reviewer whose payload contains Done-when and spec, and assert the brief file exists with verbatim proposed answer, mechanical recommendation matching the verdict, clause-anchoring respected (an unanchored finding lands advisory), and observed models named; the raw brief is readable on disk exactly as the GUI will render it.
