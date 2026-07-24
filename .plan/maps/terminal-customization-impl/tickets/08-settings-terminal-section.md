---
type: task
blocked_by: [01]
---

# Settings "Terminal" section

## Question

An operator sees their current terminal settings in the Settings surface and opens
`terminal.toml` in their own editor from there — read-value-plus-open-file, never a
second config store.

Add a "Terminal" section on the **Global** scope of the Settings surface (beside
the user config — these are per-machine cosmetic settings, not per space). It
renders the current effective settings read from the snapshot and an
open-`terminal.toml` row using the existing files-on-disk / open-in-editor pattern
(the same `ConfigLayer` open path the agent-library files use). Built on
design-system tokens and vendored primitives with Phosphor icons; no raw colour,
no amber.

Any parse/validation warnings already flow to the config-warnings surface (ticket
01); this section does not re-implement that — it may point at it.

Done when: the Global scope shows a Terminal section with the current effective
settings and an open-`terminal.toml` row that launches the operator's editor;
built on tokens + primitives; frontend + Go checks pass.

## Answer

Shipped. `terminal.toml` became a named config layer server-side
(`layerTerminalConfig`, `holds: "terminal"`), so the surface opens it through the
same space-less named-layer action the agent library and skill roots use — the
client still never sends a path. The Global scope grew a `TerminalSettings.svelte`
section: the intro, the file's own open row (the shared `layerRow` snippet, passed
in so the open action, its busy state and the editor-ladder note stay owned by
`Settings.svelte`), and eight token-and-primitive groups — Font, Rendering, Theme,
ANSI palette, Cursor, Scrolling, Scrollbar & padding, Keys & selection. The file is
listed there instead of under "Files on disk", so one file is one row.

The values come from `terminalsummary.ts`, a pure formatter over the *same* Seam 2
resolve the island mounts with (`buildTerminalOptions` + `resolveRenderer`), which
is what keeps the surface from ever showing one thing while the terminal does
another. Every row carries `set`: a value the file named renders emphasised, a row
it left alone still shows the default genuinely in force (xterm's own, the island's
alive-gated blink, the token-derived colour) in muted text. Colour rows carry the
*resolved* swatch, so an unset slot shows the colour the terminal actually paints —
the only place a concrete colour is painted from data, the same exempt-chromatic
class as the star-map's status hues. No write-back control exists anywhere in it.

Driven live in the real binary against a fully-populated `terminal.toml` (dracula
preset with a `#ffb86c` cursor override, 14px/1.2 IBM Plex Mono, bar cursor, 5000
lines, smooth scroll, auto-hiding 10px scrollbar, asymmetric padding, copy-on-select):
every group rendered with the right values on the right side of set-vs-default, the
sixteen ANSI swatches showed the preset's palette, and activating the open row
launched `$VISUAL` on the resolved absolute path. One gap the drive exposed:
`terminal.toml` was never gitignored, though it lands at the data root exactly like
`user.toml` — fixed.

Gist: a named `terminal-config` layer plus a read-only Terminal section built on a
pure formatter over the same resolve the island mounts with.
