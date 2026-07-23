---
type: task
blocked_by: [01]
---

# Layered theme + named presets

## Question

An operator picks a named theme preset in one line and optionally overrides
individual colour slots on top of it; anything left unset still falls through to
the app's token-derived default. Extend the theme across all sixteen ANSI slots
plus background, foreground, cursor, cursorAccent, and selection.

Implement the layering in the resolve builder (Seam 2) as tokens → named preset →
explicit slots. Bundle a handful of presets by name (Dracula, Solarized, Nord,
Gruvbox, and similar) as data, no network fetch. The six ANSI hues currently
hard-coded in `buildTheme` become the default preset layer, so the zero-config
look is unchanged. The server parse (Seam 1) accepts a `preset` key and per-slot
colour keys and warns on an unknown preset name or an invalid colour while keeping
the default.

Tests lead: extend the Seam 2 table test — preset applies, an explicit slot wins
over the preset, an unset slot resolves to the token default; and the Seam 1 test
— unknown preset and bad colour each warn and fall back.

Done when: selecting a preset re-themes every terminal; a per-slot override wins;
an unset slot matches today's default; unknown preset / bad colour warn without
breaking the terminal; seam tests green; frontend + Go checks pass.

## Answer

The theme is now layered and preset-driven end to end. An operator names a bundled
preset in one line and overrides any individual slot on top of it; anything unset
still falls through to the token-derived default, so the zero-config look is
unchanged.

- **Seam 1 (server, `internal/config/terminal.go`):** `TerminalPrefs` widened to a
  `Preset` name plus the full slot set — the five base slots (background,
  foreground, cursor, cursorAccent, selection) and the sixteen ANSI slots. Every
  slot validates through the same `validColour` (a `#hex` lands, anything else warns
  by its dotted key and stays unset). `preset` is validated against a bundled
  `terminalPresets` set, normalised to its lower-case key; an unknown name warns and
  leaves the default theme standing. The palettes are *not* here — colour is the
  resolve seam's job (ADR 0010) — so this side carries only a validated name. New
  cases in `terminal_test.go`; the wire mirror (`model.TerminalPrefs`) and its
  server copy (`modelTerminalPrefs` in `spaces.go`) widened in lockstep.
- **Seam 2 (client, `web/src/lib/tokens.ts`):** `buildTerminalOptions` stacks three
  layers — token-derived base (with the six ANSI hues as the default preset layer,
  so today's look is the zero-config baseline) → named preset → explicit per-slot
  overrides. Five presets are bundled as inline data (`PRESETS`: Dracula, Nord,
  Gruvbox, Solarized-dark, Solarized-light), by the same names the server validates.
  A `SLOT_KEYS` table drives the override layer (mapping `selection` →
  `selectionBackground`). Table-tested in `tokens.test.ts`: a preset applies, an
  explicit slot wins over it, an unset slot resolves to the token/default layer, and
  an unknown name is ignored defensively.
- **Island:** unchanged — it already consumes the resolved options and remounts on
  a prefs change (ticket 01). `SpacePane`'s remount key now stringifies the whole
  prefs object, so any slot or preset edit re-applies without the key having to
  track the growing set.

Both seam tests green; frontend `check`/`build`/`vitest` (98) and
`go vet`/`go test ./...` pass; no amber in the built CSS (the preset hues are
island data in JS, fed at the seam, never chrome CSS).
