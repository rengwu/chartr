---
type: task
blocked_by: [02]
---

# Revert to the vanilla lifecycle

## Question

With the review feature gone, retire the `proposed` status itself and the last
gate-shaped semantics, so the derived-status model becomes exactly wayfinder's.
Small and isolated on purpose — this is the dangerous half (a half-done parser
misreads every ticket's status, this map's own included), kept apart from ticket
02's bulk so it lands in one clean, green commit.

- **Parser** (`internal/wayfinder/parse.go`). Remove `StatusProposed`,
  `HasProposedAnswer` / `ProposedHeading`, and their `Derive()` branch; revert
  `Frontier()` to vanilla — resolved blockers unblock, with no approval and no
  unclaimed condition; `doc.go` loses "the harness's one addition." An in-flight
  `## Proposed Answer` is now an **unknown heading**: a ticket carrying it derives
  `open` (or `claimed` if a claim marker survives), never `resolved` — ignored,
  not migrated.
- **The context bundle stops treating `## Proposed Answer` as an answer.** This is
  the settled *ignore, don't tolerate-as-answer* decision (spec → Implementation
  Decisions; planning ticket 01 rejected tolerate-as-answer because it would
  silently bless answers no human approved). In `compose.go`, `AnswerSection`
  drops its `Proposed Answer` fallback — a blocker's answer reads `## Answer`, else
  `## Ruled out`, and **never** an in-flight `## Proposed Answer` — so a dependent
  is never handed an unblessed proposal as though it were the answer. Verify no
  other reader on the blocker-answer path (the context-bundle assembly, the ticket
  pane's inline blocker answers) still falls back to it.
- **Config** (`config/binding.go`). Remove the entire `autopilot` resolution — the
  fields, the warning, `Resolution.Autopilot` — confirmed resolved-and-consumed-
  nowhere, deleted without replacement.
- **Star-map.** Remove the **base** `proposed` star and its palette (`theme.ts`
  proposed hue and the base state) now that the status is gone.
- **Docs & ADRs.** Strike the review vocabulary from `CONTEXT.md` (`proposed`,
  `agent review`, `human review`, `review brief`, `abandon`, `autopilot`) and
  redefine `resolved` ("the session said so", not "always blessed"); update
  `docs/design-system.md`'s star-map grammar; write the **ADR 0004 / 0008 / 0009
  amendments** and strike **ADR 0007**'s lifecycle-selection premise (kind still
  selects the role set).

Done when: `internal/wayfinder` tests assert `Derive()` yields only open /
claimed / resolved / out_of_scope, that a `## Proposed Answer` derives open or
claimed (never resolved), and that `Effort.Frontier()` unblocks a dependent the
instant its blocker resolves; a `compose` test asserts a blocker carrying only
`## Proposed Answer` contributes **no** answer to a dependent's context bundle;
`go vet ./...` / `go test ./...` and the frontend gates are green; and ADRs 0004 /
0008 / 0009 are amended with 0007's premise struck.
