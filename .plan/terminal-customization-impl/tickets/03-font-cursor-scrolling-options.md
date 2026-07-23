---
type: task
blocked_by: [01]
---

# Font, cursor, scrolling & glyph options

## Question

An operator tunes the remaining pass-through terminal options and gets correct
wide-glyph widths. Extend `TerminalPrefs` and the resolve builder to cover:

- **Font:** weight, bold weight, line height, letter spacing, plus a curated
  bundled-font list selectable by name and a custom family string that falls back
  to a system stack (ligatures only work for a bundled font — that constraint is
  the ligatures ticket's, here we just resolve the family).
- **Cursor:** style (block / bar / underline), blink, inactive-cursor style,
  width.
- **Scrolling:** scrollback length, scroll sensitivity, fast-scroll modifier and
  sensitivity, smooth-scroll duration.
- **Accessibility:** minimumContrastRatio.
- **Glyph widths:** the unicode11 addon, lazily imported and activated at mount
  when enabled, for correct wide-glyph/emoji widths.

Server-side validation keeps a default and warns on any out-of-range or unknown
value. The unicode11 addon is bundled; no network fetch.

Tests lead: extend the Seam 2 table test so each option maps into the xterm
options object (incl. non-bundled family → system fallback), and the Seam 1 test
warns on an out-of-range value.

Done when: each option set in `terminal.toml` takes effect on remount; a
non-bundled family falls back cleanly; wide glyphs align with unicode11 on; bad
values warn and default; seam tests green; frontend + Go checks pass.
