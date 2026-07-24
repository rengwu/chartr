---
type: task
blocked_by: [09]
---

# Propose and agent review

## Question

The pipeline from work landing to a readable verdict. An implementing session committing its work and its `## Proposed Answer` derives the ticket `proposed` (already in the model layer — here it must flow through snapshot and UI as the state a review hangs on). Spawning the review role composes the payload that provably carries the ticket's Done-when and the spec; the review prompt defines the clause-anchored verdict format — every Done-when clause assessed met/unmet, a finding able to block only by citing the clause it breaks, unanchored findings advisory by rule. From the verdict chartr assembles the review brief as plain markdown on disk: the `## Proposed Answer` verbatim, the one-line verdict plus the blocking finding, a recommendation derived mechanically from the verdict (never agent free text), and the observed-model heterogeneity line.

Done when: process-boundary tests walk a stub implementer to `proposed`, spawn a stub reviewer whose payload contains Done-when and spec, and assert the brief file exists with verbatim proposed answer, mechanical recommendation matching the verdict, clause-anchoring respected (an unanchored finding lands advisory), and observed models named; the raw brief is readable on disk exactly as the GUI will render it.

## Answer

The pipeline from work landing to a readable verdict is wired end to end, reusing the spawn/claim/payload machinery ticket 09 built and adding only the review seam on top. Nothing here re-decides the design; where ticket 09 flagged a gate as needing to widen, this widens exactly that gate and no more.

**What I built:**

- **`proposed` flows through the snapshot and the UI.** The model layer already derives `proposed` from a committed `## Proposed Answer` (ADR 0004); `mapscan` already carries it onto `Ticket.Status`, so no backend change was needed for the snapshot — verified by `TestProposedFlowsToSnapshot`, which also asserts a merely-proposed blocker never lands its dependent on the stricter frontier. The chrome already renders `proposed` (a status badge in `DetailPane`, the base star from ticket 06). The one UI addition (`web/src/lib/DetailPane.svelte`): a `proposed` ticket on an implementation map now offers exactly a **review** spawn — `offeredRoles(kind, ticket)` returns the kind's roles on the frontier and `['review']` on a proposal — so the pipeline is reachable from the pane. Built on existing primitives (Button/Badge) and the existing `spawnSession` action; no new colour, no new component.

- **A review seats on a `proposed` ticket.** `internal/server/spawn.go` splits the takeable-state gate by role: `review` requires `tk.Status == "proposed"` (work landed, awaiting the gate); every other role keeps the fresh-spawn frontier gate unchanged. This is the widening ticket 09 explicitly flagged. `launchSession` is reused untouched, so a review spawn composes the review payload (which `prompt.Compose` already guarantees carries the Done-when and the spec, story 53), writes its own claim commit (`Role: review`), archives the payload, and opens the agent TUI — all the ticket-09 mechanics for free.

- **The review prompt defines the clause-anchored verdict format** (`internal/prompt/assets/review.md`): the reviewer writes `verdict.md` beside its payload, assessing *every* Done-when clause met/unmet, with each finding marked `blocking (Done-when: "<clause>")` or `advisory` — a finding blocks only by citing the clause it breaks; an unanchored finding is advisory by rule.

- **chartr assembles the brief mechanically** (`internal/server/review.go`, `POST …/sessions/{sid}/review-brief`): it parses the verdict, reads the ticket's `## Proposed Answer` verbatim off disk (`prompt.ProposedAnswerSection`), and writes `brief.md` to the session's gitignored run dir (a sibling of payload/verdict, `.gitignore`d so it can never be swept into a commit — ADR 0008). The recommendation is a pure function of the findings' anchoring — **any anchored blocking finding → Send back, none → Approve** — never lifted from the agent's pass/fail prose (story 54). A "blocking"-marked finding citing no clause is demoted into Advisories with a note saying why (story 55). Observed models come from the audit trail with no second store (story 69): the implementer's from the ticket's `Role: implement` claim trailer in git history, the reviewer's from the live session's binding, with a heterogeneity line (same-model reviewing its own work is surfaced, never gated — story 52).

**How each Done-when clause is met** — six process-boundary tests in `internal/server/review_test.go`, all on the public seam (snapshot, filesystem, git), driven against stub agents on PATH (new `chartrtest.StubProposingAgent`, which appends a `## Proposed Answer`, commits, and dies; `StubAgent` for the live reviewer):

- *walk a stub implementer to `proposed`* — `walkToProposed` spawns the proposing stub and waits for the snapshot to read `proposed` with the implementer pinned dead; `TestProposedFlowsToSnapshot` asserts the state and the frontier hold.
- *spawn a stub reviewer whose payload contains Done-when and spec* — `TestReviewBriefAssembly` reads the review session's payload file and asserts a Done-when clause, the `Done-when (the review contract)` label, and the `Spec (` section are all present.
- *brief exists with verbatim proposed answer / mechanical recommendation matching the verdict / clause-anchoring respected / observed models named* — `TestReviewBriefAssembly` asserts all four against `brief.md` on disk, and that the file is byte-identical to the action's response (the GUI renders the file, nothing more). `TestReviewBriefApproveOnPass` proves the recommendation is mechanical, not the agent's word (a clause-free verdict → Approve). `TestReviewRunsOnlyOnProposed` and `TestReviewBriefRefusedWithoutVerdict` guard the gate and the missing-verdict case.

**Tested:** `go vet ./...` and `go test ./...` pass; `svelte-check` (0/0), `vitest` (33), and the Vite build all pass; no amber in the built CSS.

**Deliberately left out** (these are tickets 12+, not scope creep here): the human review hub — approve/abandon/take-it-further, the promotion/demotion commits, the acknowledgement tick, the send-back briefing dialog, and the "Needs you" queue. The brief is written to disk and returned by the action; rendering it with buttons is the hub's job. Brief assembly is an explicit operator action rather than an automatic on-death trigger, matching the "human drives / GUI adds buttons" stance and keeping the pipeline deterministic; if a reviewer's finding wraps across multiple lines only its first line (which carries the marker, clause, and lead) is parsed — sufficient for the brief, and flagged here in case review wants the whole finding preserved.
