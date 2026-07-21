---
type: task
blocked_by: []
---

# Delete the review feature

## Question

Remove the review pipeline as a *feature* — the gate, its server mechanics, its
UI, its role — leaving a harness that compiles and, crucially, still resolves and
unblocks so it can keep driving its own map. This is the bulk deletion; retiring
the `proposed` status itself and the last gate-shaped semantics (the parser, the
`Frontier` revert) is ticket 03, split out so a stall here never strands the
parser mid-flip. Land as small, independently-green commits (frontend, then
backend).

- **Frontend.** Delete `ReviewHub.svelte` and its `actions.ts` / `model.ts`
  review surface (`readReview`, `approveTicket`, `followUp`, `abandonTicket`,
  `ticketDiff`; the `Review` / `ReviewRead` / finding types); strip the chrome
  review items (`attention.ts` review-first items + review queue kind,
  `NeedsYouQueue.svelte`, `ActionStation.svelte`, `App.svelte`, `MapCard.svelte`
  hub takeover, `DetailPane.svelte` review action/badge/section,
  `MapPickerCard.svelte` / `SpacePane.svelte` gate copy); remove the star-map
  **session/review overlay** grammar (`session.ts` proposed / agent-review /
  human-review states + their motion/mark grammar, `theme.ts` violet hue,
  `starmap.ts` sealed ring / gold warming / offscreen beacon). **Update the vitest
  specs that assert the removed grammar so the suite goes green without it** —
  `attention.test.ts`, `starmap/session.test.ts`, `starmap/starmap.test.ts`, and
  the review portions of `markdown.test.ts`. Leave the *base* `proposed` star and
  its palette alone — ticket 03 removes them with the status.
- **Backend.** Delete `internal/server/gate.go`, `review.go`, `promote.go` and the
  review/gate tests (`proposed_test.go`, `review_test.go`, `gate_test.go`,
  `gate_edges_test.go`) plus the `internal/harnesstest` review stubs; strip the
  review routes (`server.go`), the review-role seating (`spawn.go`), and the
  `reviewState` / `ticketProposed` / `Ticket.Review` wiring (`spaces.go`,
  `model.go`); remove the `compose.go` review branch (the `RoleReview` payload
  guarantee, `resolveSpec` / `reSpecLink`, `ProposedAnswerSection`). In
  `config/binding.go` remove `RoleReview`, its `Roles` entry, and its
  `RolesForKind` pair, so no review role can be seated. Deleting `promote.go`
  removes the promotion/demotion commits, shrinking the harness's lifecycle writes
  to the **claim** at spawn and the **release** at death-halt (the surviving
  append-only, `Harness-Write: true` writes). The helpers in the deleted files —
  `repoRel`, `firstLine`, `shortSHA`, `sectionBody` — have no callers outside
  them, so they go with them; no relocation is needed.
- **Prompts.** Delete `prompts/review.md` and its asset. **Retarget `implement.md`
  (and its asset) to write `## Answer` and drop the reviewer mention** — this
  lands *here*, not in ticket 03, because once the gate is gone a session still
  told to write `## Proposed Answer` would derive `proposed` and never unblock its
  dependents; retargeting keeps the map livable the moment the gate is removed.

The parser still carries `StatusProposed` after this ticket — dormant, since no
session now produces a `## Proposed Answer`. Ticket 03 removes it.

Done when: `go vet ./...` / `go test ./...` are green with gate / review / promote
and the review role gone and no review test remaining; the frontend `check` /
`build` / `vitest` are green with no review UI and no amber; a fresh session
resolves a ticket by writing `## Answer` and committing, with no gate in the path
and its dependents unblocking immediately; and no review role can be spawned.
