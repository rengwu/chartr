---
type: task
blocked_by: []
claimed_by: s8bbabd85b85f
claimed_at: 2026-07-30T17:07:07Z
---

# The working clock and the settle debounce

## Question

Build the rule, and only the rule: a pure state machine that consumes the terminal
states the publisher already emits and produces at most one `finished` event per
run. No transport, no configuration file, no UI — ticket 02 gives it its
constants, tickets 03 and 04 consume its events. Everything downstream depends on
this being right, and being testable without a PTY.

**Read the published states, not the evidence.** The machine sits downstream of
`publish.go` and folds `(state, timestamp)` pairs. The publisher already applies
asymmetric hysteresis and a startup grace; deriving a second notion of "working"
here would disagree with the one the sidebar shows, and the operator would be told
about a run they never saw.

**The rule, precisely.** A *run* begins the first time a terminal publishes
`working`. It ends at the last moment the terminal was `working` before staying
out of `working` continuously for *D*. Re-entering `working` before *D* elapses
cancels the pending end and the run continues; any other state merely updates the
reason that will be reported. On end, the run's duration is the span from its
beginning to its end — the *D* wait is *not* counted — and an event is emitted
only if that duration is at least *n*. The reason is the state the terminal
settled into: `idle`, `blocked`, `dead` or `exited`.

**One event carries everything both consumers need**: terminal id, space, the
session's map and ticket where the tab is a session, the reason, and the duration.
Neither consumer re-derives any part of the rule. A tab that is not a session
still produces events — an ad-hoc shell running a long build is a run — and
carries no map or ticket.

**Purity is the deliverable.** Time arrives as a parameter, never from a clock
read inside the fold, so the table test drives ten minutes of history in
microseconds and no test sleeps. *n* and *D* are parameters here; ticket 02 wires
them to a file and this ticket ships defaults.

Tests lead, as a table over `(state, timestamp)` sequences — the same shape as the
rule-engine table test in `internal/terminal/detect`. Cases, each named for the
behaviour it pins: a run past *n* fires exactly once; a run under *n* never fires;
a run broken by dips shorter than *D* fires once, with the duration spanning the
dips; a gap longer than *D* followed by more work is two runs; each of `idle`,
`blocked`, `dead` and `exited` is reported as its own reason; a run that settles
`blocked` and re-enters `working` before *D* does not fire on that block; a
terminal that never reaches `working` never fires. Where the recorded fixtures
under `.plan/maps/agent-state-detection/assets/` contain a real working-to-idle
turn, replay it rather than hand-writing the sequence — real bytes are already the
standard on that map, and `recording_test.go` carries the loader.

Done when: the machine is a pure fold with time as a parameter; the table test
covers every case above and no test sleeps; a real recorded Claude turn produces
exactly one event with the reason `idle`; `go vet ./...` and `go test ./...` pass.
No configuration file, no notification, no model change in this ticket.

## Answer

The rule ships as `runClock` in `internal/terminal/clock.go`, beside `publish.go`
and downstream of it, with `RunFinished` as the one event both later consumers
read. `update(state string, now time.Time) *RunFinished` is the whole surface: it
folds one published state at one instant and returns the run that ended on that
sample, or nil. Nothing else was added — no transport, no file, no model field, no
wiring into `sample()`; ticket 02 gives the clock its constants and 03/04 consume
its events.

**The rule, as written.** A run begins on the first published `working`; every
`working` sample updates `lastWorking` and clears any pending reason; any other
state records itself as the reason and, once `now - lastWorking >= settle`, ends
the run at `lastWorking`. The duration is `lastWorking - start`, so the settle wait
is never counted, and the event is returned only when that duration is at least
`after`. `running` is cleared before the `after` test, so a short run ends silently
rather than lingering into the next one — that is what makes "two runs" and "one
run spanning its dips" the same code path.

**Each Done-when clause.**

- *A pure fold with time as a parameter.* `runClock` reads no clock; `update` takes
  `now`. The only state it holds is the run in progress (`running`, `start`,
  `lastWorking`, `reason`) plus the tab's fixed identity and the two constants. The
  whole table test runs in well under a millisecond.
- *The table covers every case named, and no test sleeps.* `TestRunClockRule` in
  `clock_test.go` is a table over `(state, timestamp)` sequences — the shape of
  `detect`'s rule-engine table — with one case per behaviour the ticket lists: past
  *n* fires exactly once (samples continue past the event and produce nothing);
  under *n* never fires; dips shorter than *D* are one run whose duration spans
  them (50s across three working stretches); a gap longer than *D* is two runs with
  two events; `idle`, `blocked`, `dead` and `exited` each report as their own
  reason; a block resumed before *D* fires nothing and the run later settles
  `idle`; a tab that never works never fires. Two more rows pin the edges I had to
  decide: an empty state moves nothing, and the identity test asserts a session's
  event carries its map and ticket while an ad-hoc shell's carries neither.
- *A real recorded Claude turn produces exactly one event with the reason `idle`.*
  `TestRunClockOnRecordedClaudeTurn` replays `rec-claude.jsonl` through the real
  scanner, grid, rule engine and publishing hysteresis, and folds what the tab
  publishes into the clock exactly as the sampler will. The capture publishes
  `working 0s → idle 2.7s → working 20.4s → idle 30.9s → working 42.3s →
  blocked 44.7s → working 59.1s → idle 61.8s`; with *D* = 25s and *n* = 30s that is
  one run ending at ≈61.5s, reported once, as `idle`, having run ≈61s — spanning
  the permission dialog and the pause rather than counting only the 2.4s stretch
  after the dialog cleared.
- *The checks.* `go vet ./...` and `go test ./...` pass. No `web/` change, so the
  frontend scripts and the amber grep are not implicated.

**Four judgement calls, all reversible.**

1. *An empty state moves nothing.* A tab that has not been sampled yet publishes
   `""`. Treating it as "not working" would end runs on the absence of a reading,
   so it is folded as a no-op — the absence of a state, not an exit from one.
2. *A nil clock folds nothing.* `update` guards a nil receiver, so ticket 02 can
   express `enabled = false` as *no clock* rather than a flag each consumer
   remembers to check, which is what the map asks for. Nothing depends on it yet.
3. *Non-positive constants fall back to the defaults.* Ticket 02 validates the
   file, so a zero `settle` reaching `newRunClock` is a defect — and a zero
   `settle` fires on every dip, which is the loudest possible failure. It clamps
   instead, in the constructor, where it is one line and visible.
4. *The clock is constructed with the tab's identity and emits complete events.*
   The alternative — emit reason plus duration and let the caller attach the tab —
   would put half the event's assembly at the seam that ticket 03 writes. Holding
   `id`, `spaceID` and `*Session` (all immutable after a tab starts) keeps the
   whole event in one place, which is what "neither consumer re-derives any part of
   the rule" reads as here.

**Defaults shipped**, as constants for ticket 02 to override: `DefaultNotifyAfter`
= 60s (*n*), `DefaultNotifySettle` = 10s (*D*). The spec names neither number.
60s is the shortest run I would want interrupting for; 10s comfortably clears the
few-second dips Claude shows between tool calls in the recording (the longest
non-working gap inside its single turn is 17.7s, which is a real pause rather than
flicker — a *D* below that would split the recorded turn into two runs, and one of
them would be under any sane *n*). If ticket 02's file wants different numbers,
these are the two lines to change.

**One thing to flag, for ticket 03 rather than for this map.** The clock only ever
learns of an ending on a *sample*, so a run ends when a sample arrives at least *D*
after the last `working` — not on a timer. That is right for `idle` and `blocked`
(the sampler keeps ticking), and right for `dead`/`exited` too, since `sampleGone`
keeps being called for a pinned dead session. But a tab that is *removed* from the
manager stops being sampled, and its last run will never be reported. An ad-hoc
shell whose process exits drops from the model, and a session the operator closes
drops too; either could swallow a qualifying run's notification. Ticket 03 owns the
emitting seam and should decide whether a tab being dropped flushes its clock. I
did not build a flush here because it is transport-shaped, not rule-shaped, and
this ticket is the rule.

**Test-file change worth naming:** `recording_test.go`'s replay loop grew a
per-tick visitor (`replayPublished`) and a `tail` parameter, with `replayScreen`
rewritten as a thin caller of it. The clock needs the state at *every* tick, not
just the ticks that changed it, and it needs sampling to continue past the last
recorded byte — real sampling does not stop when an agent stops writing. No
existing test's behaviour changed.
