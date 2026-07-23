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
