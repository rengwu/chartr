---
type: task
blocked_by: []
---

# Prefs pipeline spine

## Question

Stand up the whole vertical path once, minimally, so every later ticket has a
seam to hang off. An operator sets a font family, a font size, and a
background/foreground colour in a new per-machine `terminal.toml` (under the user
config / state root, beside the agent library), and every open terminal island
picks it up.

Cut all three layers:

- **Server (Seam 1):** a pure function reads the file's contents and produces a
  resolved `TerminalPrefs` value plus a list of human-readable warnings — missing
  file yields all defaults and no warnings; an unreadable/invalid value keeps the
  default and adds a warning. Fold `TerminalPrefs` (and its warnings) into the
  pushed model snapshot so every browser receives it the way it receives the rest
  of the model, and route the warnings into the existing config-warnings surface.
- **Client (Seam 2):** a pure builder beside the token bridge in
  `web/src/lib/tokens.ts` turns `TerminalPrefs` into the concrete xterm options
  object and theme object the island consumes. Unset colour slots resolve against
  the live design tokens exactly as today's `buildTheme` does.
- **Island:** `Terminal.svelte` consumes the resolved objects instead of its
  hard-coded literals, and remounts when the prefs on the snapshot change (the
  terminal socket replays scrollback on re-attach, so nothing is lost).

Tests lead: the Seam 1 table test (file contents → prefs + warnings, incl. missing
file and one bad value) and the Seam 2 table test (prefs → options/theme, unset
colour falls to token default) land red first, mirroring the existing
`tokens.test.ts` and Go config tests.

Done when: a font/size/colour set in `terminal.toml` shows in every open terminal;
a missing file leaves today's look unchanged; both seam tests are green; frontend
`check`/`build`/`vitest` and `go vet`/`go test` pass.
