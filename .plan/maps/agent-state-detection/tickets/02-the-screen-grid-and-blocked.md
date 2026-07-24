---
type: task
blocked_by: [01]
claimed_by: s63f4ee253a9d
claimed_at: 2026-07-24T04:10:38Z
---

# The screen grid, kimi, and blocked

## Question

Reconstruct each terminal's screen server-side and let ticket 01's rule engine
read it. This is where the states that matter most actually arrive: Claude
signals working-vs-idle in its title but **never** signals blocked there — a
permission prompt reads as `✳`, byte-identical to idle — and kimi, opencode and
pi put nothing in the title at all. Everything past "is Claude thinking" needs
the screen.

**Add the grid.** Feed PTY bytes through `github.com/charmbracelet/x/vt` in
`Terminal.pump`, alongside the existing scrollback append, and expose a
`detectionText()` returning the rendered viewport as plain text
(`Emulator.String()`). This is read for *detection only* — the browser keeps
rendering through xterm.js from the raw scrollback exactly as today (ADR 0010).

Two things were measured and must not be re-litigated:

- **Cost is not a concern.** 0.70 MiB of real kimi output — a whole five-minute
  session — replays through the emulator in ~58ms including decode. Do not add
  sampling tricks to avoid emulator cost; do keep herdr's skip-rescan-when-idle
  optimisation only if it falls out naturally.
- **The emulator deadlocks unless its reply channel is drained.** Agents query
  the terminal (Claude opens with `OSC 11;?` asking the background colour); the
  emulator answers by writing into an internal pipe, and with no reader that
  write blocks forever and wedges the whole emulator mid-stream. Drain
  `Emulator.Read` from a goroutine. This is not hypothetical — it is how the
  spike first failed.

Resize the grid with the PTY so regions anchored to the bottom stay meaningful.

**Add the screen regions** behind ticket 01's region seam: `whole_recent`,
`bottom_non_empty_lines(n)`, `after_last_horizontal_rule`, `prompt_box_body`.
They are the point of the design — rules slice the screen structurally instead of
grepping it, which is what keeps a keyword in transcript prose from being read as
live chrome.

**Ship manifests for kimi, opencode and pi**, and add the screen rules that give
claude its `blocked`. herdr's shipped manifests were verified to fire unmodified
against real recordings on this machine:

- kimi **working** — `⠋ thinking...` matches an anchored
  `^\s*[⠀-⣿]+\s*(thinking\.\.\.|working\.\.\.|using )`. The anchor is
  essential: kimi's status bar reads `K2.7 Coding thinking ~` on *every* screen,
  so an unanchored `contains "thinking"` is always true. This is the trap the
  region-and-anchor design exists to avoid.
- kimi **blocked** — `↵ confirm` + `▶ Run this command?` + ` choose` +
  `Approve`/`Reject`.
- claude **blocked** — `Do you want to proceed?` + `❯ 1. Yes` +
  `Tab to amend`/`ctrl+e to explain`, scoped to `after_last_horizontal_rule`.
- claude **idle** — the `❯` prompt box, which must rank *below* the blocked
  rules so a title saying idle never overrules a screen showing a permission
  prompt.

**Anchor rules on structure, never on prose.** Reconstruction is not perfect and
the imperfection is specific: `charmbracelet/x/vt` garbles a prose line inside
Claude's permission dialog (spaces clobbered) while rendering every structural
line — the horizontal rule, the numbered options, the footer hints — cleanly. A
second emulator (`danielgatis/go-headless-term`) got that prose line right but
merged `Do you want to proceed?` into the horizontal rule above it and left stale
duplicate status lines, which is why it was rejected. Rules that match on menu
options, footer hints, box borders and rules are reliable; rules that match on
sentences are not.

**Extend the fixtures.** Ticket 01's captured recordings gain kimi's approval
panel and claude's permission dialog; the pure-engine table test grows cases that
assert each new rule fires at the recorded moment it should and stays silent
before and after.

Tests lead: extend the pure-engine table test with the screen regions and the new
manifests, driven by the recordings; add a regression asserting kimi's
always-present `thinking` status bar does **not** read as working; add a
process-boundary test that a stub agent painting a permission-prompt screen reads
`blocked` in the snapshot and returns to `idle` once answered.

Done when: kimi reads `idle` / `working` / `blocked` correctly through a real
turn; claude reads `blocked` while sitting on a permission prompt and `idle` once
answered; opencode and pi read working-vs-idle; the emulator survives a full
session without wedging (the OSC-query drain is tested, not assumed); a tab with
no known agent still reads the shell grammar; `go vet ./...` and `go test ./...`
pass; frontend `check`, `build` and `vitest` pass with no amber in the built CSS.

## Answer

Built. The states that never reached the title — claude's `blocked`, and kimi's
whole grammar — now arrive off a server-side screen reconstruction, and the rules
that read them slice the screen structurally rather than grepping it.

**What shipped.** Three commits.

`internal/terminal/grid.go` is the screen: a `github.com/charmbracelet/x/vt`
`Emulator` fed the same PTY bytes the browser renders, in `Terminal.pump` alongside
the scrollback append and the OSC scan. `detectionText()` hands its rendered
viewport to the sampler. It is read for detection only — the browser still renders
through xterm.js from the raw scrollback (ADR 0010) — and the grid follows the PTY
on resize so a bottom-anchored region stays meaningful. The emulator answers
terminal queries (claude opens with `OSC 11;?` asking the background colour) by
writing the reply into an internal pipe; unread, that write blocks forever and
wedges the emulator mid-stream, so a goroutine drains it. I stop the drain by
closing the reply pipe rather than calling `Emulator.Close`, whose internal
`closed` flag races the drain's `Read` under `-race`; closing the pipe (an
`io.Pipe`, safe for concurrent Read/Close) is race-free. The grid carries its own
lock and its off-screen scrollback is capped small, since detection reads only the
viewport.

The region seam (`detect.region`) gained four screen regions behind the same
one-case shape ticket 01 left for it: `whole_recent`, `bottom_non_empty_lines(n)`,
`after_last_horizontal_rule`, `prompt_box_body`. A region name may now carry an
integer argument. They all turn on one measured distinction — a flat rule (a run of
U+2500) is not a cornered box border (`╭ ╮ ╰ ╯` with `│` sides) — which is what lets
`prompt_box_body` frame claude's flat-ruled input box and kimi's flat-ruled approval
panel while ignoring kimi's rounded idle box. Rule *evaluation* did not change.

Manifests: kimi, opencode, pi shipped; claude gained its screen rules. Every rule is
anchored on structure, never prose (the ticket's warning is real — vt garbles the
long "Yes, and always allow…" option in claude's dialog but renders the rule, the
numbered options and the footer hints cleanly, all verified against the recording).

**Each Done-when clause.**

- *kimi reads idle/working/blocked through a real turn.* `working` off its anchored
  `⠋ thinking...` spinner, `blocked` off the `▶ Run this command?` panel framed
  between two flat rules, idle by absence. Asserted end-to-end against the real
  319-second capture in `TestKimiRecordingReadsWorkingAndBlockedFromScreen` (screen
  reconstructed at the recording's own geometry), and per-rule in the engine table.
- *claude reads blocked on the permission prompt and idle once answered.* `blocked`
  is `after_last_horizontal_rule` matching "Do you want to proceed?" + "❯ 1. Yes" +
  the footer, at priority 200 so a ✳ idle *title* can never overrule a screen plainly
  on a permission prompt; `idle-screen` reads the `❯` prompt box below it.
  `TestClaudeRecordingReadsBlockedFromScreen` drives it off the real capture; the
  live `TestAdHocShellAgentReadsBlockedFromScreen` paints a permission screen in a
  real PTY while holding the title on ✳ — a pass proves the reading came from the
  grid, not the title — and reads `idle` again once answered.
- *opencode and pi read working-vs-idle.* A braille spinner near the foot of the
  screen reads `working`, its absence idle. See the caveat below.
- *the emulator survives without wedging; the drain is tested.* `TestGridDrainsTerminalQueryReplies`
  feeds an `OSC 11;?` under a deadlock guard; and the claude capture carries a real
  `OSC 11;?`, so every screen replay exercises the drain unwedged.
- *a tab with no known agent still reads the shell grammar.*
  `TestNonAgentCommandKeepsTheShellGrammar` and `TestSampleTracksForegroundCommand`
  are unchanged and pass.
- *vet/test/frontend.* `go vet ./...` clean; `go build`/`go test` green. The
  `internal/terminal` and `internal/terminal/detect` packages pass 5/5 repeated and
  under `-race`. Frontend `check` (0 errors), `build`, and `vitest` (127/127) pass,
  no `amber` in the built CSS. I made no frontend changes — ticket 01 already retired
  `quiet` and folded in `blocked`.

**Two things a human should look at.**

*opencode and pi are unverified extrapolations.* Only claude and kimi were captured
(the ticket says so), and both put nothing in the title, so their working-vs-idle
must come from the screen — but I had no recorded screen to ground it. Their working
rule (a braille spinner in the recent lines) is the shared pattern claude and kimi
use, extrapolated and flagged inline in both manifests, exactly as ticket 01 handled
codex/grok. The engine table asserts the rule fires on a plausible spinner and stays
silent without one, but "correct through a real turn" cannot be *verified* for these
two until someone captures a fixture. This is the #1 place a wrong string would hide;
being a data fix is the point of the manifests being data.

*The grid's initial geometry is a default, not the PTY's.* go-pty exposes `Resize`
but no getter, so a new grid starts at 80×24 and only learns the real width when a
browser attaches and resizes it (the PTY and grid resize together from then on). For
a title-based agent this is irrelevant; for a screen-based one (kimi/opencode/pi)
running headless with no browser ever attached, reconstruction could be off until an
attach. In practice the cockpit attaches; I flag it rather than hide it.

**Deliberately not done.** I did not fix claude reading `idle` during a long tool
call (ticket 01's flagged finding — the ✳ title means "not generating now"). Ticket
02's Done-when does not ask for it, and a screen-based *working* rule for claude
(off the `esc to interrupt` footer) is a new decision for a human on the planning
map, not this ticket's scope. I added no veto rule to a shipped manifest (still no
grounded screen that warrants one). The pre-existing `internal/server` halt-test
flakiness is unchanged by this work: measured on this machine, `main` fails ~2/5 and
this branch ~1/6, a different halt lifecycle test each time — load-sensitive timing,
not a regression (the terminal package it exercises is deterministic here).
