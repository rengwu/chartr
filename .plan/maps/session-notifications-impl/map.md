# session notifications — implementation

## Destination

The [spec](../session-notifications/spec.md) implemented end to end: a session
that worked longer than a configurable *n* seconds tells the operator when it
stops — landing on `idle`, stopping on `blocked`, or dying — through an operating
system notification fired by the chartr server, with no browser tab open and no
permission grant. Flicker between tool calls collapses into one notification via a
settle delay *D*. A dot on the session's card records that it finished while the
operator was away and clears when they focus that tab. Done looks like one pure
clock feeding two consumers, a `notify.toml` beside `terminal.toml`, and
`attention.ts` untouched.

## Notes

**This map carries execution.** Every ticket is a `task` that delivers working
code, not a decision — all decisions were settled in the
[spec](../session-notifications/spec.md), which is the single source of truth
here. Do not re-litigate a decision; if implementation exposes one as wrong, raise
it rather than quietly deviating. This effort has no planning map: it was charted
from a design conversation straight into the spec.

**Per-session reading order:** the spec, then this map, then your ticket.
Vocabulary comes from `CONTEXT.md` at the repo root. The spec names the settled
seams; prefer them to line-level file paths, which go stale.

**This effort settles an open question on another map.** `agent-state-detection`
recorded **Notifications** as fog — `blocked` wants pushing, and how was
undecided. The spec answers it, and that patch is struck from that map and
recorded there as beyond its destination. Read that map's Destination and Notes
before ticket 01: the state grammar this clock consumes is its work, and its
publisher already applies hysteresis that must not be duplicated downstream.

**The clock reads published states, never raw evidence.** The publisher smooths a
positive signal differently from a bare absence and holds a startup grace. A
second notion of "working" derived downstream would disagree with the one the
sidebar shows, and the operator would be told about a run they never saw.

**Detection never enacts, and neither does this.** Nothing here kills, resumes or
requeues a session. The notification reports; the operator acts.

**The frontend rules are binding.** ADR 0010 and ADR 0012 apply and
`docs/design-system.md` is required reading before the dot: tokens for every
colour, a vendored primitive for every component, no raw hex, no amber, Phosphor
icons. The dot uses `--primary`. The star-map's status hues are not involved and
`web/src/lib/starmap/theme.ts` is not touched.

**No test shells out to a real notifier**, on any platform. The platform notifier
sits behind an interface and tests substitute a stub.

**Per-ticket checks:** `go vet ./...` and `go test ./...`, plus the frontend
`check`, `build` and `vitest` scripts for any ticket touching `web/` — the embed
test compiles against `dist/`. Grep the built CSS for amber before committing.

## Decisions so far

<!-- one line per resolved ticket: gist + link. Empty until the first ticket ships. -->

- **01 — The working clock and the settle debounce.** `runClock` in
  `internal/terminal/clock.go` is the whole rule: a pure fold over the published
  `(state, timestamp)` pairs, time as a parameter, emitting at most one
  `RunFinished` per run — tab id, space, map/ticket where it is a session, the
  reason it settled into, and a duration that excludes the *D* wait. An empty state
  moves nothing, a nil clock emits nothing (that is `enabled = false`), and
  non-positive constants clamp to the shipped defaults, `DefaultNotifyAfter` = 60s
  and `DefaultNotifySettle` = 10s. Table-tested per behaviour plus the real
  recorded Claude turn, which folds to exactly one `idle` event spanning its
  dialog and its pause. **Operator-approved:** the two defaults stand as the
  numbers ticket 02's file defaults to, and the gap below is accepted as ticket
  03's to settle rather than the spec's. Flagged for ticket 03: a run only ends on
  a *sample*, so a tab dropped from the manager never reports its last run — the
  emitting seam decides whether a drop flushes the clock.
  [ticket](tickets/01-the-working-clock-and-the-settle-debounce.md)

## Not yet specified

<!-- Empty. Every decision is settled in the spec; this map only executes it. A ticket that exposes a genuinely new question sends it back to the spec — it does not open fog here. -->

## Out of scope

- **Acting on a notification** — no auto-resume, no auto-kill, no requeue.
- **Clickable notifications and deep links** — no scheme exists for routing a
  click into a specific space and tab.
- **Browser or webview notifications** — the server-side path already covers the
  closed-tab case; a second transport adds a permission prompt and a weaker
  guarantee.
- **Remote or push delivery** — chartr is one offline binary and phones home
  nowhere.
- **Per-space, per-agent or per-role thresholds** — one machine-wide *n* and *D*.
- **A notification history, inbox or digest** — the dot is one bit that clears on
  focus.
- **Changing the attention grammar** — `attention.ts`'s flag, its `Liveness` and
  the undecided precedence between them are untouched.
- **Notifying on anything but a run ending** — no progress pings, no reminders, no
  notification on spawn.
