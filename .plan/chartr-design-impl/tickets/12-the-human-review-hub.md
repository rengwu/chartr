---
type: task
blocked_by: [11]
---

# The human review hub

## Question

The gate, whole: the brief-first hub plus all four exits. The hub takes over the map card and leads with the brief — full verdict and diff behind expanders, the diff row jumped into by the blocking finding's filename. **Approve** is one click naming its outcome: the promotion commit — its own pathspec-limited commit with trailers, never an amend — lands even while another session runs in the space, and the empty-commit attribution smear is detected and reported (ADR 0008); a rejecting verdict requires exactly one "I've read the blocking finding" tick first, and approving over a rejection is recorded in the commit. Approval unblocks dependents on the stricter frontier, and the post-approve strip suggests the next best frontier ticket with a spawn button that enables only after a short delay. **Send back** opens the briefing dialog — standard bundle plus the blocking finding always, advisories opt-in, an optional human note riding the payload and its archive, never the ticket file. **Take it further** stacks follow-up sessions on the still-proposed ticket, the `## Proposed Answer` rewritten in place, the diff offered in three scopes (all / since verdict / since last read). **Abandon** requires a reason addressed to the next attempt, demotes `## Proposed Answer` to dated `### Rejected` prose as a chartr commit returning the ticket to the frontier, and offers revert (and reset when the commits are verifiably the tip) as unticked levers. An agent-review rejection arrives as a forced-stop banner: it halts, never loops.

Done when: process-boundary tests cover approve (promotion commit shape, trailers, concurrent live session, smear detection, dependents unblocking), the rejection tick gating approve, send-back's note landing in the payload and nowhere else, take-it-further rewriting the proposed answer with commits stacking, and abandon's demotion returning the ticket to the frontier with levers untouched by default; in the browser every exit is drivable and the hub renders the on-disk brief with buttons and nothing else.

## Answer

The gate is four plain HTTP actions and one Svelte component that renders the
markdown ticket 11 already wrote to disk. Nothing about the review moved into a
store: the brief is still a file, the verdict is still its source, and the
recommendation is still mechanical.

**What I built:**

- **`internal/server/gate.go` — the four exits, all through one gate.**
  `gateTarget` resolves {space, map, ticket} fresh off disk, refuses anything but
  a `proposed` ticket, and — the one thing worth naming — *assembles the brief if
  the reviewer's verdict is on disk and nobody has built it yet*. Putting that at
  the single choke point rather than in the hub's read closed a hole I found while
  driving the real binary: without it, an unassembled verdict meant approve saw no
  blocking finding and asked for **one fewer** tick rather than one more.
  - **approve** promotes `## Proposed Answer` → `## Answer` and commits it, which
    is what unblocks the dependents. It answers with the commit, what it
    unblocked, and the next best frontier ticket (ranked: just-unblocked first,
    then by what it unblocks) for the post-approve strip. Over a rejecting verdict
    it demands `acknowledged` and records the override in the commit
    (`Approved-Over-Rejection`, `Acknowledged-Blocking: <the finding>`).
  - **send back / take it further** are one endpoint (`follow-up`) behind two
    doors: ticket 09's `launchSession` with steering added. The claim re-stamps
    onto the same ticket and `proposed` outranks `claimed` in the derived-status
    table, so the ticket stays proposed and comes back to the hub with the
    follow-up's commits stacked on the proposal.
  - **abandon** demotes the proposal to `### Rejected — <date>` under a
    `## Rejected attempts` heading, keeping the rejected prose verbatim beneath
    the human's reason, strips the claim so the ticket derives `open`, and commits
    it. `revert` and `reset` are levers, off by default; `reset` additionally
    requires the work commits to be verifiably the tip *of a clean tree* and runs
    **before** the demotion, so the record that survives is the demotion.
  - **`GET …/review`** serves the brief byte-for-byte off disk plus the structure
    the buttons key on; **`GET …/diff`** serves the work at the three scopes,
    anchored in git (the implement claim / the review claim / a sha the client
    passes), excluding the ticket file itself so claim churn is not read as work.
- **`internal/server/promote.go` — the two lifecycle writes and the smear.** Both
  are `git commit --only -- <ticket>` with trailers, never an amend. When that
  commit comes up empty the code distinguishes the two causes it can have: if the
  path is clean against HEAD, another writer swept our edit into their commit —
  reported as `smearedInto` with the carrier sha and a warning, 200, because the
  promotion *did* happen; if it is not clean, it is a real failure and surfaces as
  one.
- **`prompt.Steer` — steering that lives only in the payload.** `Bundle.Steering`
  renders as labelled context parts (the blocking finding, the advisories the
  human ticked by index, their note), so they appear in the payload preview and
  the archive with provenance, and *never* touch the ticket. Only abandonment
  writes to the ticket, because only abandonment needs the next fresh attempt to
  read it.
- **`model.Ticket.Review` — the gate signal on the snapshot.** Set exactly when a
  brief is assembled and waiting. This is the explicit signal ticket 13 flagged:
  `starmap/session.ts`'s `stateOf` now reads it instead of inferring human review
  from "a review session has exited", and ticket 14's queue has its gate-level
  fact without a second store.
- **`web/src/lib/ReviewHub.svelte` — the hub, variant D.** It takes over the map
  card (the map is context at the gate). Brief-first: what was done, what the
  reviewer found with the blocking finding inline, and the mechanical
  recommendation; the full verdict and the diff (with the scope bar) are behind
  expanders; `{ } raw` shows the exact file, which is the TUI-parity promise made
  visible. The blocking finding's filename jumps into the expanded diff. Approve
  is one click, plus one tick over a rejection; the post-approve strip replaces
  the hub with a spawn button that enables after 450ms. Three dialogs on the
  vendored `Dialog`; one new primitive (`checkbox`, lucide swapped for Phosphor,
  `@lucide/svelte` pruned). Tokens only — the diff's one chromatic tint is
  `--destructive` on deletions, with the `+`/`-` prefix as the non-colour channel.
- **A real bug fixed on the way:** `gitDirty` ran `git status --porcelain` on
  every rebuild, which takes `index.lock` to write its stat cache. A lifecycle
  write touches a watched ticket, which fires a rebuild, which raced the gate's
  own `git add`. Every read-only git call chartr makes now runs under
  `--no-optional-locks`. It failed one full-suite run before I found it.

**How each Done-when clause is met** (`internal/server/gate_test.go`, six
process-boundary tests; each walks a real space to the gate — proposed by a stub
implementer, reviewed by a stub reviewer, verdict written, brief assembled — then
drives one exit):

- *promotion commit shape, trailers, concurrent live session, dependents
  unblocking* — `TestApprovePromotesUnblocksAndCommitsNarrowly` asserts the
  promotion's parent is the previous HEAD (never an amend, the old sha still in
  history), that it touches only the ticket file while a *staged* debris file
  stays staged and uncommitted, that the trailers are present, that the reviewer
  is live before and after, and that ticket 02 reaches the frontier with
  `unblocked: [2]` and the strip's suggestion pointing at it.
- *smear detection* — `TestApproveDetectsTheAttributionSmear` drives the race for
  real rather than simulating it: a repository `pre-commit` hook (the operator's
  own git, not a chartr seam) commits the whole tree out from under chartr's
  commit. chartr's commit comes up empty, `smearedInto` names the hook's
  commit, the warning reaches the operator, and the ticket still resolves.
- *the rejection tick gating approve* — `TestApproveOverRejectionNeedsTheTick`:
  without it, 409 naming the blocking finding, HEAD unmoved, ticket still
  proposed; with it, 200 and the override in the commit.
- *send-back's note landing in the payload and nowhere else* —
  `TestSendBackNoteRidesThePayloadAndNowhereElse` asserts the note and the
  blocking finding in the payload and in the archive, and absent from the ticket
  on disk *and* from `git log -p` over the ticket.
- *take-it-further rewriting the proposed answer with commits stacking* —
  `TestTakeItFurtherStacksCommitsAndRewritesTheProposal` (new
  `chartrtest.StubRewritingAgent`) asserts exactly one `## Proposed Answer`
  heading carrying the new prose, the prior text gone from the file but
  recoverable from `git log -p`, both commits in history, and the ticket still
  proposed.
- *abandon's demotion returning the ticket to the frontier with levers untouched*
  — `TestAbandonDemotesToTheFrontierAndDestroysNothing` asserts open + on the
  frontier (dependent still blocked), the dated `### Rejected` section with the
  reason and the proposal verbatim, no surviving claim, one pathspec-limited
  commit with trailers, and every pre-existing commit still in history with no
  revert.
- *the hub renders the on-disk brief* — `TestHubReadsTheBriefOffDisk` asserts the
  served brief is byte-identical to `brief.md` and that the snapshot's gate signal
  matches.

**Tested:** `go vet ./...` and `go test ./...` pass (the server suite three times
over, after the lock race); `svelte-check` 0/0, `vitest` 45, the Vite build; no
amber in the built CSS.

**Driven for real:** every exit was run against the real binary on a fixture space
— approve refused without the tick, then approved over the rejection with the
override trailers; send back with a note and a ticked advisory landing all three
steering blocks in the payload; abandon writing the dated rejection and returning
the ticket to the frontier; the diff at `scope=all` resolving its anchor from the
claim trailers.

**Deliberately left / flagged for review:**

- **The hub is unverified by eye.** No browser was drivable from this session (the
  Chrome extension was not connected), so the endpoints behind every button are
  proven and the component compiles, type-checks and builds — but nobody has
  *looked* at it. Same standing caveat as tickets 06, 07 and 13, and this one is
  the largest new surface yet. Worth one browser pass before resolving.
- **Send back and take it further are one endpoint.** The ticket names them as two
  exits and the hub presents two doors, but mechanically they are the same act:
  another session on the same proposal with different steering attached. I did not
  invent a second path to make the prose literal.
- **There is no `fix-up` role.** The prototype labels the send-back session
  "fix-up"; the role set is closed (ADR 0002 / `config.Roles`), so it spawns as
  `implement` with the finding attached. Take-it-further offers implement or
  review; the prototype's "advice only, nothing accumulates" mode has no home in
  a model where every session claims a ticket, so I left it out rather than fake
  it.
- **The brief is now assembled on demand.** Ticket 11 made assembly an explicit
  operator action and flagged it. Opening the hub (or taking any exit) is itself
  an operator action, and the assembly is a pure function of the verdict and the
  ticket, so I made the gate build it rather than hide the whole hub behind a
  second button. The explicit `POST …/review-brief` still exists and is unchanged.
- **"Jump to the finding's file" is a regex.** The verdict format anchors findings
  to *clauses*, not to file positions, so the hub pulls a path-shaped token out of
  the finding's prose and matches it against the diff's file headers. It opens the
  diff either way. If review wants a real jump, the verdict format needs a
  location field — a change to ticket 11's contract, not this one.
- **`reviewFor` is keyed by the review session's tab.** A brief whose session tab
  has been discarded (a restart, an operator closing it) is orphaned: the ticket
  reads as plain `proposed` again and a fresh review must run. Honest against the
  no-second-store rule, and it is what I hit while driving the demo. If that
  proves annoying, the fix is a per-ticket brief path rather than a per-session
  one — a change to where ticket 11 writes.
- **`workCommits` identifies chartr's own commits by subject prefix**
  (`Claim `, `Release `, `Resolve `, `Abandon the proposal on `). Trailer-based
  matching would be sturdier; the subjects are chartr's own and stable, so I
  took the simpler read.
- **Not built here:** the "Needs you" queue and the action-station badge (ticket
  14). The gate signal they read now exists on the snapshot.
