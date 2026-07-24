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

## Answer

The remaining pass-through terminal options now flow the whole pipeline, and the
unicode11 addon gives correct wide-glyph widths — each landing on remount off the
same `terminal.toml`.

- **Seam 1 (server, `internal/config/terminal.go`):** `TerminalPrefs` widened with
  the font block (`weight`, `boldWeight`, `lineHeight`, `letterSpacing`), a
  `[cursor]` block (`style`, `blink`, `inactiveStyle`, `width`), a `[scrolling]`
  block (`scrollback`, `sensitivity`, `fastScrollModifier`,
  `fastScrollSensitivity`, `smoothScrollDuration`), `[accessibility]`
  (`minimumContrastRatio`), and `[glyph]` (`unicode11`). Four reusable guards do the
  validation — `validEnum` (closed sets: cursor styles, inactive styles, fast-scroll
  modifiers), `validPositive` / `validNonNegative` (sign, with zero as "unset"),
  `validRange` (the 1..21 contrast floor, its low bound doubling as unset), and
  `validWeight`, which accepts a keyword *or* a number (weight decodes as an
  `interface{}` so `weight = "bold"` and `weight = 600` both land without a
  type-mismatch nuking the file) and normalises to one wire string. `blink` and
  `unicode11` are `*bool` tri-states so an explicit `false` is distinguishable from
  unset. Each bad value warns by its dotted key and keeps the default.
- **Seam 2 (client, `web/src/lib/tokens.ts`):** `buildTerminalOptions` maps every
  new pref 1:1 onto the xterm options via a typed `setOpt` that assigns only when the
  value is defined, so an unset pref never clobbers xterm's own default. A numeric
  weight string resolves to a number, a keyword passes through. Font family now
  routes through `resolveFontFamily` + a `BUNDLED_FONTS` curation: a bundled name
  (IBM Plex Mono today; the ligatures ticket grows the list) resolves to its clean
  offline stack, a non-bundled family stacks ahead of the system fallback. The
  numeric/enum guards mirror the server so the seam is defensively safe on its own.
- **Island (`Terminal.svelte`):** consumes the widened options as before; `cursorBlink`
  is composed as `(pref ?? true) && term.alive` so a dead shell never blinks. The
  unicode11 addon is lazily `import()`ed and activated (`unicode.activeVersion = '11'`)
  only when `prefs.unicode11` is set — it code-splits into its own bundled chunk
  (`dist/assets/addon-unicode11-*.js`), no CDN. The remount key already stringifies
  the whole prefs object, so every new field re-applies for free.

Seam tests extended and green — Seam 2 asserts each option maps (incl. bundled vs
non-bundled family and defensive drops), Seam 1 warns on out-of-range values.
Frontend `check`/`build`/`vitest` (103) and `go vet`/`go test ./...` pass; no amber
in the built CSS (the addon chunk is JS, fed at the seam).
