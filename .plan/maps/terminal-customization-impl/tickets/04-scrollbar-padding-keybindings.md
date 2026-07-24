---
type: task
blocked_by: [01]
---

# Scrollbar, padding & keybindings

## Question

An operator controls the terminal's scrollbar and padding and rebinds a few keys —
the small island tweaks that need no addon.

- **Scrollbar & padding (CSS):** width, thumb colour, track colour, and auto-hide
  for the scrollbar (targeting the viewport element), and padding around the grid,
  all driven as CSS custom properties on the island host — xterm exposes no
  options for either. A padding change is followed by a refit so the shell reflows
  to the corrected column/row count.
- **Keybindings:** Shift+Enter inserts a newline instead of submitting (a pure
  `event → action` predicate in the resolve module, wired into the island via the
  custom key-event handler), plus copy-on-select, right-click-selects-word, and
  macOS-option-is-meta.

Server-side validation keeps defaults and warns on bad values.

Tests lead: the Seam 2 table test asserts the Shift+Enter predicate maps to a
newline action while a plain Enter submits; the resolve builder emits the expected
CSS-custom-prop values and pass-through selection/key options. The visual result
(scrollbar look, padding, refit) is trusted through the resolved values, not
unit-tested.

Done when: scrollbar and padding follow `terminal.toml` and the shell refits after
a padding change; Shift+Enter adds a newline while Enter still submits; selection
and meta behaviours apply; seam tests green; frontend + Go checks pass.

## Answer

The small island tweaks now ride the same pipeline as everything else — three new
`terminal.toml` blocks, resolved server-side and spent at the seam as CSS custom
properties, xterm options, and one key predicate.

- **Seam 1 (`internal/config/terminal.go`):** `[scrollbar]` (`width`, `thumb`,
  `track`, `autoHide`), `[padding]` (`top`/`right`/`bottom`/`left`), and `[keys]`
  (`shiftEnterNewline`, `copyOnSelect`, `rightClickSelectsWord`, `macOptionIsMeta`).
  No new validation machinery was needed — the existing `validPositive` /
  `validNonNegative` / `validColour` guards cover all of it, and the four key prefs
  are `*bool` tri-states TOML's own decode already type-checks. Padding is written
  per side with no `all` shorthand on purpose: each side is unset-at-zero, which a
  base value would muddle (an explicit `top = 0` would be indistinguishable from
  "inherit the base"). `shiftEnterNewline` is the one pref whose unset default is
  *on* — the Ghostty newline is the capability this file exists to deliver, so
  `false` is what restores plain submit.
- **Seam 2 (`web/src/lib/tokens.ts`):** `buildTerminalOptions` grew a third return,
  `css` — the scrollbar and padding as CSS custom properties, since xterm exposes no
  option for either. Only a property the file actually set is emitted, so every
  `var(…, fallback)` in `app.css` keeps today's look untouched. Auto-hide rides
  `--terminal-scrollbar-thumb-idle` (the thumb's colour *at rest*, set transparent),
  which `.terminal-island:hover` paints back — no data attribute, no JS. Beside it,
  `terminalKeyAction(event, prefs)` is the pure `event → action` decision:
  `newline` | `submit` | `default`, keydown-only so the matching keyup can't write
  the newline twice, and never intercepting an Enter that carries Ctrl/Alt/Meta.
- **Island (`Terminal.svelte`):** sets the resolved properties on its host before
  opening, wires `attachCustomKeyEventHandler` to the predicate (`xterm.input('\n')`
  puts the newline through the same onData path a typed key takes), passes
  `rightClickSelectsWord`/`macOptionIsMeta` through as options, and wires
  copy-on-select to `onSelectionChange` — the one selection behaviour xterm has no
  option for.

**One structural change the runtime forced.** Driving it headless first showed the
padding *not* reflowing the shell: the fit addon sizes the grid from its parent's
computed width, and a browser reports a `border-box` element's computed width as
the **border** box — so a padded host measured as the whole pane and the grid
overflowed its own padding. The island is now two elements: `.terminal-island`
carries the padding, and an unpadded `.terminal-grid` inside it is what xterm mounts
into and what the fit addon measures. Verified end to end against the real `app.css`
and the real resolve: 100×26 unpadded → 92×23 with 24/24/24/40 padding. The padded
frame is painted with the terminal's own resolved background (also fed in at the
seam), so a Dracula terminal is not framed in a token-coloured surround.

Also verified live: the resolved prefs ride the real snapshot over the control
socket; the scrollbar thumb/track reach the viewport (`scrollbar-color:
rgb(255,0,0) rgb(0,0,255)`); Shift+Enter emits `\n` while plain Enter emits `\r`,
Shift+Alt+Enter is left to xterm, and `shiftEnterNewline = false` returns `submit`.

Seam tests extended and green — Seam 2 asserts the CSS properties, the pass-through
options, and the key predicate; Seam 1 asserts the three blocks resolve and that a
bad scrollbar/padding value warns while a zero padding side stays silent. Frontend
`check`/`build`/`vitest` (110) and `go vet`/`go test ./...` pass; no amber in the
built CSS.
