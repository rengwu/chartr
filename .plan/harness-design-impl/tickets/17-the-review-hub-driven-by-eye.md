---
type: task
blocked_by: [12]
---

# The review hub, driven by eye

## Question

The one Done-when clause ticket 12 left unmet, plus the read-wrong edges its
review found. Ticket 12's backend exits are proven at the process boundary, but
the single clause about the browser — *"in the browser every exit is drivable and
the hub renders the on-disk brief with buttons and nothing else"* — was never
exercised by anyone, and the wiring fails it.

**The blocking failure is in the browser, and only the browser: approve destroys
its own post-approve strip.** `handleApprove` (`internal/server/gate.go`) calls
`s.rebuild()` — which pushes the resolved snapshot synchronously — *before* it
writes its HTTP response, so by the time `ReviewHub.approve()` sets `approved`,
`MapCard.svelte`'s `$effect` (`hubTicket !== null && hubOn?.status !== 'proposed'
→ hubTicket = null`) has already unmounted the hub, taking the `approved` strip
down with it — and with it the promotion confirmation and the `smearedInto`
attribution-smear warning that ADR 0008 report is rendered *nowhere else*. On top
of that the strip's "Next: #NN" button — which story 61 specifies as a spawn
button that enables only after a ~450ms delay — has `onclick={onclose}` as its
only handler: it neither spawns the suggested ticket nor even selects it. Keep the
hub mounted through the approve transition (exempt the locally-approved ticket
from the close effect, or lift `approved` into `MapCard` so the strip outlives the
status flip), make Next actually spawn-or-navigate to the suggestion, and then
**drive all four exits in a real browser** — the eye-verification tickets 06, 07,
12 and 13 each deferred, and that 12's Done-when finally makes non-optional.

**While the hub is open by eye, close the gate's edges the review anchored** (all
advisory in the verdict — none blocks on its own, but they make the re-review loop
read wrong at the gate):

- **The re-review shadow.** `reviewFor` / `ensureBrief` key the brief on the
  review *session's tab* and iterate `s.terms.ForSpace` in creation order, so
  after send-back → fix-up → a fresh review, the pinned dead session's stale brief
  masks the new verdict — approve keeps demanding acknowledgement of a finding the
  fix-up already cleared. The same keying orphans the brief when a tab is discarded
  or a restart drops it: the ticket falls back to plain `proposed` until a fresh
  review runs. Move the brief to a **per-ticket path** (or iterate newest-first) so
  the newest verdict wins and survives the tab.
- **The unreachable `reset` lever.** `handleAbandon` implements `reset` (with the
  tip-of-a-clean-tree verification), but the abandon dialog offers only `revert`.
  Surface `reset` as a second unticked lever, shown only when the work is verifiably
  the tip — both specified levers reachable from the browser.
- **The read-scope diff swallows its error.** `handleTicketDiff` at `scope=read`
  with a stale or unknown `since` sha discards the git error (`patch, _ :=`) and
  renders "Nothing changed in this scope" — a wrong answer dressed as an empty
  diff. Return an anchored error instead.
- **Harness commits fingerprinted by subject.** `workCommits` / `isHarnessSubject`
  identify the harness's own commits by subject prefix (`Claim `, `Resolve `, …),
  so any agent commit whose message starts with one is silently excluded from the
  revert/reset set. Match the harness's own commit **trailers** instead.
- **Cosmetic.** `demoteProposal` on a second abandonment inserts the new
  `### Rejected` section at the current proposal's position rather than grouping it
  under `## Rejected attempts`; the record can scatter.

## Done when

Every exit of the hub is driven for real in Chrome — approve (with and without the
rejection tick), send back, take it further, and abandon with each lever — and the
post-approve strip **survives** the approve: it renders the promotion result and
any `smearedInto` warning, and its Next button spawns-or-navigates to the
suggestion after its enabling delay. A re-review after a send-back shows the *new*
verdict at the gate, not the superseded one, and survives an orphaned/discarded
review tab. The abandon dialog reaches `reset`. A `scope=read` diff with an unknown
`since` returns an anchored error, not an empty diff. The revert/reset levers act
on trailer-identified work commits, never subject-matched ones. Process-boundary
tests cover the re-review-supersedes-stale case, the read-scope error, and the
trailer-based work-commit selection; the browser exits are added to whatever
by-eye check the repo adopts for the island/hub surfaces.

<!--
Source: the agent review of ticket 12 (verdict: fail — the browser clause unmet;
blocking finding plus five advisories). This ticket is that verdict's fix-up,
tracked as its own unit rather than folded back into 12. The verdict and assembled
brief lived at .wayfinder-harness/run/s1433d0d2444e/ (gitignored, per-session).
-->
