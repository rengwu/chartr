---
type: grilling
blocked_by: [01]
---

# Knowing a session finished, hung, or died

## Question

chartr watches `.plan/` and derives resolution from the files (ADR 0004), so it never has to ask an agent whether it is done. That covers the happy path and nothing else. A session can also exit cleanly having achieved nothing, exit non-zero, hang forever waiting on input nobody will give it, loop while burning tokens, or die outright while its ticket still reads `implementing`.

Given what ticket 01 finds each CLI actually exposes, decide how chartr tells **working** from **stuck** from **dead**, and what it does about each. The candidate signals all have holes worth naming: process exit is weak (an agent that exits without writing `## Proposed Answer` has failed *silently*); a wall-clock timeout punishes slow-but-honest work; output-silence heuristics are guesses, and a grilling session is *supposed* to sit idle waiting for a human.

Settle also:

- What chartr owes the human when a session dies mid-ticket. A stale `claimed_by` is litter the markdown adapter already tolerates and can reason about; a half-finished commit in the working tree is not (ticket 06 owns the git side).
- Whether a dead session's ticket returns to the frontier **automatically or only through a human**. Remember this project has **no retry loops** — every rejection halts to a human, on the argument that looping burns tokens — and the same argument applies to respawning the dead.
- Whether HITL and AFK sessions need different answers here. They almost certainly do: idleness means opposite things.
- **The spawn-mode split underneath all of this.** Ticket 01's contract was researched headless (`claude -p`, `codex exec`, …), where `observe` reads tokens from the agent's JSON event stream. A grilling session is the opposite: a human typing into a PTY. Run the agent's interactive TUI and the JSON stream — and with it the token telemetry — may vanish; run stream-JSON and the xterm.js pane shows raw events instead of a terminal. Decide whether `spawn` has two modes, and say out loud what `observe` degrades to in the interactive one — this amends ticket 01's contract openly rather than deciding around it.

## Answer

**`spawn` has one mode, and it is interactive: every session runs the agent's own TUI in a PTY.** The deciding argument is the escape hatch. Because context is assembled fresh and sessions are never resumed across work (ADR 0005), killing a drifting headless session forfeits everything it knew — whereas typing into a live TUI is lossless intervention. For a cockpit whose standing preference is that a human drives, the intervention channel is load-bearing; a two-mode split would foreclose it for exactly the AFK sessions most likely to drift, and would also oblige chartr to build a second rendering path (a JSON-digest chat surface per agent — the trap ADR 0006 stepped around).

This undermines ticket 01's headless premise, marked there rather than papered over. What `observe` degrades to: **`{alive, dead}` read from the PTY.** Token telemetry demotes from the contract floor to optional, out-of-band, per-adapter observation (claude's session JSONL, opencode `stats`, pi's session files, codex rollout files); exit codes carry no meaning beyond death (a finished or failed interactive agent just returns to its prompt); chartr-enforced budget caps leave the design, and cost visibility (ticket 14) becomes best-effort and after-the-fact. That is accepted deliberately: cost control is the human watching the cockpit, and sessions are allowed to hit provider limits. One loose end for ticket 01's asset: opening-prompt injection into each TUI (vs. headless) needs a citation pass.

**Working, stuck, dead — chartr surfaces, and never acts.** *Dead* is crisp: the PTY closed. *Finished* is already mode-independent and ticket-derived (ADR 0004). *Stuck* is only ever a hint, and it splits by ticket type because idleness means opposite things: an AFK session silent past a threshold with no `## Proposed Answer` gets a "quiet" badge in the cockpit; a HITL session sitting idle is simply waiting for its human. No auto-kill, no auto-nudge, no enforced timeout — a heuristic is not a "must always be true," so it gets rendered, not enacted. The honest cost, accepted with reservations: headless observation would catch silent failure (clean exit, no proposed answer) and loops mechanically, and TUI spinner redraws make raw output a noisy idleness signal. Interactive trades machine legibility for human legibility — the terminal pane itself is this design's best stuck-detector. **Revisit trigger:** if machine detection of stuck ever becomes critical (autopilot is the likely forcing case), that is the moment to reconsider a headless mode.

**A death halts to the human; the ticket returns to the frontier only through them.** The dead session stays pinned to its ticket, scrollback preserved (ADR 0006 already buffers PTY output server-side), and the human chooses: **resume** it, **respawn fresh**, or **abandon** — releasing the claim back to the frontier. Until then the stale `claimed_by` stands, which the markdown adapter tolerates by design. Auto-requeue is a retry loop with one extra step: the no-retry-loops rule argues every rejection halts to a human, and a death is information the next spawn should not walk in blind to. The half-finished working tree a death leaves behind is ticket 06's side of the line.

**Resume is narrowed, not reinstated: same-ticket crash recovery only.** What ADR 0005 rejects is memory accumulated *across* units of work; resuming an involuntarily interrupted session (provider limit, process death, daemon restart) to finish its own ticket accumulates nothing across anything. Never across tickets, never as an alternative to a fresh spawn — ADR 0005 carries the amendment. This also clears the deferred-tmux fog (ADR 0006): crash survival was tmux's main selling point, and a persistent daemon plus agent-side resume after daemon death covers it — tmux stays out.
