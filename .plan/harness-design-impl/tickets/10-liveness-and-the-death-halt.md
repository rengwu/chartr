---
type: task
blocked_by: [09]
---

# Liveness and the death halt

## Question

`observe` is `{alive, dead}` read from the PTY, and the harness surfaces but never acts (planning ticket 02). Working, quiet, dead enter the snapshot: quiet is a hint shown only for AFK ticket types silent past a threshold with no `## Proposed Answer` — an idle HITL session shows nothing. A death halts to the human: the dead session stays pinned to its ticket with scrollback preserved, and the operator chooses resume (same-ticket crash recovery only — ADR 0005 as amended), respawn fresh, or release the claim back to the frontier; the stale claim stands until they act. No auto-kill, no enforced timeout, no auto-requeue. A dirty working tree — from a session or an ad-hoc shell — is a badge, never a spawn gate.

Done when: process-boundary tests with stub agents assert dead is detected on exit, the halt offers exactly the three choices and takes none itself, scrollback survives death, quiet appears only for the AFK case past threshold, and a dirtied tree badges while spawn still proceeds; the absence of autonomous action is asserted, not assumed (no state change without an operator HTTP call).

## Answer

Liveness and the death halt land on top of ticket 09's session seam without a new subsystem: a session tab is still an ad-hoc tab that carries a `Session`, and everything here rides the multiplexer's existing liveness/scrollback/attach plumbing. Nothing is enacted — the harness surfaces `working` / `quiet` / `dead`, pins a death, and waits for an operator HTTP call.

**A session that dies stays pinned; the harness takes nothing.** The terminal read loop already marked a dead tab `alive=false`; the change is the cleanup callback (`terminal.Manager.onExit`). An ad-hoc shell — or a session the operator explicitly ended — drops from the model as before; a session whose process *exited on its own* stays put, dead, bound to its ticket, its scrollback buffer intact (`pinOnDeath = session != nil && !killed`). The one new bit of state is a `killed` flag `Close`/`Shutdown` set, so a natural death and an operator dismissal are told apart. `Get`/`Attach` still find the pinned tab, so the terminal socket replays its scrollback after death (`TestDeadSessionHaltsPinnedWithScrollback`). Because the read loop only ever *marks* dead and the sampler only *reports*, no state moves without a call — asserted by re-reading the snapshot, the ticket status (`claimed`), and the commit count (`1`, the claim alone) across a window.

**Exactly three choices, each an HTTP action (`internal/server/halt.go`).** `POST …/sessions/{sid}/resume|respawn|release`, each requiring the session be dead first (a live one is refused — `TestHaltRefusesLiveSession`):
- **resume** — same-ticket crash recovery (ADR 0005 as amended): relaunch the *same* session id on its own ticket, re-materializing the archived payload the opener points at; the claim stands (no new commit — `TestHaltResumeRelaunchesSameSession`).
- **respawn** — a fresh session on the same ticket: a new claim supersedes the stale one, re-stamped in place as its own pathspec-limited commit, and a fresh payload composed, so nothing carries across (`TestHaltRespawnStartsFreshOnSameTicket`).
- **release** — clear the claim back to the frontier: `writeReleaseCommit` strips `claimed_by`/`claimed_at` and commits only the ticket, so it derives `open` and takeable again (`TestHaltReleaseReturnsTicketToFrontier`).

The post-gate spawn mechanics (compose → claim → payload → archive → `OpenSession`) were factored into `Server.launchSession`, shared by spawn and respawn; `stampClaim` is now idempotent (strips any existing claim first) so a respawn re-claim never doubles the keys.

**Quiet is the AFK-only hint.** The terminal samples output silence (`lastActivity`, refreshed on every broadcast) and reports a raw `Silent` verdict for a live session; the *server* — where the role and the ticket's derived status are both in hand — turns that into `quiet` only for an AFK role (`config.RoleIsAFK`; `grill` is the sole HITL role) whose ticket carries no `## Proposed Answer` yet. An idle grilling session shows nothing, and once an AFK ticket is proposed its silence is expected and the hint withdrawn (`TestQuietOnlyForAFKPastThreshold`, driving both across the `WithQuietAfter` threshold knob). The threshold is a real `Options.QuietAfter` config value (a calm 45s default), tuned down in tests — not a test-only seam.

**A dirty tree badges, never gates.** `gitDirty` (`git status --porcelain`) feeds `Space.Dirty`; spawn never consults it, so a spawn into a dirtied tree still proceeds (`TestDirtyTreeBadgesButSpawnProceeds`).

**Frontend** (design-system compliant — tokens/primitives/Phosphor): the session-status indicator gained `quiet` (a slow, dimmed crawl) and `dead` (a frozen grey mark); a dead session row offers exactly the three halt buttons in place of the End affordance; the space footer carries a dirty badge. `web/src/lib/{model,actions}.ts` mirror the new status values, `Space.dirty`, and the three actions.

**Tested:** `go vet ./...`, `go test ./...` (7 new process-boundary tests in `halt_test.go`, all on the public seam — snapshot, filesystem, git); frontend `svelte-check` (0/0), `vitest` (33), and the Vite build all pass; no amber in the built CSS. Stub agents grew `StubDyingAgent` (emits a marker, then exits on cue).

**Deliberately left / flagged for review:**
- **AFK vs HITL** — the spec names only `grill` as HITL, so I classified every other role (prototype, research, implement, review) AFK. If prototype is meant to be interactive, that is a one-line change in `RoleIsAFK`.
- **Operator-ended live session vs death** — the halt is scoped to *death* (process exit). An operator hitting the End (`✕`) on a *live* session dismisses it (killed → dropped), matching ad-hoc shells and ticket 09's close semantics, which leaves its claim standing with no halt UI to release it. This is the accepted edge of a hard-kill; a natural interrupt (typing `/exit` into the TUI) instead flows through the death halt. Flagged in case review wants End on a live session to route through the halt.
- **release as a committed lifecycle write** — ADR 0008 enumerates claim/promotion/demotion; releasing a stale claim is the natural inverse of the claim, so I recorded it as its own append-only commit (git stays the whole audit trail: history reads claim → release). Flagged since it is not literally in the ADR's list.
- Resume/respawn/release of a session live-again elsewhere is guarded by `HasLiveSession` + `OpenSession`'s own re-check (one live session per space holds).
