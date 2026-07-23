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
