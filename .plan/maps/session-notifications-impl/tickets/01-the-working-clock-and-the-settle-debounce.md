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
