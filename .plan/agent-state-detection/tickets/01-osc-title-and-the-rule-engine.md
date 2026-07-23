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
