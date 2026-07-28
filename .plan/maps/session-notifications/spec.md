# Session notifications

## Problem Statement

An operator who spawns a session and walks away has no way to learn that it
finished. The cockpit reports state well — a tab reads `idle`, `working`,
`blocked` or `dead`, and the star-map draws it — but every one of those signals
requires the operator to be looking. The long run is exactly the one they are not
looking at: they started it *because* it would take minutes, and they went
elsewhere. They come back on a guess, and the guess is either too early or far
too late.

The two endings that most deserve interrupting them are the two that are quietest
today. A session stopped on a permission prompt (`blocked`) is waiting on the
operator and will wait forever. A session whose process died mid-run
(`dead`) has stopped without finishing, and nothing about the sidebar shouts. The
`agent-state-detection` map recorded this gap deliberately rather than guessing at
it: `blocked` is named there as the state an operator wants pushed to them, and
how it should be pushed was left unanswered.

## Solution

A session that worked for longer than a configurable *n* seconds tells the
operator when it stops — wherever the operator is, and whether or not the cockpit
is open.

One rule covers every ending. The trigger is not "a session finished"; it is "a
session that had been working for a while stopped working." Landing on `idle`,
stopping on `blocked`, and dying all fire the same notification, differing only in
what it says. That keeps the rule small enough to hold in the head and makes a
silent hole impossible: there is no ending that falls between the cases.

Two constants shape it. *n* is the threshold — work shorter than *n* is not worth
interrupting anyone over. *D* is a settle delay: an exit only counts once the
session has stayed out of `working` for *D* seconds, so the constant flicker of an
agent between tool calls collapses into one notification at the real end of the
run rather than a burst of them through it.

The notification is fired by the chartr server itself, through the operating
system, so it arrives with no browser tab open, no permission grant, and no
webview. A dot on the session's card is the quieter half: it records that the
session finished while the operator was elsewhere, and clears when they look at
it.

## User Stories

1. As an operator, I want to be told when a long-running session finishes, so
   that I can walk away from it without guessing when to come back.
2. As an operator, I want that notification to reach me when the cockpit's browser
   tab is closed, so that walking away actually means walking away.
3. As an operator, I want to be told when a session stops on a permission prompt,
   so that it is not sitting there blocked while I wait for it to finish.
4. As an operator, I want to be told when a session dies mid-run, so that a crash
   does not read as work still in progress.
5. As an operator, I want the notification to name the space and the ticket, so
   that I know which of my sessions it is about without opening anything.
6. As an operator, I want the notification to tell me how long the session ran, so
   that I can judge whether the result is worth reading immediately.
7. As an operator, I want short runs to stay silent, so that the feature does not
   interrupt me over work I was watching anyway.
8. As an operator, I want a single notification per run rather than one per tool
   call, so that a long session does not turn into a stream of alerts.
9. As an operator, I want to set the threshold, so that it matches what counts as
   "long" for the work I do.
10. As an operator, I want to turn notifications off entirely, so that I can work
    without them when I am sitting in front of the session anyway.
11. As an operator, I want a dot on the session's card when it finished while I
    was away, so that returning to the cockpit tells me what happened without
    relying on a notification I may have missed.
12. As an operator, I want that dot to clear when I look at the session, so that
    it never becomes a list of stale marks to maintain.
13. As an operator, I want the dot to survive a browser reload, so that it records
    what happened rather than what this page load happened to see.
14. As an operator on a machine with no notification daemon, I want the cockpit to
    carry on working normally, so that a missing system tool degrades the feature
    rather than the app.
15. As an operator, I want a malformed `notify.toml` to fall back to defaults with
    a warning, so that a typo never breaks the cockpit.
16. As an operator, I want the dot to sit alongside the existing status signals
    rather than replacing them, so that the sidebar grammar I already read is
    unchanged.

## Implementation Decisions

**The clock is a pure function, and it is the whole of the rule.** A state
machine consumes the terminal states the detection layer already publishes and
emits at most one `finished` event per run. It is pure — a fold over
`(state, timestamp)` pairs — so it is table-testable against recorded sequences,
exactly as the detection rule engine in `internal/terminal/detect` is. It lives
beside `publish.go`, downstream of the publisher, and reads published states
rather than raw evidence: hysteresis is already the publisher's job and must not
be done twice.

**The rule, precisely.** A *run* begins the first time a terminal publishes
`working`. It ends at the last moment the terminal was `working` before staying
out of `working` continuously for *D*. Re-entering `working` before *D* elapses
cancels the pending end and the run continues; any other state simply updates the
reason that will be reported. On end, the run's duration is the span from its
beginning to its end — the *D* wait is not counted — and an event is emitted only
if that duration is at least *n*. The reason carried is the state the terminal
settled into: `idle`, `blocked`, `dead` or `exited`.

**One event carries everything the surfaces need**: the terminal id, the space,
the session's map and ticket where the tab is a session, the reason, and the
duration. The OS notification and the dot are two consumers of one event; neither
re-derives the rule.

**The notification is fired server-side, per platform, best-effort.** macOS shells
out to `osascript`, Linux to `notify-send`, Windows to a PowerShell toast. All
three are `exec`, not linked libraries, so the cgo-free single supported artifact
(ADR 0011) is unaffected. A missing binary, a non-zero exit, or a machine with no
notification daemon logs once and is otherwise ignored — the feature degrades, the
cockpit does not. The notifier sits behind a small interface with the platform
choice made once at construction, so tests substitute a stub and never shell out.

**Notification content.** Title names the space; body names the ticket where there
is one, the reason in the operator's words ("finished", "needs you", "crashed",
"exited"), and the duration. Clicking the notification does nothing — routing a
click back into a specific cockpit view needs a deep-link scheme that does not
exist, and inventing one here would outweigh the feature.

**`notify.toml` is a per-machine file beside `terminal.toml`.** Keys: `after`
(*n*), `settle` (*D*), and `enabled`. It follows `terminal.toml`'s contract
exactly — never committed, never per-space, read into the model snapshot, and a
bad value dropped with a warning through the same warnings surface while its
default stands. It is surfaced in the existing config surface as
read-value-plus-open-file, never as a second config store. It is deliberately not
merged into `terminal.toml`, which is scoped to terminal *customization*, nor into
`user.toml`, which ADR 0009 reserves for execution choices.

**One threshold for the whole machine.** *n* and *D* are not per-space or
per-agent. A per-space threshold is a defensible want, but it belongs to the
config layering question, not to this effort, and shipping one global value first
is what tells us whether the want is real.

**The dot is per-tab server state, and it does not touch the attention grammar.**
`model.Terminal` gains a boolean recording that the tab finished a qualifying run
that the operator has not yet looked at. It renders on the session's card only.
`attention.ts` is untouched: its single `halt` flag, its separate `Liveness`, and
the precedence between them that the module explicitly leaves undecided all stay
exactly as they are. A space row shows nothing new, which is a knowingly accepted
limit — a collapsed space does not surface the dot, and the OS notification is
what covers that case.

**The dot clears on focus.** The client posts to a small per-terminal endpoint
when the operator focuses that tab, and the flag clears in the snapshot. Keeping
the flag server-side rather than in client state is what makes story 13 work: the
notification may have fired while no browser was attached at all, and a reload
must still show the dot. There is no manual dismiss and no clear-all; focusing is
the acknowledgement.

**The dot's styling is tokens and primitives.** Chrome rules apply in full
(ADR 0012, `docs/design-system.md`): no raw colour, Phosphor icons, no amber. The
mark uses `--primary`, the emphasis role the chrome already reserves. The
star-map's status hues are not involved and `web/src/lib/starmap/theme.ts` is not
touched.

**This effort settles an open question on another map.** `agent-state-detection`
carries a *Not yet specified* patch titled **Notifications**, recording that
`blocked` wants pushing and that how it folds into the existing grammar was
undecided. This spec answers it. The patch is struck from that map — a question
tracked both sharply and vaguely rots in its vague copy — and recorded there as
beyond that map's destination, which is detection rather than action.

## Testing Decisions

A good test here asserts what the operator experiences — a notification fired, or
not, with the right reason — never the internals of the timer. Two seams carry
almost all of it, and both have prior art.

- **The clock** is table-tested as a pure fold over `(state, timestamp)`
  sequences, the same shape as the rule-engine table test in
  `internal/terminal/detect`. Cases: a run past *n* fires once; a run under *n*
  fires never; a run broken by dips shorter than *D* fires exactly once with the
  full duration; a dip longer than *D* followed by more work is two runs; each of
  `idle`, `blocked`, `dead` and `exited` is reported as its own reason; a run
  that ends `blocked` and resumes working before *D* does not fire on the block.
  Where the recorded fixtures under `.plan/maps/agent-state-detection/assets/`
  contain a real working-to-idle turn, replay it rather than hand-writing the
  sequence — real bytes are already the standard on that map.
- **`internal/server`** carries the process-boundary test with a stub notifier
  substituted for the platform one: a tab running a stub agent that works past a
  short configured *n* produces exactly one notification carrying the right space,
  ticket and reason; one that works briefly produces none; `enabled = false`
  produces none. The same seam asserts the dot: it appears in the model snapshot
  when the event fires and clears after the seen endpoint is posted.
- **`internal/config`** tests `notify.toml` parsing the way `terminal.toml` is
  already tested — defaults when absent, defaults plus a warning for each
  malformed value, and the resolved values reaching the model snapshot.
- **`web`** gets vitest coverage of the pure derivation that decides whether a
  card shows the dot, following the existing pure-helper tests in
  `web/src/lib`. No test shells out to a real notifier on any platform.
- The full bar is unchanged: `go vet ./...`, `go test ./...`, frontend `check`,
  `build` and `vitest`, no amber in the built CSS.

## Out of Scope

- **Acting on a notification.** No auto-resume, no auto-kill, no requeue. The
  notification reports; the operator acts. This upholds the same boundary
  `agent-state-detection` drew.
- **Clickable notifications and deep links.** No scheme exists for routing a click
  into a specific space and tab, and inventing one is a larger effort than this.
- **Browser or webview notifications.** The server-side path covers every tier
  including the closed-tab case, so a second transport would add a permission
  prompt and a weaker guarantee for no gain.
- **Remote or push delivery.** chartr is one offline binary and phones home
  nowhere; nothing here changes that.
- **Per-space, per-agent, or per-role thresholds.** One machine-wide *n* and *D*.
- **A notification history, inbox, or digest.** The dot is a single per-tab bit
  that clears on focus, not a list.
- **Changing the attention grammar.** `attention.ts`'s flag, its `Liveness`, and
  the undecided precedence between them are untouched; the dot does not compete
  for that slot.
- **Notifying on anything but a run ending.** No progress pings, no "still
  working" reminders, no notification on spawn.
- **Sound.** Whatever the platform does by default.

## Further Notes

Three costs were accepted with open eyes. A session that blocks after three
seconds is silent, because the single *n* gate applies to every ending — the
simplicity of one rule was preferred to catching the fast block. Every
notification arrives *D* late, which is the price of collapsing flicker. And a
collapsed space in the sidebar shows nothing, because the dot deliberately stays
off the space row rather than forcing the precedence question `attention.ts` has
been holding open.

The clock reads published states rather than raw evidence on purpose. The
publisher already smooths a positive signal differently from a bare absence, with
a startup grace, and duplicating any of that downstream would produce two
disagreeing notions of when a session is working.
