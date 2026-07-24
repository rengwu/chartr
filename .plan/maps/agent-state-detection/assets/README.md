# Recordings

Real PTY captures taken 2026-07-23 while designing this map. Both tickets treat
them as test fixtures — the rule engine is tested against recorded agent bytes
rather than hand-written strings, because hand-written strings encode what we
*think* an agent draws.

- `rec-claude.jsonl` — Claude Code: idle, a turn, and a `Bash` permission prompt
  left on screen. 89s.
- `rec-kimi-0.29.0.jsonl` — Kimi Code 0.29.0: idle, a long turn with the
  `⠋ thinking...` spinner, and the `▶ Run this command?` approval panel. 319s.
- `osc-claude.log`, `osc-kimi.log` — the OSC sequences each emitted, decoded with
  codepoints. Claude's `✳`/braille title glyphs are visible here; Kimi's two
  title writes for a whole session are the evidence that it signals nothing.

## Format

Line 1 is `{"cols":N,"rows":M}`. Every line after is `[elapsed_seconds,
"<base64 chunk>"]` — the raw PTY bytes as they arrived, in order. Feeding the
chunks in sequence into a terminal emulator reconstructs the screen at any
moment; stopping at a timestamp reconstructs it as of then.

Captured at 137x65. Both agents lay out against the reported width, so replaying
at a different size will not reproduce the recorded screens — size the emulator
from the header.
