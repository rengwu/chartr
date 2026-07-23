# Agent state detection

## Destination

A tab's activity indicator reports what the *agent* is doing, not what the
operating system can see. Every terminal — ad-hoc shell or session — reads
`idle` / `working` / `blocked` from evidence the agent itself produces: the OSC
title it broadcasts, and the screen it draws, matched by per-agent rules that
ship as data rather than code. An operator glancing at the sidebar can tell a
Claude that is thinking from one waiting on them, and a Kimi sitting on an
approval dialog is visible without opening its tab. Done when the six agents in
the roster (claude, kimi, codex, opencode, grok, pi) each read correctly through
a full turn — idle, working, and stopped-for-permission — and the old
silence-based `quiet` heuristic is gone.

## Notes

**Why this map exists.** Ticket 10 on the [implementation
map](../chartr-design-impl/map.md) surfaced liveness the only way it could at the
time: an ad-hoc shell reads its PTY's foreground process group, and a session is
`working` while alive with a `quiet` hint after a silence threshold. Both are
proxies that fail the same way — a TUI agent holds the foreground for its entire
life and repaints its cursor forever, so "a program is running" and "the agent is
busy" are indistinguishable. This map replaces the proxy with the real signal. It
is a genuinely new decision, not in the [spec](../chartr-design/spec.md), which is
why it does not graduate onto `chartr-design-impl`.

**The approach is measured, not assumed.** The design follows
[herdr](https://github.com/ogulcancelik/herdr) — an agent multiplexer that solved
this — and every load-bearing claim below was verified locally against real PTY
recordings of Claude Code and Kimi 0.29.0 before this map was written. Where a
finding contradicts intuition it is called out on the ticket. The two throwaway
spikes (an OSC sniffer and an emulator replayer) are gone; the *recordings* are
worth keeping as test fixtures and the tickets say so.

**The state grammar changes.** `idle` / `working` / `blocked` replaces
`working` / `quiet` for agent-bearing tabs. `blocked` — an agent stopped on a
permission prompt — is new and is the state worth notifying on. `quiet` is
deleted rather than kept alongside: it measured PTY silence, which a blinking
cursor resets, so it was close to unreachable for exactly the agents it was meant
to catch. `dead` / `exited` are untouched; the death halt (ticket 10) keeps its
three choices.

**One grammar for shells and sessions.** Today `sampleShell` and `sampleSession`
are separate paths in `internal/terminal/terminal.go`. The reported bug was on
*ad-hoc shells* running `claude`, so detection cannot be session-only. The rule
becomes uniform: a known agent in the foreground reads the agent grammar,
anything else reads the shell grammar.

**Manifests are data, embedded.** Per-agent rules live in TOML and are
`go:embed`ed. When an agent's TUI changes, fixing it is a data change, not a code
change. Deliberately *not* copied from herdr: their remote manifest-update
system — chartr is one offline binary (ADR 0011), so manifests ship with it.

**Detection never enacts.** This map only changes what the snapshot reports.
Nothing here kills, resumes, or requeues a session; the absence of autonomous
action that ticket 10 asserted stays asserted.

**Testing.** Process-boundary as everywhere else (spec, Testing Decisions), with
one addition: the rule engine is a pure function from
`(agent, screen, osc_title, osc_progress)` to a state, so it gets a table test
fed by the captured recordings — real bytes, not hand-written fixtures. Frontend
work follows CLAUDE.md and ADR 0012 (tokens + primitives + Phosphor, no raw
colour, no amber).

## Decisions so far

<!-- one line per resolved ticket: gist + link. Empty until the first ticket ships. -->

## Not yet specified

- **Hook-reported state.** `pi` ships a real herdr extension that reports its
  lifecycle over a socket, and a hook is authoritative where it exists. Worth
  doing, but it needs a socket API and an install story, and herdr's own
  experience is a caution: they tried hook-driven state for Claude and reverted it
  (`SubagentStop` fires after a turn already ended and revived idle panes). Left
  as fog until the screen path is real — it is the fallback everything else needs
  anyway.
- **Notifications.** `blocked` is the state an operator wants pushed to them
  rather than glanced at. The star-map already has an attention grammar (ticket
  14) and a moons overlay (ticket 13); how `blocked` folds into those, and whether
  it earns a toast, is not decided here.
- **Which agents earn a manifest beyond the roster.** Six are named in the
  Destination. A seventh is a data change, but the roster is what "done" means.

## Out of scope

- **Re-theming or re-rendering the terminal island.** The grid is a *server-side*
  reconstruction read for detection only; the browser keeps rendering through
  xterm.js exactly as today (ADR 0010 — never reach inside a renderer).
- **Replaying the reconstructed screen to the browser.** The scrollback replay on
  attach is unchanged; the grid is not a second source of truth for display.
- **Acting on a detected state** — no auto-kill, no auto-resume, no timeout.
  Detection reports; the operator acts.
- **Remote or auto-updating manifests** — bundled and embedded only.
- **Agent-specific features beyond state** — the OSC title also carries Claude's
  live task summary, which is tempting; anything past the three states is a
  separate effort.
