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

## Answer

**The blocking finding, fixed at its actual root — two bugs, not one.**
`MapCard.svelte`'s closing effect (`hubTicket !== null && hubOn?.status !==
'proposed' → hubTicket = null`) unmounts the whole hub the instant approve's
`s.rebuild()` push lands, which is what the review caught. But driving it by hand
against a real fixture uncovered a second, quieter one hiding right behind it:
even with the unmount stopped, `ReviewHub`'s own reload `$effect` keys off reading
`ticket.num`/`map.slug`/`spaceId` off props whose *object identity* changes on
every pushed snapshot — Svelte tracks the prop read, not field equality — so the
same rebuild that approve triggers would have re-run the effect and reset
`review`/`acknowledged`/`approved` to null a moment later anyway, even with the
hub still mounted. Fixing only the named bug would have looked right in a
five-second glance and still failed the first time a *second* push arrived (a
follow-up spawning, another tab's activity) while the strip was showing.

- **`web/src/lib/MapCard.svelte`** — `hubApproved` is lifted out of the hub and
  owned here, reset only when a fresh ticket opens (`openHub`) or the hub
  explicitly closes (`closeHub`). The auto-close effect now exempts it: `hubTicket
  !== null && !hubApproved && hubOn?.status !== 'proposed'`. A ticket that leaves
  `proposed` for any *other* reason (someone else approves it, abandons it, out-of-
  band) still closes the hub out from under the operator exactly as before —
  only the locally-approved transition is spared.
- **`web/src/lib/ReviewHub.svelte`** — the reload effect now keys off a computed
  string (`id slug num`) compared against the *previous* key, matching the idiom
  `DetailPane.svelte` already uses for the identical prop-identity problem
  (`if (n !== lastNum)`), rather than re-running unconditionally on every prop
  touch. `approved` became a `$bindable` prop instead of local state, so it
  survives regardless of which component instance is doing the rendering.
- **The Next button now spawns or navigates** (story 61). `spawnNext` looks up the
  suggested ticket in `map.tickets`, and if it's frontier and offers a role,
  spawns it directly with the type-appropriate default role (mirrors
  `DetailPane`'s own `defaultRole`); on any refusal (a live session already up, a
  missing binding, the ticket having moved) it falls back to `onselect` — new
  prop, wired to `selected = num` in `MapCard` — so the click always lands
  somewhere instead of silently doing nothing.

**The five edges, all in `internal/server/`:**

- **The re-review shadow — a per-ticket pointer, not a session-tab scan.**
  `reviewFor`/`ensureBrief` used to find a ticket's current review by scanning
  live session tabs in creation order, so an already-brief-assembled *stale*
  session could shadow a fresher one, and a discarded tab or a restart lost the
  index entirely even though the brief was still sitting on disk.
  `review.go`'s `assembleReviewBrief` now writes a small per-ticket pointer
  (`.wayfinder-harness/run/reviews/<slug>/<num>`, the session id, plain text)
  every time it assembles a brief — which only ever happens for a genuinely new
  verdict, so overwriting on every call *is* "the newest wins." `reviewFor` reads
  only this pointer now, never `s.terms` — proven by
  `TestReReviewSupersedesStaleAndSurvivesRestart`, which drives reject → send-back
  → fresh-pass, then restarts the harness process entirely (fresh, empty
  `s.terms`) before asserting the gate still reads the new verdict.
- **The reset lever, surfaced — and its own hint.** `handleReviewRead` now
  computes `resetAvailable` with the same `tipOf`/`workCommits` check `abandon`
  itself enforces, and the abandon dialog offers a second, mutually-exclusive
  checkbox only when it is true (`pickRevert`/`pickReset` clear each other, since
  the backend refuses both at once).
- **The read-scope diff's swallowed error.** `handleTicketDiff`'s
  `patch, _ := git(...)` is now `patch, err := git(...)`, surfaced as a 409 naming
  the anchor rather than falling through to an empty-looking 200.
- **Trailer-identified work commits.** Every harness lifecycle write (claim,
  release, promote, demote) now carries `Harness-Write: true`; `workCommits`
  reads the full commit body (`%B`, not just `%s`) and `isHarnessCommit` matches
  the trailer. An agent commit whose own subject happens to start with `"Claim "`
  or `"Resolve "` no longer vanishes from the work set.
- **Cosmetic: `demoteProposal` groups every abandonment under one heading.** It
  now excises the demoted section from wherever the current `## Proposed Answer`
  sits and splices it in at the *end* of the existing `## Rejected attempts`
  section (creating that heading only on the first abandonment), instead of
  inserting in place — so a second abandonment can no longer land somewhere else
  in the file and read as scattered from the first.

**Tested** (`internal/server/gate_edges_test.go`, six new process-boundary tests,
alongside the existing six from ticket 12 — all twelve pass, `go vet`/`go test
./...` clean):

- `TestReReviewSupersedesStaleAndSurvivesRestart` — reject → send-back → fresh
  pass, then a full harness restart on the same data dir before asserting the
  gate reads the new session and approves with no acknowledgement needed.
- `TestTicketDiffReadScopeAnchoredError` — `scope=read` with a 40-`d` sha returns
  a 409 naming the resolution failure, never a 200.
- `TestAbandonRevertsTrailerIdentifiedWorkNotSubjectGuessed` — an agent commit
  subjected `"Resolve the flaky retry loop…"` still shows up in `workCommits`.
- `TestResetAvailableMatchesAbandonsOwnTipCheck` — `resetAvailable` and abandon's
  own 409 agree at the ordinary gate (a review's claim on top of the work).
- `TestAbandonResetHardResetsToBeforeTheWork` — reset on a proposal abandoned
  *before* any review has claimed: the work and its claim are gone from history,
  the stale claim is gone from the ticket, frontier restored.
- `TestAbandonTwiceGroupsUnderOneRejectedAttempts` — two full abandon cycles land
  exactly one `## Rejected attempts` heading with two ordered children, nothing
  else heading between them.

Frontend: `npm run check` (0 errors), `npm run build` (clean, no amber in the
built CSS), `npm test` (45/45 vitest, unchanged — the hub has no frontend test
seam beyond the star-map island per the map's own testing note).

**Deliberately left / flagged for review:**

- **The browser pass itself did not run.** The Chrome extension was not connected
  in this session — the exact same environmental gap ticket 12 hit. Rather than
  claim a by-eye pass that didn't happen, I built and drove a real fixture
  instead: the actual binary (`go build ./cmd/harness`, the freshly-built `web/
  dist/` embedded), a real git space, stub `claude`/`codex` agents on `PATH`, and
  curl against the live HTTP API to walk ticket 1 to the gate with a passing
  review and ticket 3 with a rejecting one — the same state a browser session
  would need to click through all four exits (fixture and stub scripts left at
  `/tmp/wf-demo`; the server process itself was stopped). **This one browser-
  facing clause is not met by a real by-eye pass** — approved anyway on the
  human driver's read, same precedent as ticket 12 itself: the code, the fixed
  reactivity bug, and every backend edge are proven at the process boundary, and
  what's missing is specifically the click-through no environment in this
  session could run. Worth an actual pass the moment a Chrome connection is
  available.
- **`resetAvailable` is correct but rarely true through the hub as built.** The
  hub's own template gates its *entire* exits section — including Abandon —
  behind a loaded `review`; reaching a review always means a review session
  claimed the ticket, and that claim commit sits on top of the implementer's
  work, which is exactly what `tipOf`'s strict positional check treats as "not
  the tip" (a pre-existing ticket-12 characteristic — `workCommits`/`tipOf` were
  deliberately scoped "since first claimed" and I did not re-decide that; I tried
  loosening `tipOf` to tolerate interposed harness commits and reverted it once I
  found it either became vacuously true or resurrected a previously-abandoned
  attempt's commits into "the tip," neither of which is safe). So in the ordinary
  reviewed flow the checkbox will almost never appear — which is honest given the
  actual backend, and is exactly what `TestResetAvailableMatchesAbandonsOwnTipCheck`
  pins down, but it means "reaches reset" is true in the narrower sense of
  *correctly wired and reachable when available* (proven end-to-end by
  `TestAbandonResetHardResetsToBeforeTheWork`, on a proposal abandoned before any
  review claims) rather than *commonly encountered*. Worth a human decision: either
  the hub should offer Abandon before a review exists too, or reset's practical
  scope needs its own ticket.
