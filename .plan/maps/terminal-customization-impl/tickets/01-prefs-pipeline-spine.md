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

## Answer

The vertical spine is standing. An operator sets `font.family`, `font.size`, and
`theme.background`/`theme.foreground` in a per-machine `terminal.toml` (beside
`user.toml` under the state root), and every open terminal island picks it up.

- **Seam 1 (server, `internal/config/terminal.go`):** `ResolveTerminalPrefs([]byte)
  → (TerminalPrefs, []string)` — pure. Missing file → all defaults, no warnings; a
  malformed file → all defaults + one warning; a non-positive size / non-`#hex`
  colour / unknown key each keeps its default and warns. `buildModelFor` resolves
  it once and folds the prefs onto `Model.Terminal` (a wire-local
  `model.TerminalPrefs`), routing the warnings onto each space beside the
  agent-library warnings. Table-tested in `terminal_test.go`, and the
  parse-folded-with-snapshot in `internal/server/terminalprefs_test.go`.
- **Seam 2 (client, `web/src/lib/tokens.ts`):** `buildTerminalOptions(prefs)`
  turns `TerminalPrefs` into the xterm options + `ITheme`, resolving every unset
  colour off the live design tokens exactly as the old `buildTheme` did, stacking a
  pref font ahead of the bundled default, and carrying the six ANSI hues as the
  default preset layer. Table-tested in `tokens.test.ts`.
- **Island (`Terminal.svelte`):** consumes the resolved options via a `prefs`
  prop instead of its hard-coded literals; `SpacePane` remounts it through a
  `{#key}` that includes a prefs identity, so editing the file re-applies (the
  terminal socket replays scrollback on re-attach — nothing lost).

Both seam tests green; frontend `check`/`build`/`vitest` (94) and
`go vet`/`go test ./...` pass; no amber in the built CSS.
