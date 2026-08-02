# Recordings

Real PTY captures of every agent in the roster. Both the design tickets and the
implementation tickets treat them as test fixtures — the rule engine is tested
against recorded agent bytes rather than hand-written strings, because
hand-written strings encode what we *think* an agent draws. Ticket 04 is the
proof: four manifests written from herdr's data and from the braille shape the
first two agents share, and measurement contradicted three of them.

Taken 2026-07-23 while designing this map:

- `rec-claude.jsonl` — Claude Code: idle, a turn, and a `Bash` permission prompt
  left on screen. 89s.
- `rec-kimi-0.29.0.jsonl` — Kimi Code 0.29.0: idle, a long turn with the
  `⠋ thinking...` spinner, and the `▶ Run this command?` approval panel. 319s.
- `osc-claude.log`, `osc-kimi.log` — the OSC sequences each emitted, decoded with
  codepoints. Claude's `✳`/braille title glyphs are visible here; Kimi's two
  title writes for a whole session are the evidence that it signals nothing.

Taken 2026-08-03 for ticket 04, each driving one agent through idle → working →
stopped-for-permission → approved → idle:

- `rec-codex-0.146.0.jsonl` — Codex CLI, run `-a untrusted -s read-only` so it
  stops on an approval rather than auto-running. Title is `agentcwd` idle,
  `⠹ agentcwd` working, `[ . ] Action Required | agentcwd` blocked. 65s.
- `rec-opencode-1.2.27.jsonl` — opencode, with `permission.bash = "ask"` in a
  project `opencode.json` so its `△ Permission required` panel appears. The
  spinner is square cells, not braille. 51s.
- `rec-grok-0.2.118.jsonl` — Grok Build Beta, stopped on its numbered approval
  panel and approved with `2`. Title carries `⚠ Action Required`; **no OSC 9;4
  progress is emitted at any point**. 48s.
- `rec-pi-0.78.0.jsonl` — pi: a turn driving four bash commands, with a follow-up
  line typed into the input box mid-turn (that is deliberate — it pushes the
  `⠇ Working...` spinner one line further from the foot and is what sized the
  rule's region). No permission state: pi ships no approval gate. 45s.

Taken 2026-08-03, after an operator reported an opencode session on a
multiple-choice question reading as idle:

- `rec-opencode-1.2.27-question.jsonl` — opencode stopped on its `question` tool
  choice panel, answered with ↓/enter, then run to completion. Working from 12s,
  the panel 15s–89s, working again, idle. It exists because a *permission* is not
  the only way opencode blocks: the ticket-04 capture drove one, the panel shares
  none of its chrome, and nothing was watching the gap. 133s.

The lesson `-question` adds to the one above: measuring an agent is not the same
as measuring an agent's *states*. Three manifests were wrong because their
patterns were extrapolated; this one was wrong because a state was never driven.
A capture only proves the transitions it walks through.

## Format

Line 1 is `{"cols":N,"rows":M}`. Every line after is `[elapsed_seconds,
"<base64 chunk>"]` — the raw PTY bytes as they arrived, in order. Feeding the
chunks in sequence into a terminal emulator reconstructs the screen at any
moment; stopping at a timestamp reconstructs it as of then.

All captured at 137x65. Agents lay out against the reported width, so replaying
at a different size will not reproduce the recorded screens — size the emulator
from the header.

The recorder that writes this format is deliberately throwaway and does not ship;
it is rebuilt when a capture is needed. It spawns the agent on a PTY of the
header's size and replays a script of `wait` / `type` / `key` steps into it, so a
turn can be driven unattended. Roughly 150 lines around `creack/pty`.
