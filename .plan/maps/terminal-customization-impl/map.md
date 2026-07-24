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
- **02 — Layered theme + named presets.** Prefs widen to a validated `preset` name
  plus the full slot set (5 base + 16 ANSI); Seam 2 stacks tokens → preset →
  explicit slots, with five palettes bundled as inline data (Dracula, Nord,
  Gruvbox, Solarized dark/light). Unknown preset / bad colour warn and fall back;
  the zero-config look is unchanged. [ticket](tickets/02-layered-theme-presets.md)
- **03 — Font, cursor, scrolling & glyph options.** Prefs widen to the remaining
  pass-through options — font weight/bold-weight/line-height/letter-spacing, the
  `[cursor]`/`[scrolling]`/`[accessibility]` blocks, and a `[glyph]` unicode11
  toggle — validated by reusable enum/sign/range guards (weight takes a keyword or a
  number; `blink`/`unicode11` are tri-state). Seam 2 maps each 1:1 onto xterm via a
  typed `setOpt` (unset never clobbers a default), and a bundled-vs-custom font
  family resolves through `BUNDLED_FONTS`. The unicode11 addon lazy-loads into its
  own bundled chunk at mount when enabled.
  [ticket](tickets/03-font-cursor-scrolling-options.md)
- **04 — Scrollbar, padding & keybindings.** Three new blocks (`[scrollbar]`,
  `[padding]` per side, `[keys]`) resolve through the existing guards; Seam 2 grew a
  `css` return carrying them as CSS custom properties (xterm has no option for
  either), plus `terminalKeyAction`, the pure keydown predicate behind Shift+Enter →
  newline (unset means *on*). The island split in two — `.terminal-island` holds the
  padding, an unpadded `.terminal-grid` is what xterm mounts into — because the fit
  addon measures its parent's *border*-box width, so a padded host would overflow
  instead of refitting. [ticket](tickets/04-scrollbar-padding-keybindings.md)
- **05 — Renderer + ligatures.** `font.ligatures` (tri-state) drives one pure
  Seam 2 decision, `resolveRenderer`: WebGL by default, canvas + ligatures when the
  pref is on and the font is bundled, suppressed to WebGL for a non-bundled family.
  The island lazily imports three bundled addons off that choice — `addon-webgl`
  (with `onContextLoss` → dispose → DOM fallback) or `addon-canvas` +
  `addon-ligatures`, the latter needing `allowProposedApi` for its character joiner.
  No network font fetch. Driven live incl. the context-loss fallback.
  [ticket](tickets/05-renderer-ligatures.md)
- **06 — Clickable links + shell hook.** The bundled `addon-web-links` (own lazy
  chunk, no pref) routes every click through `openExternal` in the new
  `web/src/lib/external.ts` — the native-shell seam beside `titlebar.ts`, not the
  prefs resolve: shell hook when `window.__chartrOpenExternal` is there, a
  `noopener` new tab when it is not. The shell binds it in `main_webview.go` to
  `openExternalURL` (`cmd/webview/external.go`, untagged so its guard is tested by
  the cgo-free build), which hands the URL to `open`/`xdg-open`/`explorer`. Both
  sides hold an http(s)-only allowlist, because a clicked URL is text an agent
  printed and `open` would launch an app for `file:` or a custom scheme. Not driven
  live — no Chrome extension this session. [ticket](tickets/06-links-shell-hook.md)
- **07 — In-terminal find (Cmd+F).** Frontend-only (find is transient UI, not
  config). A bundled, lazily imported search addon behind `TerminalFind.svelte` — a
  token + primitive + Phosphor widget floating at the island's top-right that owns
  only its input and drives the addon through `Terminal.svelte` (ADR 0010).
  Cmd+F opens/focuses (meta only), Enter/Shift+Enter cycle, "Aa" toggles case, Esc
  closes and clears; the live count/index ride `onDidChangeResults`, and the match
  colours resolve from tokens at the seam (`terminalSearchDecorations`). The match
  decorations use `registerDecoration` (xterm proposed API), so `allowProposedApi` is
  now on for every terminal, not gated to ligatures. Driven live.
  [ticket](tickets/07-in-terminal-find.md)
- **08 — Settings "Terminal" section.** `terminal.toml` is now a named config layer
  (`terminal-config`, `holds: "terminal"`), so the Global scope opens it through the
  same space-less named-layer action as the agent library — and lists it there rather
  than twice. Beside the open row, eight token-and-primitive groups render the
  *effective* settings from `terminalsummary.ts`, a pure formatter over the same Seam
  2 resolve the island mounts with; a value the file set is emphasised, an unset one
  still shows the default in force, and a colour row carries the resolved swatch. No
  write-back control. Driven live, including the editor launch; the drive also caught
  that `terminal.toml` was never gitignored.
  [ticket](tickets/08-settings-terminal-section.md)

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
