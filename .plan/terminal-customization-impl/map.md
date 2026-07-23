# Terminal customization — implementation

## Destination

Every terminal island is fully customizable from a single per-machine
`terminal.toml`: font, a layered theme (app defaults → named preset → per-slot
overrides), cursor, scrolling, scrollbar, padding, and keybindings including
Shift+Enter for a newline. New capabilities ship on top — a GPU renderer with a
safe fallback, optional ligatures, clickable links, an in-terminal find, correct
wide-glyph widths, and a contrast floor. The file is the single source of truth;
editing it re-applies to every open terminal, a bad value falls back to a default
with a warning, and the Settings surface stays read-value-plus-open-file. Done
when the [spec](../terminal-customization/spec.md) is implemented end to end.

## Notes

**This map carries execution.** Every ticket is a `task` that delivers working
code, not a decision — all decisions were made through the grilling rounds
synthesized into the [spec](../terminal-customization/spec.md), which is the
single source of truth here. Do not re-litigate a decision; if implementation
exposes one as wrong, raise it against the spec rather than quietly deviating.
(There is no separate planning `map.md` for this effort — the deciding happened in
conversation and lives entirely in the spec.)

**Per-session reading order:** the spec first, then this map, then the ticket you
claim. Use `CONTEXT.md` at the repo root for vocabulary — "island", "chrome",
"control socket", "terminal socket", "user config", "Settings surface". Respect
the ADRs in `docs/adr/` for the area you touch, especially 0010 (Svelte chrome /
imperative islands — never reach inside a renderer; re-theme at the seam), 0012
(shadcn-svelte design system — tokens + primitives, no raw colour in the chrome,
no amber), and 0013 (webview shell) for the links work.

**The two test seams** (per the spec's Testing Decisions): Seam 1 is the pure Go
parse — `terminal.toml` contents → `TerminalPrefs` + warnings, folded together
with the settings landing on the snapshot (tested as one). Seam 2 is the pure
client resolve builder beside the token bridge in `web/src/lib/tokens.ts` —
`TerminalPrefs` → xterm options + theme object, plus the Shift+Enter
`event → action` predicate. Tests observe those seams only; the imperative island
(mount, addons, WebGL fallback, CSS, find widget) is trusted once the resolve seam
hands it the right object, matching how the islands are treated today.

**Before committing frontend changes** (per CLAUDE.md): run the frontend `check`
and `build` scripts plus `vitest`, and `go vet ./...` / `go test ./...` (the embed
test compiles against `dist/`). No amber in the built CSS. All addons and fonts
are bundled — no CDN, no runtime fetch. Review the diff and drive the real
behaviour where "Done when" is only real at runtime, then resolve by shipping:
append `## Answer` with what shipped plus a gist + link under Decisions so far.

## Decisions so far

<!-- one line per resolved ticket: gist + link. Empty until the first ticket ships. -->

- **01 — Prefs pipeline spine.** `terminal.toml` → pure Go `ResolveTerminalPrefs`
  (Seam 1) folds prefs+warnings onto the snapshot; `buildTerminalOptions` in
  `tokens.ts` (Seam 2) resolves them into xterm options/theme; `Terminal.svelte`
  consumes them and remounts on change. [ticket](tickets/01-prefs-pipeline-spine.md)

## Not yet specified

<!-- Empty. Every decision is settled in the spec; this map only executes it. A ticket that exposes a genuinely new question sends it back to the spec — it does not open fog here. -->

## Out of scope

- **A write-back settings UI** — editing happens in the operator's editor; the
  Settings surface stays read-value-plus-open-file. In-panel controls that write
  `terminal.toml` would reintroduce a second config store.
- **Per-space or committed terminal settings** — this is per-machine user config
  only.
- **Theme sync, sharing, or a preset marketplace** — beyond the bundled named
  presets.
- **Retheming the renderer internals** — all customization is fed in at the seam.
- **macOS shell build/packaging work** — beyond exposing the external-open hook
  the links ticket depends on.
- **Sixel/image output and any addon not named in the spec.**
