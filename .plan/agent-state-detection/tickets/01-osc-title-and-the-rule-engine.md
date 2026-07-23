---
type: task
claimed_by: s308815498edf
claimed_at: 2026-07-23T19:11:03Z
---

# The OSC title and the rule engine

## Question

Stand up agent identification, the manifest rule engine, and the one evidence
source that needs no terminal emulator — the OSC title the agent broadcasts
about itself. Ticket 02 adds the screen grid behind the same engine; nothing
here should need rewriting when it does.

**Identify the agent.** A tab's foreground process resolves to a known agent or
to nothing. Follow herdr's shape rather than today's `procName`: inspect the
whole foreground process *group*, not just its leader, and score candidates so a
generic runtime never wins — `node`, `bun`, `python`, `sh`, `zsh`, `tmux` rank
below a real match, which is what makes a `node`-launched `claude` resolve to
claude. Aliases come from the manifest (`claude-code` → claude).

**Sniff OSC in the read loop.** Parse OSC `0`/`2` (title) and OSC `9` (progress)
out of the PTY byte stream in `Terminal.pump`, retaining the latest value of
each. It must be a byte-at-a-time state machine spanning chunk boundaries — a
title genuinely splits across two `Read`s — handling both terminators (BEL and
ST). Clear retained OSC state when the foreground agent changes, so one agent
never inherits another's evidence. Kimi emits ~1000 OSC 8 hyperlink sequences per
turn, so non-title OSCs must be discarded cheaply rather than buffered.

**The rule engine.** Per-agent TOML manifests, `go:embed`ed, each a list of
rules: `{ id, state, priority, region, matchers, flags }`. Highest-priority match
wins; `skip_state_update` lets a rule veto a sample entirely (a transcript viewer
or model picker showing stale prompt text must not be read as blocked). Matchers
needed now: `contains` (all), `any`, `all`, `not`, `regex`, `line_regex`. Regions
in this ticket: `osc_title` and `osc_progress` only — both read the retained
values, not the screen. Region lookup must be a single seam so ticket 02 adds
screen regions without touching rule evaluation.

**Ship manifests for claude, codex and grok** — the three agents whose title
carries state. Measured on this machine, Claude Code sets OSC 0 to
`<glyph> <task summary>`: `✳` (U+2733) means not-working, a braille frame
(U+2800–U+28FF) means working, updating about once a second. Codex and grok
additionally put a blocked signal in the title (`Action Required`) and grok emits
OSC 9;4 progress; take their rules from herdr's manifests, which are current.
Kimi, opencode and pi emit no state in the title at all and get no manifest here
— they are ticket 02's.

**Publish with hysteresis, or it strobes.** Copy herdr's asymmetry, which is what
makes the indicator calm: a rule that *positively* matched an idle signal
publishes immediately, but a mere absence of `working` must be confirmed over
roughly three samples (~100ms apart, capped ~700ms) before it publishes. Add a
startup grace (~3s) — Claude emits no title for its first several seconds, and a
tab must not flicker idle while an agent boots. Sample at roughly 300ms when an
agent is identified. Publish only on change, as `sampleShell` already does.

**Collapse the two sampling paths.** `sample` currently forks into `sampleShell`
and `sampleSession` with different grammars. Make it uniform: a known agent in
the foreground reads the agent grammar regardless of whether the tab carries a
`Session`; anything else keeps today's foreground-group shell grammar
(`idle`/`working`/`exited`). The reported bug was on ad-hoc shells, so this is
load-bearing, not tidying.

**Retire `quiet`.** Delete the silence threshold, `Terminal.silent`,
`Info.Silent`, `Options.QuietAfter`, and the `config.RoleIsAFK` fold in
`internal/server/spaces.go`. It measured PTY silence, which any cursor blink
resets, so it never fired for the agents it was written for. `model.TerminalQuiet`
gives way to `model.TerminalBlocked`; update `web/src/lib/model.ts`, the indicator
in `App.svelte`, and ticket 13's star-map session grammar
(`web/src/lib/starmap/session.ts`) which reads these states. `dead` and `exited`
are untouched.

**Capture the fixtures.** The engine is a pure function from
`(agent, osc_title, osc_progress, screen)` to a state — table-test it against
real recorded byte streams rather than hand-written strings, and keep the
recordings in the repo for ticket 02 to extend.

Tests lead: a table test over the pure engine (each shipped manifest's rules, and
the veto cases), a hysteresis test asserting a positive idle publishes at once
while a bare working→idle is held and then confirmed, and a process-boundary test
that a tab running a stub agent which paints a title reads `working` then `idle`
in the snapshot.

Done when: an ad-hoc shell running `claude` reads `idle` at the prompt and
`working` during a turn — the reported bug, gone; codex and grok additionally
read `blocked` from their titles; a shell running a non-agent command still reads
the old shell grammar; no tab flickers on a normal turn; `quiet` is gone from the
Go model, the TypeScript model, the indicator and the star-map grammar; `go vet
./...` and `go test ./...` pass; frontend `check`, `build` and `vitest` pass with
no amber in the built CSS.

## Answer

Built. A tab's indicator now reports what the agent says about itself, and the
reported bug — an ad-hoc shell running `claude` pinned to `working` forever — is
gone, asserted at the process boundary.

**What shipped.** Three pieces, in three commits.

`internal/terminal/detect` is the rule engine: per-agent TOML manifests,
`go:embed`ed, each a list of `{id, state, priority, region, matchers, flags}`.
Highest priority wins and `skip_state_update` vetoes a sample outright. All six
matchers are implemented (`contains`, `any`, `all`, `not`, `regex`,
`line_regex`), ANDed so a rule narrows as fields are added; a rule with no
matcher at all never matches, so a stray empty rule cannot swallow every sample.
`Evaluate` is pure. `region()` is the single seam ticket 02 extends — it already
carries a `screen` case wired to an `Evidence.Screen` field, so ticket 02 adds
screen regions without touching rule evaluation. `Identify` scores every argv
token across the whole foreground group, skipping generic runtimes, which is what
resolves a `node`-launched `claude` to claude.

`internal/terminal/osc.go` is the sniffer: a byte-at-a-time state machine in
`Terminal.pump` carrying state between reads, retaining the latest OSC 0/2 and
OSC 9, honouring BEL and ST. The numeric code is parsed as it arrives, so a
non-title OSC is skipped without ever being buffered — Kimi's ~1000 OSC 8
hyperlinks a turn cost one state transition each. Retained values are cleared
when the identified agent changes.

The wiring collapses `sample()`: no more `sampleShell`/`sampleSession` fork. A
tab resolves to a known agent or to nothing, and a known agent reads the agent
grammar whether or not the tab carries a `Session`. Publishing is asymmetric
(`publisher` in `publish.go`) — a positive match lands at once, a bare absence is
confirmed over three samples, and a ~3s startup grace refuses an absence-derived
idle while an agent boots. Agent-bearing tabs sample at ~300ms, everything else
at the old ~900ms.

**Each Done-when clause.** An ad-hoc shell running a stub `claude` that paints a
title reads `working` then `idle`, asserted through a real PTY in
`TestAdHocShellRunningAnAgentReadsItsTitle`, and confirmed against the real
89-second Claude capture in `TestClaudeRecordingReadsWorkingThenIdle`. Codex and
grok read `blocked` from `Action Required` in the engine table test. A non-agent
command keeps the old grammar (`TestNonAgentCommandKeepsTheShellGrammar`, plus
the pre-existing `TestSampleTracksForegroundCommand`, unchanged and passing). No
flicker: replaying the real recording publishes **seven** transitions across 89
seconds, never the same state twice running, asserted with a bound in
`TestClaudeRecordingDoesNotFlicker`. `quiet` is gone from the Go model, the
TypeScript model, the indicator and the star-map grammar. `go vet ./...`,
`go test ./...`, and frontend `check`/`build`/`vitest` all pass, with no amber in
the built CSS.

**Two things a human should look at.**

*Claude reads `idle` during a long tool call.* The map defines `✳` (U+2733) as
"not-working", so that is what I implemented — but the real capture shows Claude
sitting on `✳` for 26 seconds mid-turn while a tool ran, and the tab therefore
reads idle for that stretch. The title glyph appears to mean "not generating
tokens right now", which is not the same as "not working". This is a finding
against a settled decision, not a deviation from it: ticket 02's screen evidence
(which can see the `esc to interrupt` footer) is the natural place to fix it, and
that is a human's call to make.

*The codex and grok manifests are partly extrapolated.* Only claude and kimi were
captured as fixtures, so only claude's rules are grounded in recorded bytes. The
blocked signal for both (`Action Required`) and grok's OSC 9;4 progress come from
the ticket's own description; codex's working title (`Working`/`Thinking`/
`Running`) is my inference from its documented status line. herdr's manifests
turned out not to be extractable — they are compiled into its Rust binary, not
shipped as loose TOML, and the repo's tree API response truncates before them. I
flagged this inline in both manifest files. A wrong string there is a data fix,
which is the point of the manifests being data.

**Deliberately not done.** No veto rule was added to a shipped manifest: the veto
mechanism is implemented and tested (against an inline synthetic manifest, so the
test invents no agent behaviour), but a title-only veto would have been fabricated
— vetoes are for screens, and belong to ticket 02. Kimi, opencode and pi get no
manifest, per the ticket;
`TestKimiRecordingCarriesNoTitleState` pins that claim against the real capture,
so it fails loudly if Kimi ever starts broadcasting state. In the star-map,
`blocked` inherits the crawl-and-blink channels `quiet` had, and `spaceLiveness`
keeps its existing precedence, because how `blocked` folds into the attention
grammar is explicitly left open on the map (*Not yet specified — Notifications*).

**One thing I changed my mind about, with a measurement behind it.** My first
pass identified a session's agent by inspecting its foreground process group, the
same way an ad-hoc shell's is. That put a `TIOCGPGRP` ioctl and a `ps` onto
session PTYs, which had never carried either, and it made `internal/server` flaky
— the halt tests failed roughly half of all runs, against a baseline that passed
2/2. It was not a push storm (I counted: ~30 model pushes across the suite) and
not a slow `ps` (2 invocations across the suite). A session does not need
inspecting at all: chartr launched the agent and recorded which adapter it ran,
so the binding *is* the answer. Resolving a session from `Session.Agent` is
cheaper, steadier, and keeps both syscalls off session PTYs as before. Measured
after the change: **branch 0/8 failures, baseline 0/8** — parity restored. Two
smaller correctness bugs fell out of the same hunt and are fixed: an unreadable
foreground (`pgrp <= 0`) no longer drops the identification and restarts the
grace on every tick, and a live session with no manifest keeps reading `working`
rather than the shell grammar's permanently-wrong `idle`.

**Fixtures.** The engine is tested against the recordings already in
`.plan/agent-state-detection/assets/`, read from there rather than copied into
`testdata/`, since the assets README designates them as the fixtures both tickets
use and a copy would drift. `recording_test.go` carries the loader; ticket 02
extends it.
