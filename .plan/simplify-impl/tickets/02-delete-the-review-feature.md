---
type: task
blocked_by: []
---

# Delete the review feature

## Question

Remove the review pipeline as a *feature* — the gate, its server mechanics, its
UI, its role — leaving a chartr that compiles and, crucially, still resolves and
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
  `gate_edges_test.go`) plus the `internal/chartrtest` review stubs; strip the
  review routes (`server.go`), the review-role seating (`spawn.go`), and the
  `reviewState` / `ticketProposed` / `Ticket.Review` wiring (`spaces.go`,
  `model.go`); remove the `compose.go` review branch (the `RoleReview` payload
  guarantee, `resolveSpec` / `reSpecLink`, `ProposedAnswerSection`). In
  `config/binding.go` remove `RoleReview`, its `Roles` entry, and its
  `RolesForKind` pair, so no review role can be seated. Deleting `promote.go`
  removes the promotion/demotion commits, shrinking the chartr's lifecycle writes
  to the **claim** at spawn and the **release** at death-halt (the surviving
  append-only, `Chartr-Write: true` writes). The helpers in the deleted files —
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

## Answer

The review pipeline is gone as a feature, in two independently-green commits —
frontend, then backend. The chartr still builds, still derives ticket status,
and still spawns; it now resolves a ticket the vanilla-wayfinder way.

**Frontend.** `ReviewHub.svelte` deleted, with the `actions.ts` review surface
(`readReview` / `approveTicket` / `followUp` / `abandonTicket` / `ticketDiff` and
the `Review` / `ReviewRead` / finding types) and `model.ts`'s `Review` type and
`Ticket.review`. `review` left `ROLES` and `rolesForKind('implementation')`. The
chrome's review items went with it: `attention.ts` lost its review-first ranking
and its `review` queue/attention kind (`ActionItem` is now just a ranked frontier
row, `QueueEntry`/`Attention` just `halt`), and `NeedsYouQueue`, `ActionStation`,
`App`, `MapCard` (hub takeover), `DetailPane` (review section + `onopenreview`),
`MapPickerCard` and `SpacePane` followed. The star-map's session overlay lost
`proposed` / `agent-review` / `human-review` and their whole grammar — the violet
counter-orbiter, the gold warming, the ping rings, the sealed ring, and the
offscreen beacon chevron (`beckoning()` / `#drawChevrons` / `#beckonRect` with
it). The base `proposed` star and its palette are deliberately untouched. The
vitest specs asserting the removed grammar were **updated, not deleted**:
`attention.test.ts`, `starmap/session.test.ts`, `starmap/starmap.test.ts`.

**Backend.** `gate.go`, `review.go`, `promote.go` deleted with
`gate_test.go` / `gate_edges_test.go` / `review_test.go` / `proposed_test.go`,
the review routes, and the `chartrtest` review stubs. `RoleReview` left
`config/binding.go` entirely, so no review role can be seated; `spawn.go` lost
its proposed-ticket seating branch and every role now spawns onto the frontier.
`compose.go` lost the review branch, `resolveSpec` / `reSpecLink` /
`ProposedAnswerSection`, and `spaces.go` lost `reviewState` / `ticketProposed` /
the `Ticket.Review` wiring. Deleting `promote.go` shrank the chartr's lifecycle
writes to the **claim** and the **release**.

**Prompts.** `review.md` and its asset deleted. `implement.md` was retargeted to
write `## Answer` — and **`core.md` with it**, which the ticket did not name: the
same "on an implementation map you write a `## Proposed Answer` and stop"
instruction lives in both, and either one left pointing at the old heading would
derive `proposed` and strand every dependent, which is the exact failure this
retarget exists to prevent. The injected glossary lost `review` as a role.

**Three judgement calls, each an extension of a rule the ticket already states.**
The ticket's own rule for the deleted files' helpers — no callers outside them,
so they go with them — applied cleanly to three more casualties: `Steer` /
`Bundle.Steering` (the gate's follow-up briefing; its only producer was the
deleted `handleFollowUp`), `Bundle.MapDir` and `doneWhen` (which existed only to
anchor and fill the review payload's spec guarantee), and `InstallSweepHook`
(which only ever exercised the promotion commit's attribution smear — the
mechanism it tested died with `promote.go`). `TestQuietOnlyForAFKPastThreshold`
lost its final beat, since removing `ticketProposed` removes the withdrawal it
asserted; the AFK-vs-HITL split it exists for is intact.

**Left for ticket 03, on purpose.** `StatusProposed` and its derivation, the
`## Proposed Answer` fallback in `AnswerSection` / `DetailPane`'s
`ANSWER_SECTIONS` / `markdown.test.ts`, the base `proposed` star and its palette,
autopilot resolution, and the CONTEXT.md / ADR vocabulary. The parser is
unchanged — the dangerous half stays isolated in its own small ticket, which is
the whole reason the cut was split.

**Verified against the real binary,** not just the suite: the review routes fall
through to the SPA; a review role is refused at both spawn (`role review is not
offered by a implementation map`) and preview (`unknown role "review"; want one
of [grill prototype research implement]`); and a session spawned on a frontier
ticket resolved it by writing `## Answer` and committing, with its dependent
unblocking on the next pushed snapshot and only two commits in the log — the
chartr's claim and the agent's own.

Done when, met: `go vet ./...` / `go test ./...` green with gate / review /
promote and the review role gone and no review test remaining; frontend `check` /
`build` / `vitest` green with no review UI and no amber in the built CSS; a fresh
session resolves by `## Answer` with no gate in the path and its dependents
unblocking immediately; and no review role can be spawned. **No ADR is touched
here** — the 0004 / 0008 / 0009 amendments and 0007's struck premise land with
ticket 03, which is where the semantics they describe actually change.

One local, uncommitted casualty worth naming: `user.toml` carried a
`roles.review` binding (kimi), the role this ticket removed. It was dropped
rather than repointed — which agent runs `implement` is the operator's call, not
the cut's.
